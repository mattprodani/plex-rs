//! UI overlays for displaying messages and errors.
//!
//! Provides reusable graphical overlays that can be drawn on top of
//! the current screen, such as error dialogs.

use crate::AppError;
use crate::core::app::{App, AppCtx, AppResult};
use alloc::string::{String, ToString as _};
use alloc::vec::Vec;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X15},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use uefi::proto::console::text::{Key, ScanCode};

/// An error overlay that displays an application error and waits for the user
/// to acknowledge it before dismissing.
pub struct ErrorOverlay<'a> {
    error: &'a AppError,
}

impl<'a> ErrorOverlay<'a> {
    /// Creates a new error overlay.
    pub fn new(error: &'a AppError) -> Self {
        Self { error }
    }
}

impl<'a> App for ErrorOverlay<'a> {
    fn run(&mut self, ctx: &mut AppCtx) -> AppResult {
        let text = alloc::format!("{}", self.error);
        draw_error_overlay(ctx, &text);

        if let Err(err) = ctx.display.flush() {
            return AppResult::Error(err.into());
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
                if matches!(key, Key::Special(ScanCode::END)) {
                    return AppResult::Done;
                }
            }
        }
    }
}

fn draw_error_overlay(ctx: &mut AppCtx, text: &str) {
    let size = ctx.display.size();
    let screen_w = size.width as i32;
    let screen_h = size.height as i32;
    let box_w = (screen_w * 2 / 3).max(280);
    let box_h = (screen_h / 3).max(120);
    let left = (screen_w - box_w) / 2;
    let top = (screen_h - box_h) / 2;

    let background = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::new(20, 20, 20))
        .stroke_color(Rgb888::new(220, 220, 220))
        .stroke_width(2)
        .build();
    Rectangle::new(Point::new(left, top), Size::new(box_w as u32, box_h as u32))
        .into_styled(background)
        .draw(ctx.display)
        .ok();

    let title_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 80, 80));
    let body_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);

    let padding_x = 12;
    let padding_y = 16;
    let line_height = 18;
    let max_chars = ((box_w - padding_x * 2) / 9).max(1) as usize;
    let max_lines = ((box_h - padding_y * 2) / line_height).max(1) as usize;

    Text::new(
        "Error",
        Point::new(left + padding_x, top + padding_y),
        title_style,
    )
    .draw(ctx.display)
    .ok();

    let lines = wrap_lines(text, max_chars, max_lines.saturating_sub(1));
    for (idx, line) in lines.iter().enumerate() {
        let y = top + padding_y + line_height * (idx as i32 + 1);
        Text::new(line.as_str(), Point::new(left + padding_x, y), body_style)
            .draw(ctx.display)
            .ok();
    }
}

fn wrap_lines(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let mut start = 0;
        let bytes = raw.as_bytes();
        while start < bytes.len() {
            let end = (start + max_chars).min(bytes.len());
            let slice = &raw[start..end];
            lines.push(slice.to_string());
            start = end;
            if lines.len() >= max_lines {
                return lines;
            }
        }
        if lines.len() >= max_lines {
            return lines;
        }
    }
    lines
}
