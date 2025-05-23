//! An icon that we can draw.
//!
//! IMAGE_WIDTH = The actual pixel width of the image.
//! BYTE_WIDTH = The number of bytes wide the image is, the following should always hold:
//! ```
//! BYTE_WIDTH = (IMAGE_WIDTH + (8 - (IMAGE_WIDTH % 8))) / 8
//! ```
//!
//! HEIGHT = the height of the image in pixels.
pub struct Image<const IMAGE_WIDTH: usize, const BYTE_WIDTH: usize, const HEIGHT: usize> {
    pub rows: [[u8; BYTE_WIDTH]; HEIGHT],
}
