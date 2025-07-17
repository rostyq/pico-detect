use image::GenericImageView;
use imageproc::rect::Rect;

/// Padding around a rectangular region.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Padding {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Padding {
    /// Create a new padding with specified values.
    #[inline]
    pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Set the top padding value.
    #[inline]
    pub fn top(self, value: i32) -> Self {
        Self { top: value, ..self }
    }

    /// Set the right padding value.
    #[inline]
    pub fn right(self, value: i32) -> Self {
        Self { right: value, ..self }
    }

    /// Set the bottom padding value.
    #[inline]
    pub fn bottom(self, value: i32) -> Self {
        Self { bottom: value, ..self }
    }

    /// Create padding with the same value for all sides.
    #[inline]
    pub fn all(value: i32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }

    /// Set vertical padding (top and bottom).
    #[inline]
    pub fn vertical(self, value: i32) -> Self {
        Self { top: value, bottom: value, ..self }
    }

    /// Set horizontal padding (left and right).
    #[inline]
    pub fn horizontal(self, value: i32) -> Self {
        Self { right: value, left: value, ..self }
    }

    /// Set the left padding value.
    #[inline]
    pub fn left(self, value: i32) -> Self {
        Self { left: value, ..self }
    }

    /// Convert padding to a rectangle.
    #[inline]
    pub fn rect(self, width: u32, height: u32) -> Rect {
        let w = (width as i32) - self.right - self.left;
        let h = (height as i32) - self.bottom - self.top;
        Rect::at(self.left, self.top).of_size(w as u32, h as u32)
    }

    /// Convert padding to a rectangle based on the image dimensions.
    #[inline]
    pub fn image_rect<I: GenericImageView>(self, image: &I) -> Rect {
        self.rect(image.width(), image.height())
    }
}