#![no_main]
#![no_std]
extern crate alloc;

mod boot_menu;
mod bootables;
mod config;
mod display;
mod errors;
#[cfg(feature = "iso")]
mod iso;
mod path;

pub use errors::AppError;

use log::info;
use uefi::{
    prelude::*,
    proto::console::{gop::GraphicsOutput, text::Input},
};

use crate::bootables::Boot;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Initialized UEFI helpers successfully.");

    // Load configuration from fixed path
    const CONFIG_PATH: &str = "\\plex.toml";
    let config = match config::Config::load_from_file(CONFIG_PATH) {
        Ok(cfg) => cfg,
        Err(e) => {
            log::error!("Failed to load config from {}: {:?}", CONFIG_PATH, e);
            return Status::LOAD_ERROR;
        }
    };

    info!(
        "Loaded {} boot targets from config",
        config.boot_targets.len()
    );

    let handle = boot::image_handle();
    let manager = path::DiskManager::new(handle).unwrap();
    let boot_targets = config.into_boot_targets();

    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();

    let input_handle = boot::get_handle_for_protocol::<Input>().unwrap();
    let mut input = boot::open_protocol_exclusive::<Input>(input_handle).unwrap();

    let menu = boot_menu::BootMenu::<bootables::BootTarget>::new(&mut gop, boot_targets.as_slice());
    menu.run(&mut input)
        .unwrap()
        .boot(handle, &manager)
        .unwrap();

    Status::SUCCESS
}
