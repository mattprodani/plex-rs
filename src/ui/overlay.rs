//! UI overlays for displaying messages and errors.
//!
//! Provides reusable graphical overlays that can be drawn on top of
//! the current screen, such as error dialogs.

use crate::AppError;
use crate::core::app::{App, AppCtx, AppResult};
use crate::ui::theme::Theme;
use uefi::proto::console::text::{Key, ScanCode};

/// An error overlay that displays an application error and waits for the user
/// to acknowledge it before dismissing.
pub struct ErrorOverlay<'a> {
    error: &'a AppError,
    theme: Theme,
}

impl<'a> ErrorOverlay<'a> {
    /// Creates a new error overlay.
    pub fn new(error: &'a AppError, theme: Theme) -> Self {
        Self { error, theme }
    }
}

impl<'a> App for ErrorOverlay<'a> {
    fn run(&mut self, ctx: &mut AppCtx) -> AppResult {
        if let Err(e) = self.theme.draw_error_overlay(ctx, self.error) {
            log::error!("failed to draw error overlay: {}", e);
        }

        loop {
            let mut events = [unsafe { ctx.input.wait_for_key_event().unwrap_unchecked() }];

            if uefi::boot::wait_for_event(&mut events).is_err() {
                return AppResult::Error(uefi::Status::INVALID_PARAMETER.into());
            }

            if let Ok(Some(key)) = ctx.input.read_key() {
                if matches!(key, Key::Printable(c) if c == '\r' || c == '\n') {
                    return AppResult::Done;
                }
                if matches!(
                    key,
                    Key::Special(ScanCode::END) | Key::Special(ScanCode::ESCAPE)
                ) {
                    return AppResult::Done;
                }
            }
        }
    }
}
