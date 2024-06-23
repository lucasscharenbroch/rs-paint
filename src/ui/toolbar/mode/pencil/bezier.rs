use super::PencilState;
use super::super::{Canvas, Toolbar};

pub struct BezierSegment {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl BezierSegment {
    fn from_grouped(
        (x0, y0): (usize, usize),
        (x1, y1): (usize, usize),
        (x2, y2): (usize, usize),
    ) -> Self {
        Self { x0, y0, x1, y1, x2, y2 }
    }

    /// Approximate of the curve's length
    pub fn rough_length(&self) -> f64 {
        let x0 = self.x0 as f64;
        let x1 = self.x1 as f64;
        let x2 = self.x2 as f64;
        let y0 = self.y0 as f64;
        let y1 = self.y1 as f64;
        let y2 = self.y2 as f64;

        // average the legs' sum and the hypotenuse
        let a = ((x0 - x1).powi(2) + (y0 - y1).powi(2)).sqrt();
        let b = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
        let c = ((x0 - x2).powi(2) + (y0 - y2).powi(2)).sqrt();

        a + b + c / 2.0
    }

    pub fn sample_n_pixels(&self, num_pix: usize) -> Vec<(usize, usize)> {
        (0..num_pix).map(|i| {
            let t = (i as f64 + 0.5) / num_pix as f64;
            let tn = 1.0 - t;
            let x0 = self.x0 as f64;
            let x1 = self.x1 as f64;
            let x2 = self.x2 as f64;
            let y0 = self.y0 as f64;
            let y1 = self.y1 as f64;
            let y2 = self.y2 as f64;

            let x = tn * (tn * x0 + t * x1) + t * (tn * x1 + t * x2);
            let y = tn * (tn * y0 + t * y1) + t * (tn * y1 + t * y2);
            (x, y)
        })
            // filter out the negatives, else they'll be converted to 0
            // (and stick to the side of the image)
            .filter(|(x, y)| *x > 0.0 && *y > 0.0)
            .map(|(x, y)| (x as usize, y as usize))
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Copy)]
pub enum IncrementalBezierSnapshot {
    NoPoints,
    One((usize, usize)),
    Two((usize, usize), (usize, usize)),
}

impl IncrementalBezierSnapshot {
    pub fn append_point(
        &mut self, pt: (usize, usize)
    ) -> Option<BezierSegment> {
        match self {
            Self::NoPoints => {
                *self = Self::One(pt);
                None
            },
            Self::One(last) => {
                *self = Self::Two(*last, pt);
                None
            },
            Self::Two(last_last, last) => {
                let res = Some(BezierSegment::from_grouped(*last_last, *last, pt));
                *self = Self::One(pt);
                res
            }
        }
    }
}
