use winapi::shared::minwindef::{HKEY, DWORD};
use winreg::enums::*;
use std::ffi::CString;
use std::ptr;
use failure::Error;
use std::ffi::{OsString, OsStr};
use std::io;
use std::os::windows::ffi::OsStrExt;
use winapi::um::winnt::REG_NOTIFY_CHANGE_NAME;
use winapi::um::synchapi::{CreateEventW, WaitForMultipleObjects};
use winapi::um::winreg::RegNotifyChangeKeyValue;
use winapi::shared::minwindef::{TRUE, FALSE};
use winapi::um::winbase::INFINITE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winreg::RegCloseKey;
use failure::err_msg;
use software_manager::SoftwareInfo;
use winreg::RegKey;

macro_rules! werr {
    ($e:expr) => (
        Err(io::Error::from_raw_os_error($e as i32))
    )
}

fn to_utf16<P: AsRef<OsStr>>(s: P) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0).into_iter()).collect()
}

fn get_reg_key_with_flag<S: AsRef<OsStr>>(root: HKEY, subpath: S, mask: u32) -> Result<HKEY, Error> {
    use winapi::um::winreg::RegOpenKeyExW;
    let c_path = to_utf16(subpath);

    let mut new_hkey = ptr::null_mut();
    match unsafe {
        RegOpenKeyExW(
            root,
            c_path.as_ptr(),
            0,
            mask,
            &mut new_hkey,
        ) as DWORD
    } {
        0 => Ok(new_hkey),
        err => werr!(err).map_err(|e| e.into())
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub enum SoftWareChangeMsg {
    X64_64_LM,
    X64_64_CU,
    X64_32_LM,
    X86_CU,
    X86_LM,
}

const uninstall_software_key: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
const notify_32_mask: u32 = KEY_NOTIFY | KEY_WOW64_32KEY;
const notify_64_mask: u32 = KEY_NOTIFY | KEY_WOW64_64KEY;

pub fn notify_software_change() -> Result<SoftWareChangeMsg, Error> {
    unsafe {
        let x64_32_lm_key = get_reg_key_with_flag(HKEY_LOCAL_MACHINE, uninstall_software_key, notify_32_mask)?;
        let x64_64_lm_key = get_reg_key_with_flag(HKEY_LOCAL_MACHINE, uninstall_software_key, notify_64_mask)?;
        let x64_64_cu_key = get_reg_key_with_flag(HKEY_CURRENT_USER, uninstall_software_key, notify_64_mask)?;

        let x64_32_lm_event = CreateEventW(ptr::null_mut(), FALSE, TRUE, ptr::null());
        let x64_64_lm_event = CreateEventW(ptr::null_mut(), FALSE, TRUE, ptr::null());
        let x64_64_cu_event = CreateEventW(ptr::null_mut(), FALSE, TRUE, ptr::null());

        let events = vec![x64_32_lm_event, x64_64_lm_event, x64_64_cu_event];
        //TODO check res of RegNotifyChangeKeyValue
        RegNotifyChangeKeyValue(x64_32_lm_key, TRUE, REG_NOTIFY_CHANGE_NAME, x64_32_lm_event, TRUE);
        RegNotifyChangeKeyValue(x64_64_lm_key, TRUE, REG_NOTIFY_CHANGE_NAME, x64_64_lm_event, TRUE);
        RegNotifyChangeKeyValue(x64_64_cu_key, TRUE, REG_NOTIFY_CHANGE_NAME, x64_64_cu_event, TRUE);

        let res = WaitForMultipleObjects(events.len() as u32, events.as_ptr(), FALSE, INFINITE);
        //TODO correspond https://msdn.microsoft.com/en-us/library/windows/desktop/ms687025(v=vs.85).aspx handle all error
        match res {
            0 => Ok(SoftWareChangeMsg::X64_32_LM),
            1 => Ok(SoftWareChangeMsg::X64_64_LM),
            2 => Ok(SoftWareChangeMsg::X64_64_CU),
            _ => {
                //TODO use GetLastError
                Err(err_msg("get in invalid return value of WaitForMultipleObjects"))
            }
        }
    }
}

pub fn get_software_from_reg<S: AsRef<OsStr>>(root: HKEY, subpath: S, mask: u32) -> Vec<SoftwareInfo> {
    fn clear_quote(data: String) -> String {
        data.trim_matches(|c| c == '"').to_string()
    }
    info!("get_software_from_reg {:?}", subpath.as_ref());
    RegKey::predef(root).open_subkey_with_flags(subpath, mask)
        .and_then(|reg_key| {
            info!("staer enum");
            let software_list = reg_key
                .enum_keys()
                .map(|k| {
                    info!("{:?}", k);
                    k
                })
                .filter_map(|key| key.ok())
                .filter_map(|key| reg_key.open_subkey_with_flags(key, KEY_READ).ok())
                .map(|item| {
                    info!("staer get data");
                    let icon: String = item.get_value("DisplayIcon").unwrap_or_default();
                    let name: String = item.get_value("DisplayName").unwrap_or_default();
                    let version: String = item.get_value("DisplayVersion").unwrap_or_default();
                    let vendor: String = item.get_value("Publisher").unwrap_or_default();
                    let installtime: String = item.get_value("InstallDate").unwrap_or_default();
                    let installlocation = item.get_value("InstallLocation").unwrap_or_default();
                    let installlocation = clear_quote(installlocation);
                    let uninstallstring = item.get_value("uninstallString").unwrap_or_default();
                    let clear_uninstallstring = clear_quote(uninstallstring);
                    info!("software name is {}", name);
                    SoftwareInfo {
                        name: name.clone(),
                        caption: name.clone(),
                        desc: name.clone(),
                        installtime: installtime.clone(),
                        installlocation: installlocation.clone(),
                        version: version.clone(),
                        vendor: vendor.clone(),
                        uninstallstring: clear_uninstallstring.clone(),
                        icon: icon.clone(),
                    }
                })
                .collect();
            info!("this over ");
            Ok(software_list)
        }).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::util::*;

    #[test]
    fn test_get_software_from_reg() {
        log_init();
        get_software_from_reg(HKEY_LOCAL_MACHINE, uninstall_software_key, notify_32_mask);
    }
}
