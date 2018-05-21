extern crate winreg;
extern crate winapi;
extern crate failure;

use winapi::shared::minwindef::{HKEY,DWORD};
use winreg::enums::*;
use std::ffi::CString;
use std::ptr;
use failure::Error;
use std::ffi::{OsString,OsStr};
use std::io;
use std::os::windows::ffi::OsStrExt;

macro_rules! werr {
    ($e:expr) => (
        Err(std::io::Error::from_raw_os_error($e as i32))
    )
}

fn to_utf16<P: AsRef<OsStr>>(s: P) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0).into_iter()).collect()
}

fn get_reg_key_with_flag(root:HKEY,subpath:String,mask:u32)->Result<HKEY,Error> {
    use winapi::um::winreg::RegOpenKeyExW;
//    let c_path = to_utf16("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall");
    let c_path = to_utf16(subpath);

//    let mask = KEY_NOTIFY | KEY_WOW64_32KEY;
    let mut new_hkey= ptr::null_mut();
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
        err => werr!(err).map_err(|e|e.into())
    }
}

enum SoftWareChangeMsg{
    X64_64_LM,
    X64_64_CU,
    X64_86_LM,
}


fn notify_software() -> Result<SoftWareChangeMsg, Error> {
    let w64_32_key = get_reg_key_with_flag(HKEY_LOCAL_MACHINE,
                                     "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string(),
                                     KEY_NOTIFY | KEY_WOW64_32KEY)?;
    let w64_64_key = get_reg_key_with_flag(HKEY_LOCAL_MACHINE,
                                     "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string(),
                                     KEY_NOTIFY | KEY_WOW64_64KEY)?;

//    let hkey = get_reg_key_with_flag(HKEY_LOCAL_MACHINE,
//                                     "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string(),
//                                     KEY_NOTIFY | KEY_WOW64_32KEY)?;

    wait_key_change(hkey)
}


struct WaitItem {
    key:HKEY
}

fn wait_key_change(keys:Vec<HKEY>)->Result<(),Error> {
    use winapi::um::winnt::REG_NOTIFY_CHANGE_NAME;
    use winapi::um::synchapi::{CreateEventW,WaitForMultipleObjects};
    use winapi::um::winreg::RegNotifyChangeKeyValue;
    use winapi::shared::minwindef::{TRUE,FALSE};
    use winapi::um::winbase::INFINITE;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winreg::RegCloseKey;
    unsafe {

//        let x86_event = CreateEventW(ptr::null_mut(),FALSE,TRUE,ptr::null());
//        let x64_event = CreateEventW(ptr::null_mut(),FALSE,TRUE,ptr::null());
        let mut events = Vec::with_capacity(key.len());
        let events = events
            .map(|_|CreateEventW(ptr::null_mut(),FALSE,TRUE,ptr::null()))
            .collect();
        for key in keys {
            RegNotifyChangeKeyValue(key,TRUE,REG_NOTIFY_CHANGE_NAME,event,TRUE);
        }
        let dwEvent = WaitForMultipleObjects(2, events.as_ptr(), FALSE, INFINITE);
        println!("dwEvent {:?}",dwEvent);
        for event in events {
            CloseHandle(event);
        }
        RegCloseKey(key);
    }
    Ok(())
}

fn main() {
    loop {
        println!("start monitor");
        let key = get_reg_key_with_flag().unwrap();
        if let Ok(change)= wait_key_change(key) {
            println!("has change {:?}",change);
        }else {
            println!("without change");
        }
    }
}
