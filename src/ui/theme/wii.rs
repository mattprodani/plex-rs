use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10},
        MonoTextStyleBuilder,
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::{Alignment, Text, TextStyleBuilder},
};

use crate::{
    core::app::{App, AppCtx, DisplayEntry},
    ui::boot_menu::BootMenu,
    ui::theme::LineWrapper,
    AppError,
};

// --- Wii Palette ---
const BG: Rgb888 = Rgb888::new(0xF0, 0xF0, 0xF0);
const WHITE: Rgb888 = Rgb888::new(0xFF, 0xFF, 0xFF);
const SHADOW: Rgb888 = Rgb888::new(0xD0, 0xD0, 0xD0);
const TEXT_DARK: Rgb888 = Rgb888::new(0x33, 0x33, 0x33);
const TEXT_LIGHT: Rgb888 = Rgb888::new(0x88, 0x88, 0x88);
const BLUE: Rgb888 = Rgb888::new(0x00, 0xAE, 0xEF);
const SELECTION_BG: Rgb888 = Rgb888::new(0xE0, 0xF4, 0xFC);
const BORDER: Rgb888 = Rgb888::new(0xCC, 0xCC, 0xCC);
const RED: Rgb888 = Rgb888::new(0xFF, 0x33, 0x33);

const LOGO: &[&str] = &[
    r"           _ _ ",
    r" __      _(_|_)",
    r" \ \ /\ / / | |",
    r"  \ V  V /| | |",
    r"   \_/\_/ |_|_|",
    r"               ",
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

    display.clear(BG);

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
        Point::new(box_x + 4, box_y + 4),
        Size::new(box_width_u32, box_height_u32),
    );
    RoundedRectangle::with_equal_corners(shadow_rect, Size::new(24, 24))
        .into_styled(PrimitiveStyle::with_fill(SHADOW))
        .draw(display)
        .ok();

    let window_style = PrimitiveStyleBuilder::new()
        .fill_color(WHITE)
        .stroke_color(BORDER)
        .stroke_width(2)
        .build();

    RoundedRectangle::with_equal_corners(modal_rect, Size::new(24, 24))
        .into_styled(window_style)
        .draw(display)
        .ok();
}

fn draw_window_controls<D>(display: &mut D, box_x: i32, box_y: i32, box_width: i32)
where
    D: DrawTarget<Color = Rgb888>,
{
    let title_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(TEXT_DARK)
        .build();

    let center_style = TextStyleBuilder::new().alignment(Alignment::Center).build();

    Text::with_text_style(
        "Wii Menu",
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
    .into_styled(PrimitiveStyle::with_stroke(BORDER, 2))
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
        .text_color(TEXT_DARK)
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
        .text_color(TEXT_LIGHT)
        .build();

    let quote = "be pragmatically non-pragmatic";

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
        .text_color(TEXT_LIGHT)
        .build();

    Text::new(
        "Channels",
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
        let item_rect = Rectangle::new(
            Point::new(panel_x, y),
            Size::new(
                u32::try_from(panel_width).unwrap_or(u32::MAX),
                u32::try_from(item_height).unwrap_or(u32::MAX),
            ),
        );

        let label = display_opts.label.as_str();
        let is_selected = i == menu.selected();

        if is_selected {
            RoundedRectangle::with_equal_corners(item_rect, Size::new(16, 16))
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(SELECTION_BG)
                        .stroke_color(BLUE)
                        .stroke_width(2)
                        .build(),
                )
                .draw(display)
                .ok();

            let sel_text_style = MonoTextStyleBuilder::new()
                .font(&FONT_10X20)
                .text_color(TEXT_DARK)
                .build();

            Text::new(label, Point::new(panel_x + 20, y + 26), sel_text_style)
                .draw(display)
                .ok();
        } else {
            RoundedRectangle::with_equal_corners(item_rect, Size::new(16, 16))
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(BG)
                        .stroke_color(BORDER)
                        .stroke_width(1)
                        .build(),
                )
                .draw(display)
                .ok();

            let unsel_text_style = MonoTextStyleBuilder::new()
                .font(&FONT_10X20)
                .text_color(TEXT_LIGHT)
                .build();

            Text::new(label, Point::new(panel_x + 20, y + 26), unsel_text_style)
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
        .text_color(TEXT_LIGHT)
        .build();

    let center_style = TextStyleBuilder::new().alignment(Alignment::Center).build();

    Text::with_text_style(
        "Use UP/DOWN to navigate, ENTER to start",
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
    RoundedRectangle::with_equal_corners(shadow_rect, Size::new(24, 24))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(SHADOW).build())
        .draw(display)
        .ok();

    let background = PrimitiveStyleBuilder::new()
        .fill_color(WHITE)
        .stroke_color(RED)
        .stroke_width(3)
        .build();
    let modal_rect = Rectangle::new(
        Point::new(left, top),
        Size::new(box_width_u32, box_height_u32),
    );
    RoundedRectangle::with_equal_corners(modal_rect, Size::new(24, 24))
        .into_styled(background)
        .draw(display)
        .ok();

    let title_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(RED)
        .build();
    let body_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(TEXT_DARK)
        .build();
    let center_style = TextStyleBuilder::new().alignment(Alignment::Center).build();

    Text::with_text_style(
        "System Error",
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
        .text_color(TEXT_LIGHT)
        .build();
    Text::with_text_style(
        "Press A (Enter) or B (Esc) to continue...",
        Point::new(left + box_w / 2, top + box_h - 15),
        footer_style,
        center_style,
    )
    .draw(display)
    .ok();

    display.flush().map_err(Into::into)
}
