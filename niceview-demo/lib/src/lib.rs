#![no_std]
#![no_main]
#![deny(missing_docs)]

pub mod color;

/// A display used on the keyboard.
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

    fn draw_pixel(&mut self, x: usize, y: usize, color: Color);

    async fn write(&mut self, data: &[u8]);
}
