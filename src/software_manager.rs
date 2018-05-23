use std::ffi::OsStr;
use winapi::shared::minwindef::HKEY;

use software_util::get_software_from_reg;
use winapi::um::winreg::HKEY_LOCAL_MACHINE;
use winapi::um::winreg::HKEY_CURRENT_USER;
use winapi::um::winnt::KEY_NOTIFY;
use winapi::um::winnt::KEY_WOW64_32KEY;
use winapi::um::winnt::KEY_WOW64_64KEY;
use std::cmp::Ordering;

const uninstall_software_key: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
const notify_32_mask: u32 = KEY_NOTIFY | KEY_WOW64_32KEY;
const notify_64_mask: u32 = KEY_NOTIFY | KEY_WOW64_64KEY;

#[derive(Serialize, Debug, Clone)]
pub struct SoftwareInfo {
    pub name: String,
    pub caption: String,
    pub desc: String,
    pub installtime: String,
    pub installlocation: String,
    pub version: String,
    pub vendor: String,
    pub uninstallstring: String,
    #[serde(skip_serializing)]
    pub icon: String,
}

pub struct SoftwareManager {}

impl SoftwareManager {
    pub fn find_all() -> Vec<SoftwareInfo> {
        let mut list = Self::find_from_32_lm();
        list.extend(Self::find_from_64_lm());
        list.extend(Self::find_from_cu());
        list.sort_by(|left, right|
            left.name.partial_cmp(&right.name).unwrap_or(Ordering::Equal));
        return list;
    }
    pub fn find_from_32_lm() -> Vec<SoftwareInfo> {
        info!("before get_software_from_reg");
        get_software_from_reg(HKEY_LOCAL_MACHINE, uninstall_software_key, notify_32_mask)
    }
    pub fn find_from_64_lm() -> Vec<SoftwareInfo> {
        get_software_from_reg(HKEY_LOCAL_MACHINE, uninstall_software_key, notify_64_mask)
    }
    pub fn find_from_cu() -> Vec<SoftwareInfo> {
        get_software_from_reg(HKEY_CURRENT_USER, uninstall_software_key, notify_64_mask)
    }
}