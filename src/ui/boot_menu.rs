//! Boot menu interface.
//!
//! Renders the list of configured boot targets and handles user input
//! to select and boot one.

use uefi::proto::console::text::{Key, ScanCode};

use crate::{
    AppError,
    core::app::{App, AppCtx, AppResult, DisplayEntry},
    ui::overlay::ErrorOverlay,
    ui::theme::Theme,
};

/// The main boot menu interface for displaying and selecting boot targets.
pub struct BootMenu<'a, T>
where
    T: App + DisplayEntry,
{
    targets: &'a mut [T],
    selected: usize,
    theme: Theme,
}

impl<'a, T: App + DisplayEntry> BootMenu<'a, T> {
    /// Creates a new boot menu to manage the provided list of targets.
    pub fn new(targets: &'a mut [T], theme: Theme) -> Self {
        Self {
            targets,
            selected: 0,
            theme,
        }
    }

    /// Exposes the list of boot targets.
    pub fn targets(&self) -> &[T] {
        self.targets
    }

    /// Returns the currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Draws boot options to the buff.
    pub fn draw(&mut self, ctx: &mut AppCtx) -> Result<(), AppError> {
        self.theme.draw_boot_menu(ctx, self)
    }

    /// Handle arrow key input and return the selected index when Enter is pressed.
    pub fn wait_for_selection(&mut self, ctx: &mut AppCtx) -> Result<usize, AppError> {
        loop {
            self.draw(ctx)?;

            // unchecked because Option::<NonNull>::None.unwrap_unchecked() == 0
            // due to the niche optimization with valid size and alignment.
            let mut events = [unsafe { ctx.input.wait_for_key_event().unwrap_unchecked() }];

            uefi::boot::wait_for_event(&mut events)
                .map_err(|_| uefi::Error::from(uefi::Status::INVALID_PARAMETER))?;

            // Read the key
            if let Some(key) = ctx.input.read_key()? {
                match key {
                    Key::Special(ScanCode::UP) => {
                        if self.selected > 0 {
                            self.selected -= 1;
                        } else {
                            // wrap around
                            self.selected = self.targets.len().saturating_sub(1);
                        }
                    }
                    Key::Special(ScanCode::DOWN) => {
                        if self.selected < self.targets.len().saturating_sub(1) {
                            self.selected += 1;
                        } else {
                            // wrap around
                            self.selected = 0;
                        }
                    }
                    Key::Printable(c) if c == '\r' || c == '\n' => {
                        return Ok(self.selected);
                    }
                    _ => {}
                }
            }
        }
    }
}

impl<'a, T: App + DisplayEntry> App for BootMenu<'a, T> {
    fn run(&mut self, ctx: &mut AppCtx) -> AppResult {
        loop {
            let selection = self.wait_for_selection(ctx);
            let result = match selection {
                Ok(selection) => {
                    let bootable = self.targets.get_mut(selection).unwrap();
                    bootable.run(ctx)
                }

                Err(e) => {
                    log::error!("encountered an error in boot menu loop: {e}");
                    AppResult::Done
                }
            };

            match result {
                AppResult::Done | AppResult::Yield => {
                    log::info!("returning control flow back to boot menu loop")
                }
                AppResult::Booted => {
                    log::info!("booted target successfully, exiting");
                    return result;
                }
                AppResult::Error(ref err) => {
                    let mut overlay = ErrorOverlay::new(err, self.theme);
                    if let AppResult::Error(_) = overlay.run(ctx) {
                        log::error!("the error overlay errored, oops.");
                        return result;
                    }
                }
            }
        }
    }
}
