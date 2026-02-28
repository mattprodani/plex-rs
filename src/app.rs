use crate::display::GopDisplay;
use crate::path::DiskManager;
use crate::AppError;
use uefi::proto::console::text::Input;

pub enum AppResult {
    Done,
    Booted,
    Error(AppError),
}

pub struct AppCtx<'a> {
    pub display: &'a mut GopDisplay<'a>,
    pub input: &'a mut Input,
    pub disk_manager: &'a DiskManager,
    pub handle: uefi::Handle,
}

pub trait App {
    fn run(&mut self, ctx: &mut AppCtx) -> AppResult;
}
