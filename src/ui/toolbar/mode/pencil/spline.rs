pub trait SplineSegment {
    /// Approximate of the curve's length
    fn rough_length(&self) -> f64;
    fn sample_n_pixels(&self, num_pix: usize) -> Vec<(i32, i32)>;
    fn endpoint(&self) -> (f64, f64);
}

pub struct SplineSegment3 {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl SplineSegment3 {
    pub fn from_grouped(
        (x0, y0): (i32, i32),
        (x1, y1): (i32, i32),
        (x2, y2): (i32, i32),
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

    fn sample_n_pixels(&self, num_pix: usize) -> Vec<(i32, i32)> {
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
            .map(|(x, y)| (x.round() as i32, y.round() as i32))
            .collect::<Vec<_>>()
    }

    fn endpoint(&self) -> (f64, f64) {
        (self.x2 as f64, self.y2 as f64)
    }
}

pub struct SplineSegment4 {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    x3: i32,
    y3: i32,
}

impl SplineSegment4 {
    fn from_grouped(
        (x0, y0): (i32, i32),
        (x1, y1): (i32, i32),
        (x2, y2): (i32, i32),
        (x3, y3): (i32, i32),
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

    fn sample_n_pixels(&self, num_pix: usize) -> Vec<(i32, i32)> {
        (0..num_pix).map(|i| {
            let t = (i as f64 + 0.5) / num_pix as f64;
            let t2 = t.powi(2);
            let t3 = t.powi(3);
            let x0 = self.x0 as f64;
            let x1 = self.x1 as f64;
            let x2 = self.x2 as f64;
            let x3 = self.x3 as f64;
            let y0 = self.y0 as f64;
            let y1 = self.y1 as f64;
            let y2 = self.y2 as f64;
            let y3 = self.y3 as f64;

            // cubic b-spline
            let x = 1.0/6.0 * ((-x0 + 3.0 * x1 - 3.0 * x2 + x3) * t3 +
                                    (3.0 * x0 - 6.0 * x1 + 3.0 * x2) * t2 +
                                    (-3.0 * x0 + 3.0 * x2) * t +
                                    (x0 + 4.0 * x1 + x2));
            let y = 1.0/6.0 * ((-y0 + 3.0 * y1 - 3.0 * y2 + y3) * t3 +
                                    (3.0 * y0 - 6.0 * y1 + 3.0 * y2) * t2 +
                                    (-3.0 * y0 + 3.0 * y2) * t +
                                    (y0 + 4.0 * y1 + y2));
            (x, y)
        })
            .map(|(x, y)| (x.round() as i32, y.round() as i32))
            .collect::<Vec<_>>()
    }

    fn endpoint(&self) -> (f64, f64) {
        (self.x3 as f64, self.y3 as f64)
    }
}

#[derive(Clone, Copy)]
pub enum IncrementalSplineSnapshot {
    NoPoints,
    One((i32, i32)),
    Two((i32, i32), (i32, i32)),
    Three((i32, i32), (i32, i32), (i32, i32)),
}

impl IncrementalSplineSnapshot {
    pub fn append_point(
        &mut self, pt: (i32, i32)
    ) -> Option<SplineSegment4> {
        match self {
            Self::NoPoints => {
                // B-Spline: replicate the first point
                *self = Self::Three(pt, pt, pt);
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
                *self = Self::Three(*last_last, *last, pt);
                res
            }
        }
    }
}
