extern crate winreg;
extern crate winapi;
extern crate failure;
#[macro_use]
extern crate actix;
#[macro_use]
extern crate sugar;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate actix_derive;
extern crate simple_logger;

use actix::prelude::*;

mod software_util;
mod software_manager;
mod util;

use util::*;
use software_manager::{SoftwareInfo, SoftwareManager};
use software_util::{notify_software_change, SoftWareChangeMsg};
use std::collections::{BTreeSet, HashMap};
use std::thread;
use std::time::Duration;
use winapi::um::winnt::KEY_NOTIFY;
use winapi::um::winnt::KEY_WOW64_32KEY;
use winapi::um::winnt::KEY_WOW64_64KEY;
use winapi::um::winreg::HKEY_LOCAL_MACHINE;
use winapi::um::winreg::HKEY_CURRENT_USER;

// save software list which from different registery
struct SoftwareMap {
    keys: BTreeSet<String>,
    values: HashMap<String, SoftwareInfo>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct SoftwareChange {
    pub install: Vec<SoftwareInfo>,
    pub uninstall: Vec<SoftwareInfo>,
}

impl SoftwareMap {
    fn new(software: Vec<SoftwareInfo>) -> Self {
        let keys: BTreeSet<String> = software.iter().map(|s| s.name.clone()).collect();
        let values = c! {s.name.clone() => s, for s in software};
        SoftwareMap {
            keys,
            values,
        }
    }

    fn diff(&self, other: &SoftwareMap) -> SoftwareChange {
        info!("software diff len self {} other {}", self.keys.len(), other.keys.len());

        let uninstall_keys: Vec<_> = self.keys.difference(&other.keys).cloned().collect();
        let uninstall_values: Vec<SoftwareInfo> = uninstall_keys.iter().filter_map(|name| {
            self.values.get(name)
        }).map(|v| {
            (*v).clone()
        })
            .collect();

        let install_keys: Vec<_> = other.keys.difference(&self.keys).cloned().collect();
        let install_values: Vec<SoftwareInfo> = install_keys.iter().filter_map(|name| {
            other.values.get(name)
        }).map(|v| {
            (*v).clone()
        })
            .collect();


        SoftwareChange {
            install: install_values,
            uninstall: uninstall_values,
        }
    }
    fn get_software(&self) -> Vec<SoftwareInfo> {
        self.values.values().cloned().collect()
    }
}

type SoftwareChangeSubscriber = Recipient<Syn, SoftwareChange>;

pub struct SoftwareService {
    software_32_lm: SoftwareMap,
    software_64_lm: SoftwareMap,
    software_cu: SoftwareMap,
    subscribers: Vec<SoftwareChangeSubscriber>,
}

impl SoftwareService {
    fn new() -> Self {
        const uninstall_software_key: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
        const notify_32_mask: u32 = KEY_NOTIFY | KEY_WOW64_32KEY;
        const notify_64_mask: u32 = KEY_NOTIFY | KEY_WOW64_64KEY;
        info!("in new");

        let software_32_lm = SoftwareManager::find_from_32_lm();
        info!("after  software_32_lm");

        let software_64_lm = vec![];// SoftwareManager::find_from_64_lm();
        let software_cu = vec![];//SoftwareManager::find_from_cu();
        Self {
            software_32_lm: SoftwareMap::new(software_32_lm),
            software_64_lm: SoftwareMap::new(software_64_lm),
            software_cu: SoftwareMap::new(software_cu),
            subscribers: vec![],
        }
    }
}

impl Actor for SoftwareService {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("SoftwareService start");
        let address: Addr<Syn, _> = ctx.address();

// start a thread to monitor software
        let _ = thread::Builder::new().name("SoftwareService".to_string()).spawn(move || {
            loop {
                match notify_software_change() {
                    Ok(change) => {
                        info!(" software_registry change {:?}", change);
                        address.do_send(change);
                    }
                    Err(err) => {
                        error!("notify_software_registry_change fail {}", err.to_string());
                        thread::sleep(Duration::from_secs(3));
                    }
                }
            }
        }).unwrap();
    }
}

impl Handler<SoftWareChangeMsg> for SoftwareService {
    type Result = ();
    fn handle(&mut self, msg: SoftWareChangeMsg, _: &mut Context<Self>) -> Self::Result {}
}


fn main() {
    use std::env;
    log_init();
    SoftwareService::new();
//    let system = System::new("software-service");
//
//    let _address: Addr<Syn, _> = SoftwareService::new().start();
//    system.run();
}
