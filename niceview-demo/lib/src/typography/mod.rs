use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Dimensions, DrawTarget, Point, Size},
};

use crate::KeyboardDisplay;

/// Simple hacky font.
pub mod text_db;

/// Generated files.
#[doc(hidden)]
pub mod generated;

pub struct Wrapper<'a, T: ?Sized>(&'a mut T);

impl<'a, T> Wrapper<'a, T>
where
    T: KeyboardDisplay + ?Sized,
{
    pub fn new(v: &'a mut T) -> Self {
        Self(v)
    }
}

// TODO: Integer castings!
impl<'a, T: KeyboardDisplay + ?Sized> Dimensions for Wrapper<'a, T> {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        embedded_graphics::primitives::Rectangle::new(
            Point::zero(),
            Size::new(T::WIDTH as u32, T::HEIGHT as u32),
        )
    }
}

impl<'a, T: KeyboardDisplay + ?Sized> DrawTarget for Wrapper<'a, T> {
    type Color = BinaryColor;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for p in pixels {
            if p.1.is_on() {
                let Point { x, y } = p.0;
                self.0
                    .draw_pixel(x as usize, y as usize, crate::color::Color::Black);
            }
        }

        Ok(())
    }
}
