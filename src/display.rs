//! Module implementing embedded_graphics_core::DrawTarget
//! for the Rust UEFI crate. It is actually just a buffer,
//! and the push to uefi is done via blit during flush().
use alloc::vec;
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};

pub struct GopDisplay {
    width: usize,
    height: usize,
    buffer: Vec<BltPixel>,
}

impl GopDisplay {
    /// Create a new `GopDisplay` matching the current GOP mode resolution.
    pub fn new(gop: &GraphicsOutput) -> Self {
        let (width, height) = gop.current_mode_info().resolution();
        Self {
            width,
            height,
            buffer: vec![BltPixel::new(0, 0, 0); width * height],
        }
    }

    /// Blit the entire buffer to the framebuffer.
    pub fn flush(&self, gop: &mut GraphicsOutput) -> Result<(), uefi::Error> {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.buffer,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }

    /// Clear the buffer to a specific color.
    pub fn clear(&mut self, color: Rgb888) {
        let pixel = BltPixel::new(color.r(), color.g(), color.b());
        self.buffer.fill(pixel);
    }
}

impl DrawTarget for GopDisplay {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Bounds check - discard out of bounds pixels per DrawTarget requirements
            let (x, y) = match (coord.x, coord.y) {
                (x, y) if x >= 0 && y >= 0 => (x as usize, y as usize),
                _ => continue,
            };

            if x < self.width && y < self.height {
                let index = y * self.width + x;
                let pixel = &mut self.buffer[index];
                pixel.red = color.r();
                pixel.green = color.g();
                pixel.blue = color.b();
            }
        }
        Ok(())
    }
}

impl OriginDimensions for GopDisplay {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}
