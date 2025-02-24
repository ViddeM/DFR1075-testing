#![no_std]
#![no_main]
#![deny(missing_docs)]

//! Lib for display.

use color::Color;
use typography::text_db::{CHAR_HEIGHT, CHAR_WIDTH, is_pixel_for_char};

/// Colors.
pub mod color;

/// Text rendering.
mod typography;

/// A display used on the keyboard.
#[allow(async_fn_in_trait)]
pub trait KeyboardDisplay {
    /// Display width in pixels.
    const WIDTH: usize = 160;

    /// Display height in pixels.
    const HEIGHT: usize = 68;

    /// Send a clear command to the screen.
    /// Effectively equivalent to [Self::fill_white] + [Self::flush] but faster.
    async fn clear_display(&mut self);

    /// Flush the current write buffer to the device.
    async fn flush(&mut self);

    /// Fill the entire screen with white.
    fn fill_white(&mut self);

    /// Set the color for a particular pixel.
    fn draw_pixel(&mut self, x: usize, y: usize, color: Color);

    /// Write data to the display.
    async fn write(&mut self, data: &[u8]);

    /// Draw a circle.
    fn draw_circle(&mut self, center_x: usize, center_y: usize, color: Color, radius: usize) {
        let center_x = center_x as isize;
        let center_y = center_y as isize;

        for x in 0..Self::WIDTH {
            for y in 0..Self::HEIGHT {
                let distance = {
                    let (x, y) = (x as isize, y as isize);
                    (center_x - x).abs() + (center_y - y).abs()
                };

                if distance <= radius as isize {
                    self.draw_pixel(x, y, color);
                }
            }
        }
    }

    /// TODO:
    fn draw_text(&mut self, text: &str, start_x: usize, start_y: usize) {
        use eg_bdf::BdfTextStyle;
        use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, text::Text};
        use typography::{Wrapper, generated::tamzen_10x20r::tamzen_10x20r};

        let text_style = BdfTextStyle::new(&tamzen_10x20r, BinaryColor::On);
        let mut drawable_display = Wrapper::new(self);

        let t = Text::new("some text", Point::new(20, 40), text_style).draw(&mut drawable_display);
    }

    // /// Draw a character starting at the specified coordinates (from top-left of screen).
    // fn draw_char_at(&mut self, char: char, x: usize, y: usize) {
    //     for off_y in 0..CHAR_HEIGHT {
    //         for off_x in 0..CHAR_WIDTH {
    //             let color = if is_pixel_for_char(char, off_x, off_y) {
    //                 log::info!("{off_x}x{off_y} is black!");
    //                 Color::Black
    //             } else {
    //                 log::info!("is white");
    //                 Color::White
    //             };
    //             let y = y + off_y;
    //             let x = x + off_x;
    //             self.draw_pixel(x, y, color);
    //         }
    //     }
    // }
}
