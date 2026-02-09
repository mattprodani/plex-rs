use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X15},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use uefi::Result;
use uefi::proto::console::{
    gop::GraphicsOutput,
    text::{Input, Key, ScanCode},
};

use crate::{bootables::Boot, display::GopDisplay};

/// Very simple BootMenu that displays listings, handles keyboard input.
pub struct BootMenu<'a, T>
where
    T: Boot,
{
    targets: &'a [T],
    selected: usize,
    display: GopDisplay<'a>,
}

impl<'a, T: Boot> BootMenu<'a, T> {
    pub fn new(gop: &'a mut GraphicsOutput, targets: &'a [T]) -> Self {
        let display = GopDisplay::new(gop);
        Self {
            targets,
            selected: 0,
            display,
        }
    }

    /// Draws boot options to the buff.
    pub fn draw(&mut self) {
        self.display.clear(Rgb888::new(0, 0, 0));

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
                    .draw(&mut self.display)
                    .ok();
            }

            let this_text_style = if i == self.selected {
                selected_text_style
            } else {
                text_style
            };
            Text::new(display_opts.label.as_str(), position, this_text_style)
                .draw(&mut self.display)
                .ok();
        }
    }

    /// Handle arrow key input and return the selected index when Enter is pressed.
    pub fn run(mut self, input: &'a mut Input) -> Result<&'a T> {
        loop {
            self.draw();
            self.display.flush()?;

            // unchecked because Option::<NonNull>::None.unwrap_unchecked() == 0
            // due to the niche optimization with valid size and alignment.
            let mut events = [unsafe { input.wait_for_key_event().unwrap_unchecked() }];

            uefi::boot::wait_for_event(&mut events)
                .map_err(|_| uefi::Error::from(uefi::Status::INVALID_PARAMETER))?;

            // Read the key
            if let Some(key) = input.read_key()? {
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
                        return Ok(self.targets.get(self.selected).unwrap());
                    }
                    _ => {}
                }
            }
        }
    }
}
