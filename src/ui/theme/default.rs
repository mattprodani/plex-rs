use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X15},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

use crate::{
    AppError,
    core::app::{App, AppCtx, DisplayEntry},
    ui::boot_menu::BootMenu,
    ui::theme::LineWrapper,
};

pub fn draw_boot_menu<'a, T: App + DisplayEntry>(
    ctx: &mut AppCtx,
    menu: &BootMenu<'a, T>,
) -> Result<(), AppError> {
    let display = &mut *ctx.display;
    display.clear(Rgb888::new(0, 0, 0));

    let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
    let selected_text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::BLACK);

    let start_y = 100;
    let line_height = 25;

    for (i, target) in menu.targets().iter().enumerate() {
        let display_opts = target.display_options();
        let y = start_y + (i * line_height) as i32;
        let position = Point::new(50, y);

        if i == menu.selected() {
            // draw white bckg to indicate selected
            let rect = Rectangle::new(Point::new(40, y - 15), Size::new(400, 20));
            rect.into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
                .draw(display)
                .ok();
        }

        let this_text_style = if i == menu.selected() {
            selected_text_style
        } else {
            text_style
        };
        Text::new(display_opts.label.as_str(), position, this_text_style)
            .draw(display)
            .ok();
    }

    let size = display.size();
    let quote = plex_quotes::random_quote!();
    Text::new(quote, Point::new(50, size.height as i32 - 50), text_style)
        .draw(display)
        .ok();

    display.flush()?;
    Ok(())
}

pub fn draw_error_overlay(ctx: &mut AppCtx, error: &AppError) -> Result<(), AppError> {
    let text = alloc::format!("{}", error);

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

    let wrapper = LineWrapper {
        text: &text,
        max_chars,
        max_lines: max_lines.saturating_sub(1),
        lines_yielded: 0,
    };
    for (idx, line) in wrapper.enumerate() {
        let y = top + padding_y + line_height * (idx as i32 + 1);
        Text::new(line, Point::new(left + padding_x, y), body_style)
            .draw(ctx.display)
            .ok();
    }

    ctx.display.flush().map_err(Into::into)
}
