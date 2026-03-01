use crate::{
    AppError,
    core::app::{App, AppCtx, DisplayEntry},
    ui::boot_menu::BootMenu,
};
use serde::Deserialize;

pub mod default;

#[cfg(feature = "mocha")]
pub mod mocha;

#[cfg(feature = "wii")]
pub mod wii;

/// A UI Theme that determines how the boot menu and overlays are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Default,
    #[cfg(feature = "mocha")]
    #[serde(alias = "catppuccin", alias = "rice")]
    Mocha,
    #[cfg(feature = "wii")]
    #[serde(alias = "nintendo")]
    Wii,
}

impl Theme {
    /// Draw the boot menu.
    pub fn draw_boot_menu<'a, T: App + DisplayEntry>(
        &self,
        ctx: &mut AppCtx,
        menu: &BootMenu<'a, T>,
    ) -> Result<(), AppError> {
        match self {
            Theme::Default => default::draw_boot_menu(ctx, menu),
            #[cfg(feature = "mocha")]
            Theme::Mocha => mocha::draw_boot_menu(ctx, menu),
            #[cfg(feature = "wii")]
            Theme::Wii => wii::draw_boot_menu(ctx, menu),
        }
    }

    /// Draw an error overlay.
    pub fn draw_error_overlay(&self, ctx: &mut AppCtx, error: &AppError) -> Result<(), AppError> {
        match self {
            Theme::Default => default::draw_error_overlay(ctx, error),
            #[cfg(feature = "mocha")]
            Theme::Mocha => mocha::draw_error_overlay(ctx, error),
            #[cfg(feature = "wii")]
            Theme::Wii => wii::draw_error_overlay(ctx, error),
        }
    }
}

pub(crate) struct LineWrapper<'a> {
    pub text: &'a str,
    pub max_chars: usize,
    pub max_lines: usize,
    pub lines_yielded: usize,
}

impl<'a> Iterator for LineWrapper<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lines_yielded >= self.max_lines || self.text.is_empty() {
            return None;
        }

        let mut byte_idx = 0;
        let mut char_count = 0;
        let mut last_space = None;
        let mut newline_idx = None;

        for (i, c) in self.text.char_indices() {
            if c == '\n' {
                newline_idx = Some(i);
                break;
            }
            if char_count == self.max_chars {
                break;
            }
            if c.is_whitespace() {
                last_space = Some(i);
            }
            byte_idx = i + c.len_utf8();
            char_count += 1;
        }

        if let Some(nl) = newline_idx {
            let res = &self.text[..nl];
            self.text = &self.text[nl + 1..];
            self.lines_yielded += 1;
            return Some(res);
        }

        if byte_idx == self.text.len() {
            let res = self.text;
            self.text = "";
            self.lines_yielded += 1;
            return Some(res);
        }

        if let Some(space) = last_space {
            if space == 0 {
                self.text = self.text[1..].trim_start();
                return self.next();
            }
            let res = &self.text[..space];
            self.text = self.text[space + 1..].trim_start();
            self.lines_yielded += 1;
            Some(res)
        } else {
            let res = &self.text[..byte_idx];
            self.text = &self.text[byte_idx..];
            self.lines_yielded += 1;
            Some(res)
        }
    }
}
