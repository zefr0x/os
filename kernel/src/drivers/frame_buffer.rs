// Refrences:
//  https://github.com/commonkestrel/minesweeper-os/blob/02d675c2e3099b6bf14f16a32722da4bd15d6bcc/kernel/src/framebuffer.rs

use bootloader_api::info::{FrameBuffer, PixelFormat};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};

pub struct Display {
    framebuffer: &'static mut FrameBuffer,
}

impl Display {
    pub fn new(framebuffer: &'static mut FrameBuffer) -> Display {
        Self { framebuffer }
    }

    pub fn clear(&mut self) {
        self.framebuffer.buffer_mut().fill(0);
    }

    /// Caller must ensure that the x and y coordinates are both >= 0
    /// and less than the framebuffer width and height respectively.
    unsafe fn set_pixel(framebuffer: &mut FrameBuffer, x: i32, y: i32, r: u8, g: u8, b: u8) {
        let info = framebuffer.info();

        match info.pixel_format {
            PixelFormat::Rgb => {
                let index = info.stride * info.bytes_per_pixel * (y as usize)
                    + info.bytes_per_pixel * (x as usize)
                    + info.bytes_per_pixel
                    - 3;
                let buffer = framebuffer.buffer_mut();

                buffer[index] = r;
                buffer[index + 1] = g;
                buffer[index + 2] = b;
            }
            PixelFormat::Bgr => {
                let index = info.stride * info.bytes_per_pixel * (y as usize)
                    + info.bytes_per_pixel * (x as usize)
                    + info.bytes_per_pixel
                    - 3;
                let buffer = framebuffer.buffer_mut();

                buffer[index] = b;
                buffer[index + 1] = g;
                buffer[index + 2] = r;
            }
            PixelFormat::U8 => {
                let r16 = r as u16;
                let g16 = g as u16;
                let b16 = b as u16;
                let y = ((3 * r16 + b16 + 4 * g16) / 8) as u8;

                let index = info.stride * info.bytes_per_pixel * (y as usize)
                    + info.bytes_per_pixel * (x as usize)
                    + info.bytes_per_pixel
                    - 1;

                framebuffer.buffer_mut()[index] = y;
            }
            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => {
                let index = info.stride * info.bytes_per_pixel * (y as usize)
                    + info.bytes_per_pixel * (x as usize);
                let buffer = framebuffer.buffer_mut();

                buffer[index + red_position as usize] = r;
                buffer[index + green_position as usize] = g;
                buffer[index + blue_position as usize] = b;
            }
            _ => {}
        }
    }

    fn draw_pixel(&mut self, Pixel(point, color): Pixel<Rgb888>) {
        let size = self.size();

        if (0..(size.width as i32)).contains(&point.x)
            && (0..(size.height as i32)).contains(&point.y)
        {
            unsafe {
                Display::set_pixel(
                    self.framebuffer,
                    point.x,
                    point.y,
                    color.r(),
                    color.g(),
                    color.b(),
                )
            };
        }
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        let info = self.framebuffer.info();

        Size::new(info.width as u32, info.height as u32)
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels.into_iter() {
            self.draw_pixel(pixel);
        }

        Ok(())
    }
}
