#![no_main]
#![no_std]
extern crate alloc;

mod app;
mod boot_menu;
mod bootables;
mod config;
mod display;
mod errors;
#[cfg(feature = "iso")]
mod iso;
mod overlay;
mod path;

pub use errors::AppError;

use log::info;
use uefi::{
    prelude::*,
    proto::console::{gop::GraphicsOutput, text::Input},
};

use crate::display::GopDisplay;
use crate::{
    app::{App, AppCtx, AppResult},
    overlay::ErrorOverlay,
};

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
    let disk_manager = path::DiskManager::new(handle).unwrap();
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
    let mut menu = boot_menu::BootMenu::<bootables::BootTarget>::new(boot_targets.as_mut_slice());
    if let AppResult::Error(ref err) = menu.run(&mut app_ctx) {
        let mut overlay = ErrorOverlay::new(err);
        let _ = overlay.run(&mut app_ctx);
    }

    Status::SUCCESS
}
