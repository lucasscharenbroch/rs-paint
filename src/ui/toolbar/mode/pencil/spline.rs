use super::PencilState;
use super::super::{Canvas, Toolbar};

pub trait SplineSegment {
    /// Approximate of the curve's length
    fn rough_length(&self) -> f64;
    fn sample_n_pixels(&self, num_pix: usize) -> Vec<(usize, usize)>;
    fn endpoint(&self) -> (f64, f64);
}

pub struct SplineSegment3 {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl SplineSegment3 {
    pub fn from_grouped(
        (x0, y0): (usize, usize),
        (x1, y1): (usize, usize),
        (x2, y2): (usize, usize),
    ) -> Self {
        Self { x0, y0, x1, y1, x2, y2 }
    }
}

impl SplineSegment for SplineSegment3 {
    fn rough_length(&self) -> f64 {
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

    fn sample_n_pixels(&self, num_pix: usize) -> Vec<(usize, usize)> {
        (0..num_pix).map(|i| {
            let t = (i as f64 + 0.5) / num_pix as f64;
            let tn = 1.0 - t;
            let x0 = self.x0 as f64;
            let x1 = self.x1 as f64;
            let x2 = self.x2 as f64;
            let y0 = self.y0 as f64;
            let y1 = self.y1 as f64;
            let y2 = self.y2 as f64;

            // quadratic bezier
            let x = tn * (tn * x0 + t * x1) + t * (tn * x1 + t * x2);
            let y = tn * (tn * y0 + t * y1) + t * (tn * y1 + t * y2);
            (x, y)
        })
            // filter out the negatives, else they'll be converted to 0
            // (and stick to the side of the image)
            .filter(|(x, y)| *x > 0.0 && *y > 0.0)
            .map(|(x, y)| (x.round() as usize, y.round() as usize))
            .collect::<Vec<_>>()
    }

    fn endpoint(&self) -> (f64, f64) {
        (self.x2 as f64, self.y2 as f64)
    }
}

pub struct SplineSegment4 {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    x3: usize,
    y3: usize,
}

impl SplineSegment4 {
    fn from_grouped(
        (x0, y0): (usize, usize),
        (x1, y1): (usize, usize),
        (x2, y2): (usize, usize),
        (x3, y3): (usize, usize),
    ) -> Self {
        Self { x0, y0, x1, y1, x2, y2, x3, y3 }
    }
}

impl SplineSegment for SplineSegment4 {
    fn rough_length(&self) -> f64 {
        let x0 = self.x0 as f64;
        let x1 = self.x1 as f64;
        let x2 = self.x2 as f64;
        let x3 = self.x3 as f64;
        let y0 = self.y0 as f64;
        let y1 = self.y1 as f64;
        let y2 = self.y2 as f64;
        let y3 = self.y3 as f64;

        let a = ((x0 - x1).powi(2) + (y0 - y1).powi(2)).sqrt();
        let b = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
        let c = ((x2 - x3).powi(2) + (y2 - y3).powi(2)).sqrt();
        let d = ((x0 - x3).powi(2) + (y0 - y3).powi(2)).sqrt();

        a + b + c + d / 2.0
    }

    fn sample_n_pixels(&self, num_pix: usize) -> Vec<(usize, usize)> {
        (0..num_pix).map(|i| {
            let t = (i as f64 + 0.5) / num_pix as f64;
            let t2 = t.powi(2);
            let t3 = t.powi(3);
            let tn = 1.0 - t;
            let tn2 = tn.powi(2);
            let tn3 = tn.powi(3);
            let x0 = self.x0 as f64;
            let x1 = self.x1 as f64;
            let x2 = self.x2 as f64;
            let x3 = self.x3 as f64;
            let y0 = self.y0 as f64;
            let y1 = self.y1 as f64;
            let y2 = self.y2 as f64;
            let y3 = self.y3 as f64;

            // cubic bezier
            let x = tn3 * x0 + 3.0 * tn2 * t * x1 + 3.0 * tn * t2 * x2 + t3 * x3;
            let y = tn3 * y0 + 3.0 * tn2 * t * y1 + 3.0 * tn * t2 * y2 + t3 * y3;
            (x, y)
        })
            // filter out the negatives, else they'll be converted to 0
            // (and stick to the side of the image)
            .filter(|(x, y)| *x > 0.0 && *y > 0.0)
            .map(|(x, y)| (x.round() as usize, y.round() as usize))
            .collect::<Vec<_>>()
    }

    fn endpoint(&self) -> (f64, f64) {
        (self.x3 as f64, self.y3 as f64)
    }
}

#[derive(Clone, Copy)]
pub enum IncrementalSplineSnapshot {
    NoPoints,
    One((usize, usize)),
    Two((usize, usize), (usize, usize)),
    Three((usize, usize), (usize, usize), (usize, usize)),
}

impl IncrementalSplineSnapshot {
    pub fn append_point(
        &mut self, pt: (usize, usize)
    ) -> Option<SplineSegment4> {
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
                *self = Self::Three(*last_last, *last, pt);
                None
            },
            Self::Three(last_last_last, last_last, last) => {
                let res = Some(SplineSegment4::from_grouped(
                    *last_last_last, *last_last, *last, pt
                ));
                *self = Self::One(pt);
                res
            }
        }
    }
}
