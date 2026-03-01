use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X15},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use uefi::proto::console::text::{Key, ScanCode};

use crate::{
    AppError,
    core::app::{App, AppCtx, AppResult, DisplayEntry},
    core::display::GopDisplay,
    ui::overlay::ErrorOverlay,
};

/// Very simple BootMenu that displays listings, handles keyboard input.
pub struct BootMenu<'a, T>
where
    T: App,
{
    targets: &'a mut [T],
    selected: usize,
}

impl<'a, T: App + DisplayEntry> BootMenu<'a, T> {
    pub fn new(targets: &'a mut [T]) -> Self {
        Self {
            targets,
            selected: 0,
        }
    }

    /// Draws boot options to the buff.
    pub fn draw(&mut self, display: &mut GopDisplay) -> Result<(), AppError> {
        display.clear(Rgb888::new(0, 0, 0));

        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let selected_text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::BLACK);

        let start_y = 100;
        let line_height = 25;

        for (i, target) in self.targets.iter().enumerate() {
            let y = start_y + (i * line_height) as i32;
            let position = Point::new(50, y);
            let display_opts = target.display_options();

            if i == self.selected {
                // draw white bckg to indicate selected
                let rect = Rectangle::new(Point::new(40, y - 15), Size::new(400, 20));
                rect.into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
                    .draw(display)
                    .ok();
            }

            let this_text_style = if i == self.selected {
                selected_text_style
            } else {
                text_style
            };
            Text::new(display_opts.label.as_str(), position, this_text_style)
                .draw(display)
                .ok();
        }
        display.flush()?;
        Ok(())
    }

    /// Handle arrow key input and return the selected index when Enter is pressed.
    pub fn wait_for_selection(&mut self, ctx: &mut AppCtx) -> Result<usize, AppError> {
        loop {
            self.draw(ctx.display)?;

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
                        }
                    }
                    Key::Special(ScanCode::DOWN) => {
                        if self.selected < self.targets.len() - 1 {
                            self.selected += 1;
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
                    let mut overlay = ErrorOverlay::new(err);
                    if let AppResult::Error(_) = overlay.run(ctx) {
                        log::error!("the error overlay errored, oops.");
                        return result;
                    }
                }
            }
        }
    }
}
