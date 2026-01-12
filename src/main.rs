#![no_main]
#![no_std]
extern crate alloc;

mod boot_menu;
mod bootables;
mod display;
mod errors;

use log::info;
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Initialized UEFI helpers successfully.");
    todo!("implement config loading");

    // let handle = boot::image_handle();
    // run(handle);
    // uefi::boot::stall(Duration::from_secs(5));
    //
    // Status::SUCCESS
}
