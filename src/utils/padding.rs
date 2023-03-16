use imageproc::rect::Rect;

#[derive(Copy, Clone, Debug, Default)]
pub struct Padding {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Padding {
    pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[inline]
    pub fn rect(&self, width: u32, height: u32) -> Rect {
        let w = (width as i32) - self.right;
        let h = (height as i32) - self.bottom;
        Rect::at(self.left, self.top).of_size(w as u32, h as u32)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PaddingBuilder {
    pub top: Option<i32>,
    pub right: Option<i32>,
    pub bottom: Option<i32>,
    pub left: Option<i32>,
}

impl PaddingBuilder {
    pub fn with_all(mut self, value: i32) -> Self {
        self.top = Some(value);
        self.right = Some(value);
        self.bottom = Some(value);
        self.left = Some(value);
        self
    }

    pub fn with_vertical(mut self, value: i32) -> Self {
        self.top = Some(value);
        self.bottom = Some(value);
        self
    }

    pub fn with_horizontal(mut self, value: i32) -> Self {
        self.right = Some(value);
        self.left = Some(value);
        self
    }

    pub fn with_top(mut self, value: i32) -> Self {
        self.top = Some(value);
        self
    }

    pub fn with_right(mut self, value: i32) -> Self {
        self.right = Some(value);
        self
    }

    pub fn with_bottom(mut self, value: i32) -> Self {
        self.bottom = Some(value);
        self
    }

    pub fn with_left(mut self, value: i32) -> Self {
        self.left = Some(value);
        self
    }

    pub fn build(self) -> Padding {
        Padding {
            top: self.top.unwrap_or_default(),
            right: self.right.unwrap_or_default(),
            bottom: self.bottom.unwrap_or_default(),
            left: self.left.unwrap_or_default(),
        }
    }
}
