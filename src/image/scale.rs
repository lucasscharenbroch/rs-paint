use super::undo::action::{DoableAction, StaticDoableAction, ActionName};
use super::{Image, ImageLike, Pixel, UnifiedImage};

#[derive(Clone)]
pub enum ScaleMethod {
    NearestNeighbor,
    Bilinear,
}

#[derive(Clone)]
pub struct Scale {
    method: ScaleMethod,
    w: usize,
    h: usize,
}

impl Scale {
    pub fn new(w: usize, h: usize, method: ScaleMethod) -> Self {
        Scale {
            w,
            h,
            method
        }
    }
}

impl StaticDoableAction for Scale {
    fn dyn_clone(&self) -> Box<dyn DoableAction> {
        Box::new(self.clone())
    }
}

impl Scale {
    fn exec_scale_with_fn(&self, image: &mut UnifiedImage, interpolation_fn: fn(&Image, f32, f32) -> Pixel) {
        let new_sz = self.w * self.h;
        let mut new_pix = Vec::with_capacity(new_sz);

        for i in 0..self.w {
            for j in 0..self.h {
                // project (i, j) into the coords of `image`
                let x_proj = j as f32 * (image.width() as f32 / self.w as f32);
                let y_proj = i as f32 * (image.height() as f32 / self.h as f32);
                let p = interpolation_fn(&image.image, x_proj, y_proj);

                new_pix.push(p);
            }
        }

        image.set_image(Image::new(new_pix, self.w, self.h));
    }
}

impl DoableAction for Scale {
    fn name(&self) -> ActionName {
        ActionName::Scale
    }

    fn exec(&self, image: &mut UnifiedImage) {
        match self.method {
            ScaleMethod::NearestNeighbor => self.exec_scale_with_fn(image, nearest_neighbor),
            ScaleMethod::Bilinear => self.exec_scale_with_fn(image, bilinear),
        }
    }
}

#[inline]
fn nearest_neighbor(image: &Image, x: f32, y: f32) -> Pixel {
    image.pix_at(y.floor() as usize, x.floor() as usize).clone()
}

impl Pixel {
    #[inline]
    fn to_rgba_f32(&self) -> (f32, f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }

    #[inline]
    fn from_rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Pixel::from_rgba(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        )
    }

    #[inline]
    fn weighted_avg(a: &Pixel, b: &Pixel, percent_a: f32) -> Pixel {
        let percent_b = 1.0 - percent_a;

        let (ar, ag, ab, aa) = a.to_rgba_f32();
        let (br, bg, bb, ba) = b.to_rgba_f32();

        Pixel::from_rgba_f32(
            ar * percent_a + br * percent_b,
            ag * percent_a + bg * percent_b,
            ab * percent_a + bb * percent_b,
            aa * percent_a + ba * percent_b,
        )
    }
}

#[inline]
fn bilinear(image: &Image, x: f32, y: f32) -> Pixel {
    // find four nearest points:
    // (the `.` is (x, y))
    // -----------
    // |p00 | p10|
    // ---- . ----
    // |p01   p11|
    // -----------

    let p00 = (x.floor() as usize, y.floor() as usize);
    let p01 = (p00.0, p00.1 + 1);
    let p10 = (p00.0 + 1, p00.1);
    let p11 = (p00.0 + 1, p00.1 + 1);

    let percent_left = 1.0 - (x - p00.0 as f32);
    let percent_up = 1.0 - (y - p00.1 as f32);

    // p00 should always be in-bounds
    let p00 = image.pix_at(p00.1, p00.0);

    // default p01/p10 to p00, and p11 to either of p01/p10 that isn't defaulted already
    // perhaps this is slow (branching) / unnecessary (only counts on borders when enlarging)
    let maybe_p01 = image.try_pix_at(p01.1, p01.0);
    let maybe_p10 = image.try_pix_at(p10.1, p10.0);
    let p11 = image.try_pix_at(p11.1, p11.0).unwrap_or(
        maybe_p01.unwrap_or(maybe_p10.unwrap_or(p00))
    );
    let p01 = maybe_p01.unwrap_or(p00);
    let p10 = maybe_p10.unwrap_or(p00);

    let top_two_avg = Pixel::weighted_avg(&p00, &p10, percent_left);
    let bottom_two_avg = Pixel::weighted_avg(&p01, &p11, percent_left);

    Pixel::weighted_avg(&top_two_avg, &bottom_two_avg, percent_up)
}
