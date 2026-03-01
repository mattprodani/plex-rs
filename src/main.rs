#![no_main]
#![no_std]

//! The main entry point for the Plex UEFI bootloader.
//!
//! Initializes UEFI services, loads the configuration from disk,
//! sets up graphics and input protocols, and launches the graphical
//! boot menu.

extern crate alloc;
use log::info;
use plex::config::Config;
use plex::core::app::{App, AppCtx, AppResult};
use plex::core::bootables::BootTarget;
use plex::core::display::GopDisplay;
use plex::path::DiskManager;
use plex::ui;
use uefi::{
    prelude::*,
    proto::console::{gop::GraphicsOutput, text::Input},
};

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Initialized UEFI helpers successfully.");

    const CONFIG_PATH: &str = "\\plex.toml";
    let config = match Config::load_from_file(CONFIG_PATH) {
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
    let disk_manager = DiskManager::new(handle).unwrap();
    let mut boot_targets = config.into_boot_targets();

    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();

    let input_handle = boot::get_handle_for_protocol::<Input>().unwrap();
    let mut input = boot::open_protocol_exclusive::<Input>(input_handle).unwrap();

    let mut display = GopDisplay::new(&mut gop);
    let mut app_ctx = AppCtx {
        display: &mut display,
        input: &mut input,
        disk_manager: &disk_manager,
        handle,
    };
    let mut menu = ui::boot_menu::BootMenu::<BootTarget>::new(boot_targets.as_mut_slice());
    if let AppResult::Error(ref err) = menu.run(&mut app_ctx) {
        let mut overlay = ui::overlay::ErrorOverlay::new(err);
        let _ = overlay.run(&mut app_ctx);
    }

    Status::SUCCESS
}
