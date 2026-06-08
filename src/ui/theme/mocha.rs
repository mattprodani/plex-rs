use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10},
        MonoTextStyleBuilder,
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{
        Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
    },
    text::{Alignment, Text, TextStyleBuilder},
};

use crate::{
    core::app::{App, AppCtx, DisplayEntry},
    ui::boot_menu::BootMenu,
    ui::theme::LineWrapper,
    AppError,
};

// --- Catppuccin Mocha Palette ---
const CRUST: Rgb888 = Rgb888::new(0x11, 0x11, 0x1B);
const MANTLE: Rgb888 = Rgb888::new(0x18, 0x18, 0x25);
const BASE: Rgb888 = Rgb888::new(0x1E, 0x1E, 0x2E);
const SURFACE0: Rgb888 = Rgb888::new(0x31, 0x32, 0x44);
const SURFACE1: Rgb888 = Rgb888::new(0x45, 0x47, 0x5A);
const SURFACE2: Rgb888 = Rgb888::new(0x58, 0x5B, 0x70);
const OVERLAY0: Rgb888 = Rgb888::new(0x6C, 0x70, 0x86);
const TEXT: Rgb888 = Rgb888::new(0xCD, 0xD6, 0xF4);
const SUBTEXT0: Rgb888 = Rgb888::new(0xA6, 0xAD, 0xC8);

const BLUE: Rgb888 = Rgb888::new(0x89, 0xB4, 0xFA);
const MAUVE: Rgb888 = Rgb888::new(0xCB, 0xA6, 0xF7);
const GREEN: Rgb888 = Rgb888::new(0xA6, 0xE3, 0xA1);
const RED: Rgb888 = Rgb888::new(0xF3, 0x8B, 0xA8);

const LOGO: &[&str] = &[
    r"    ___ _             ",
    r"   / _ \ | _____  __  ",
    r"  / /_)/ |/ _ \ \/ /  ",
    r" / ___/| |  __/>  <   ",
    r" \/    |_|\___/_/\_\  ",
    r"                      ",
    r"  ____                ",
    r" | __ )  ___   ___ | |_",
    r" |  _ \ / _ \ / _ \| __|",
    r" | |_) | (_) | (_) | |_ ",
    r" |____/ \___/ \___/ \__|",
];

/// # Errors
/// Returns any drawing error from the underlying display.
pub fn draw_boot_menu<T: App + DisplayEntry>(
    ctx: &mut AppCtx,
    menu: &BootMenu<'_, T>,
) -> Result<(), AppError> {
    let display = &mut *ctx.display;
    let size = display.size();
    let w = i32::try_from(size.width).unwrap_or(i32::MAX);
    let h = i32::try_from(size.height).unwrap_or(i32::MAX);

    display.clear(CRUST);

    let box_width = 800.min((w - 60).max(400));
    let box_height = 500.min((h - 60).max(300));
    let box_x = (w - box_width) / 2;
    let box_y = (h - box_height) / 2;

    draw_modal_background(display, box_x, box_y, box_width, box_height);
    draw_window_controls(display, box_x, box_y, box_width);

    let show_logo = box_width >= 650;
    let left_panel_width = if show_logo { 300 } else { 0 };

    if show_logo {
        draw_logo_panel(display, box_x, box_y);
    }

    let right_panel_x = if show_logo {
        box_x + left_panel_width + 20
    } else {
        box_x + 40
    };
    let right_panel_width = box_width - left_panel_width - (if show_logo { 40 } else { 80 });

    draw_boot_entries(display, menu, right_panel_x, box_y + 60, right_panel_width);
    draw_footer(display, box_x, box_y, box_width, box_height);

    display.flush()?;
    Ok(())
}

fn draw_modal_background<D>(
    display: &mut D,
    box_x: i32,
    box_y: i32,
    box_width: i32,
    box_height: i32,
) where
    D: DrawTarget<Color = Rgb888>,
{
    let box_width_u32 = u32::try_from(box_width).unwrap_or(u32::MAX);
    let box_height_u32 = u32::try_from(box_height).unwrap_or(u32::MAX);
    let modal_rect = Rectangle::new(
        Point::new(box_x, box_y),
        Size::new(box_width_u32, box_height_u32),
    );

    let shadow_rect = Rectangle::new(
        Point::new(box_x + 8, box_y + 8),
        Size::new(box_width_u32, box_height_u32),
    );
    RoundedRectangle::with_equal_corners(shadow_rect, Size::new(12, 12))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(5, 5, 8)))
        .draw(display)
        .ok();

    let window_style = PrimitiveStyleBuilder::new()
        .fill_color(BASE)
        .stroke_color(SURFACE1)
        .stroke_width(2)
        .build();

    RoundedRectangle::with_equal_corners(modal_rect, Size::new(12, 12))
        .into_styled(window_style)
        .draw(display)
        .ok();
}

fn draw_window_controls<D>(display: &mut D, box_x: i32, box_y: i32, box_width: i32)
where
    D: DrawTarget<Color = Rgb888>,
{
    let dot_y = box_y + 20;
    let dot_spacing = 15;
    let mut dot_x = box_x + 20;

    Circle::new(Point::new(dot_x, dot_y - 6), 12)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0xF3, 0x8B, 0xA8)))
        .draw(display)
        .ok();
    dot_x += dot_spacing;
    Circle::new(Point::new(dot_x, dot_y - 6), 12)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0xF9, 0xE2, 0xAF)))
        .draw(display)
        .ok();
    dot_x += dot_spacing;
    Circle::new(Point::new(dot_x, dot_y - 6), 12)
        .into_styled(PrimitiveStyle::with_fill(GREEN))
        .draw(display)
        .ok();

    let title_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(TEXT)
        .build();

    let center_style = TextStyleBuilder::new().alignment(Alignment::Center).build();

    Text::with_text_style(
        "plex ~ boot",
        Point::new(box_x + box_width / 2, box_y + 26),
        title_style,
        center_style,
    )
    .draw(display)
    .ok();

    Line::new(
        Point::new(box_x, box_y + 40),
        Point::new(box_x + box_width, box_y + 40),
    )
    .into_styled(PrimitiveStyle::with_stroke(MANTLE, 2))
    .draw(display)
    .ok();
}

fn draw_logo_panel<D>(display: &mut D, box_x: i32, box_y: i32)
where
    D: DrawTarget<Color = Rgb888>,
{
    let logo_x = box_x + 30;
    let logo_y = box_y + 80;

    let logo_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(MAUVE)
        .build();

    for (i, line) in LOGO.iter().enumerate() {
        let idx = i32::try_from(i).unwrap_or(i32::MAX);
        Text::new(line, Point::new(logo_x, logo_y + idx * 20), logo_style)
            .draw(display)
            .ok();
    }

    let logo_lines = i32::try_from(LOGO.len()).unwrap_or(i32::MAX);
    let quote_y = logo_y + logo_lines * 20 + 50;
    let quote_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(SUBTEXT0)
        .build();

    let quote = "matthew says hi";

    Text::new(quote, Point::new(logo_x + 10, quote_y), quote_style)
        .draw(display)
        .ok();
}

fn draw_boot_entries<D, T>(
    display: &mut D,
    menu: &BootMenu<'_, T>,
    panel_x: i32,
    panel_y: i32,
    panel_width: i32,
) where
    D: DrawTarget<Color = Rgb888>,
    T: App + DisplayEntry,
{
    let list_header_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(SUBTEXT0)
        .build();

    Text::new(
        "Select Boot Entry:",
        Point::new(panel_x, panel_y + 20),
        list_header_style,
    )
    .draw(display)
    .ok();

    let item_start_y = panel_y + 50;
    let item_height = 40;

    for (i, target) in menu.targets().iter().enumerate() {
        let display_opts = target.display_options();
        let y = item_start_y + i32::try_from(i).unwrap_or(i32::MAX) * item_height;
        let panel_width_u32 = u32::try_from(panel_width).unwrap_or(u32::MAX);
        let item_height_u32 = u32::try_from(item_height).unwrap_or(u32::MAX);
        let item_rect = Rectangle::new(
            Point::new(panel_x, y),
            Size::new(panel_width_u32, item_height_u32),
        );

        let label = display_opts.label.as_str();
        let is_selected = i == menu.selected();

        if is_selected {
            RoundedRectangle::with_equal_corners(item_rect, Size::new(8, 8))
                .into_styled(PrimitiveStyle::with_fill(SURFACE0))
                .draw(display)
                .ok();

            let indicator_height = item_height_u32.saturating_sub(16);
            let indicator =
                Rectangle::new(Point::new(panel_x, y + 8), Size::new(4, indicator_height));
            RoundedRectangle::with_equal_corners(indicator, Size::new(2, 2))
                .into_styled(PrimitiveStyle::with_fill(BLUE))
                .draw(display)
                .ok();

            let sel_text_style = MonoTextStyleBuilder::new()
                .font(&FONT_10X20)
                .text_color(TEXT)
                .build();
            let arrow_style = MonoTextStyleBuilder::new()
                .font(&FONT_10X20)
                .text_color(BLUE)
                .build();

            Text::new(">", Point::new(panel_x + 15, y + 26), arrow_style)
                .draw(display)
                .ok();
            Text::new(label, Point::new(panel_x + 35, y + 26), sel_text_style)
                .draw(display)
                .ok();
        } else {
            let unsel_text_style = MonoTextStyleBuilder::new()
                .font(&FONT_10X20)
                .text_color(SURFACE2)
                .build();

            Text::new(label, Point::new(panel_x + 35, y + 26), unsel_text_style)
                .draw(display)
                .ok();
        }
    }
}

fn draw_footer<D>(display: &mut D, box_x: i32, box_y: i32, box_width: i32, box_height: i32)
where
    D: DrawTarget<Color = Rgb888>,
{
    let footer_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(OVERLAY0)
        .build();

    let center_style = TextStyleBuilder::new().alignment(Alignment::Center).build();

    Text::with_text_style(
        "Use UP/DOWN to navigate, ENTER to boot",
        Point::new(box_x + box_width / 2, box_y + box_height - 15),
        footer_style,
        center_style,
    )
    .draw(display)
    .ok();
}

/// # Errors
/// Returns any drawing error from the underlying display.
pub fn draw_error_overlay(ctx: &mut AppCtx, error: &AppError) -> Result<(), AppError> {
    let text = alloc::format!("{error}");

    let display = &mut *ctx.display;
    let size = display.size();
    let screen_w = i32::try_from(size.width).unwrap_or(i32::MAX);
    let screen_h = i32::try_from(size.height).unwrap_or(i32::MAX);
    let box_w = (screen_w * 2 / 3).max(400);
    let box_h = (screen_h / 3).max(200);
    let left = (screen_w - box_w) / 2;
    let top = (screen_h - box_h) / 2;

    let box_width_u32 = u32::try_from(box_w).unwrap_or(u32::MAX);
    let box_height_u32 = u32::try_from(box_h).unwrap_or(u32::MAX);
    let shadow_rect = Rectangle::new(
        Point::new(left + 8, top + 8),
        Size::new(box_width_u32, box_height_u32),
    );
    RoundedRectangle::with_equal_corners(shadow_rect, Size::new(12, 12))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(CRUST).build())
        .draw(display)
        .ok();

    let background = PrimitiveStyleBuilder::new()
        .fill_color(BASE)
        .stroke_color(RED)
        .stroke_width(2)
        .build();
    let modal_rect = Rectangle::new(
        Point::new(left, top),
        Size::new(box_width_u32, box_height_u32),
    );
    RoundedRectangle::with_equal_corners(modal_rect, Size::new(12, 12))
        .into_styled(background)
        .draw(display)
        .ok();

    let title_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(RED)
        .build();
    let body_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(TEXT)
        .build();
    let center_style = TextStyleBuilder::new().alignment(Alignment::Center).build();

    Text::with_text_style(
        "Oops, something went wrong!",
        Point::new(left + box_w / 2, top + 30),
        title_style,
        center_style,
    )
    .draw(display)
    .ok();

    let padding_x = 20;
    let padding_y = 60;
    let line_height = 20;
    let max_chars = usize::try_from(((box_w - padding_x * 2) / 10).max(1)).unwrap_or(usize::MAX);
    let max_lines =
        usize::try_from(((box_h - padding_y * 2) / line_height).max(1)).unwrap_or(usize::MAX);

    let wrapper = LineWrapper {
        text: &text,
        max_chars,
        max_lines: max_lines.saturating_sub(1),
        lines_yielded: 0,
    };

    for (idx, line) in wrapper.enumerate() {
        let y = top + padding_y + line_height * i32::try_from(idx).unwrap_or(i32::MAX);
        Text::new(line, Point::new(left + padding_x, y), body_style)
            .draw(display)
            .ok();
    }

    let footer_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(SURFACE1)
        .build();
    Text::with_text_style(
        "Press Enter or Esc to continue...",
        Point::new(left + box_w / 2, top + box_h - 15),
        footer_style,
        center_style,
    )
    .draw(display)
    .ok();

    display.flush().map_err(Into::into)
}
