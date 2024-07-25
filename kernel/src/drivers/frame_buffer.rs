// Refrences:
//  https://github.com/commonkestrel/minesweeper-os/blob/02d675c2e3099b6bf14f16a32722da4bd15d6bcc/kernel/src/framebuffer.rs

use bootloader_api::info::{FrameBuffer, PixelFormat};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};
use spin::{once::Once, Mutex};

pub static DISPLAY: Once<Mutex<Display>> = Once::new();

pub struct Display {
    framebuffer: &'static mut FrameBuffer,
}

impl Display {
    pub fn new(framebuffer: &'static mut FrameBuffer) -> Self {
        Self { framebuffer }
    }

    pub fn fill0(&mut self) {
        self.framebuffer.buffer_mut().fill(0);
    }

    /// Caller must ensure that the x and y coordinates are both >= 0
    /// and less than the framebuffer width and height respectively.
    #[expect(clippy::many_single_char_names)]
    fn set_pixel(framebuffer: &mut FrameBuffer, x: i32, y: i32, r: u8, g: u8, b: u8) {
        let info = framebuffer.info();

        match info.pixel_format {
            PixelFormat::Rgb => {
                #[expect(clippy::unwrap_used)]
                let index = info.stride * info.bytes_per_pixel * (usize::try_from(y).unwrap())
                    + info.bytes_per_pixel * (usize::try_from(x).unwrap())
                    + info.bytes_per_pixel
                    - 3;
                let buffer = framebuffer.buffer_mut();

                buffer[index] = r;
                buffer[index + 1] = g;
                buffer[index + 2] = b;
            }
            PixelFormat::Bgr => {
                #[expect(clippy::unwrap_used)]
                let index = info.stride * info.bytes_per_pixel * (usize::try_from(y).unwrap())
                    + info.bytes_per_pixel * (usize::try_from(x).unwrap())
                    + info.bytes_per_pixel
                    - 3;
                let buffer = framebuffer.buffer_mut();

                buffer[index] = b;
                buffer[index + 1] = g;
                buffer[index + 2] = r;
            }
            PixelFormat::U8 => {
                #[expect(clippy::unwrap_used)]
                #[expect(clippy::integer_division)]
                let pixel =
                    u8::try_from((3 * u16::from(r) + u16::from(b) + 4 * u16::from(g)) / 8).unwrap();

                #[expect(clippy::unwrap_used)]
                let index = info.stride * info.bytes_per_pixel * (usize::try_from(y).unwrap())
                    + info.bytes_per_pixel * (usize::try_from(x).unwrap())
                    + info.bytes_per_pixel
                    - 1;

                framebuffer.buffer_mut()[index] = pixel;
            }
            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => {
                #[expect(clippy::unwrap_used)]
                let index = info.stride * info.bytes_per_pixel * (usize::try_from(y).unwrap())
                    + info.bytes_per_pixel * (usize::try_from(x).unwrap());
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

        #[expect(clippy::unwrap_used)]
        if (0_i32..(i32::try_from(size.width).unwrap())).contains(&point.x)
            && (0_i32..(i32::try_from(size.height).unwrap())).contains(&point.y)
        {
            Self::set_pixel(
                self.framebuffer,
                point.x,
                point.y,
                color.r(),
                color.g(),
                color.b(),
            );
        }
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        let info = self.framebuffer.info();

        Size::new(
            #[expect(clippy::unwrap_used)]
            u32::try_from(info.width).unwrap(),
            #[expect(clippy::unwrap_used)]
            u32::try_from(info.height).unwrap(),
        )
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_pixel(pixel);
        }

        Ok(())
    }
}
