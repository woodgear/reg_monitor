extern crate winreg;
extern crate winapi;
extern crate failure;

#[macro_use]
extern crate actix;

use actix::prelude::*;

mod notify_software_change;

use notify_software_change::notify_software_change;

fn main() {
    loop {
        println!("start monitor");
        let res = notify_software_change();
        println!("monitor change {:?}", res);
    }
}
