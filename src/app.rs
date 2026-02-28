use crate::AppError;
use crate::bootables::DisplayOptions;
use crate::display::GopDisplay;
use crate::path::DiskManager;
use uefi::proto::console::text::Input;

/// Outcome for a blocking app run.
pub enum AppResult {
    /// App finished and returned control to the caller.
    Done,
    /// App initiated a boot flow and should not return to the caller.
    Booted,
    /// App encountered an unrecoverable error during execution.
    Error(AppError),
}

/// Borrowed UI and system resources for a running app.
pub struct AppCtx<'a> {
    /// Display buffer for drawing. Caller retains ownership.
    pub display: &'a mut GopDisplay<'a>,
    /// Input source for key events. Caller retains ownership.
    pub input: &'a mut Input,
    /// Disk access helpers scoped to the caller's lifetime.
    pub disk_manager: &'a DiskManager,
    /// Image handle for UEFI service calls.
    pub handle: uefi::Handle,
}

/// Blocking app entry point.
pub trait App {
    fn run(&mut self, ctx: &mut AppCtx) -> AppResult;
}

/// A trait that defines an App that can be drawn as an entry
/// in the boot menu.
pub trait DisplayEntry {
    fn display_options(&self) -> DisplayOptions;
}
