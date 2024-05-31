use super::Pixel;

// defines a way to average two pixels
#[derive(Clone, Copy)]
pub enum BlendingMode {
    Overwrite,
    Average,
    Paint,
}

impl BlendingMode {
    pub fn blend(&self, above: &Pixel, below: &Pixel) -> Pixel {
        match self {
            BlendingMode::Overwrite => above.clone(),
            BlendingMode::Average => {
                Pixel::from_rgba((above.r as f64 * 0.5 + below.r as f64 * 0.5) as u8,
                                 (above.g as f64 * 0.5 + below.g as f64 * 0.5) as u8,
                                 (above.b as f64 * 0.5 + below.b as f64 * 0.5) as u8,
                                 (above.a as f64 * 0.5 + below.a as f64 * 0.5) as u8)
            },
            BlendingMode::Paint => {
                let o = above.a as f64 / 255.0;
                let t = 1.0 - o;
                Pixel::from_rgba((above.r as f64 * o + below.r as f64 * t) as u8,
                                 (above.g as f64 * o + below.g as f64 * t) as u8,
                                 (above.b as f64 * o + below.b as f64 * t) as u8,
                                 std::cmp::max(above.a, below.a))
            }
        }
    }
}