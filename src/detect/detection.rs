use crate::traits::Region;

/// Object detection data.
#[derive(Debug, Copy, Clone)]
pub struct Detection<R: Region> {
    pub(crate) region: R,
    pub(crate) score: f32,
}

impl<R: Region> Detection<R> {
    #[inline]
    pub fn new(region: R, score: f32) -> Self {
        assert!(score > 0.0);
        Self { region, score }
    }

    /// Detection score.
    #[inline]
    pub fn score(&self) -> f32 {
        self.score
    }

    /// Detection rectangle.
    #[inline]
    pub fn region(&self) -> &R {
        &self.region
    }
}

impl<R: Region> AsRef<R> for Detection<R> {
    #[inline]
    fn as_ref(&self) -> &R {
        &self.region
    }
}

impl<R: Region> PartialEq for Detection<R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl<R: Region> PartialOrd for Detection<R> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}
