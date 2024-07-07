use super::undo::action::{ActionName, AutoDiffAction, MultiLayerAction};
use super::{FusedLayer, Image, ImageLike, ImageLikeUnchecked, TrackedLayeredImage, Pixel,};

use gtk::gdk::RGBA;
use itertools::{Itertools, Either};

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

impl Scale {
    fn exec_scale_with_fn(&self, layer: &mut FusedLayer, interpolation_fn: fn(&Image, f32, f32) -> Pixel) {
        let new_sz = self.w * self.h;
        let mut new_pix = Vec::with_capacity(new_sz);

        for i in 0..self.w {
            for j in 0..self.h {
                // project (i, j) into the coords of `image`
                let x_proj = j as f32 * (layer.width() as f32 / self.w as f32);
                let y_proj = i as f32 * (layer.height() as f32 / self.h as f32);
                let p = interpolation_fn(&layer.image, x_proj, y_proj);

                new_pix.push(p);
            }
        }

        // TODO
        // image.set_image(Image::new(new_pix, self.w, self.h));
    }
}

impl AutoDiffAction for Scale {
    fn name(&self) -> ActionName {
        ActionName::Scale
    }

    fn exec(self, image: &mut impl TrackedLayeredImage) {
        todo!()
        /* TODO add full-image back-up mechanism to TrackedLayeredImage
        match self.method {
            ScaleMethod::NearestNeighbor => self.exec_scale_with_fn(image, nearest_neighbor),
            ScaleMethod::Bilinear => self.exec_scale_with_fn(image, bilinear),
        }
        */
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

/// Specifies where to place the current image with respect
/// to the added pixels
#[derive(Clone, PartialEq)]
pub enum ExpandJustification {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl ExpandJustification {
    /// Is `idx` within the justified window of size (`old_w`, `old_h`)
    /// within the image of size (`new_w`, `new_h`)?
    #[inline]
    fn take_idx(&self, idx: usize, old_w: usize, old_h: usize, new_w: usize, new_h: usize) -> bool {
        let new_sz = new_w * new_h;
        let (r, c) = (idx / new_w, idx % new_w);

        let r_in_window = match self {
            ExpandJustification::TopLeft |
            ExpandJustification::TopCenter |
            ExpandJustification::TopRight => r < old_h,
            ExpandJustification::MiddleLeft |
            ExpandJustification::MiddleCenter |
            ExpandJustification::MiddleRight => {
                let center = new_h / 2;
                let low_half = old_h / 2;
                let low_r = center - low_half;
                let high_r = low_r + old_h;
                r >= low_r && r < high_r
            },
            ExpandJustification::BottomLeft |
            ExpandJustification::BottomCenter |
            ExpandJustification::BottomRight => r >= new_h - old_h,
        };

        let c_in_window = match self {
            ExpandJustification::TopLeft |
            ExpandJustification::MiddleLeft |
            ExpandJustification::BottomLeft  => c < old_w,
            ExpandJustification::TopCenter |
            ExpandJustification::MiddleCenter |
            ExpandJustification::BottomCenter => {
                let center = new_w / 2;
                let low_half = old_w / 2; // truncate, if odd
                let low_c = center - low_half;
                let high_c = low_c + old_w;
                c >= low_c && c < high_c
            },
            ExpandJustification::TopRight |
            ExpandJustification::MiddleRight |
            ExpandJustification::BottomRight => c >= new_w - old_w,
        };

        r_in_window && c_in_window
    }

    /// Given a height/width, plus amounts to truncate,
    /// determine the rectangle of the justified region
    pub fn bounding_box_in(
        &self,
        height: usize,
        width: usize,
        height_margin: usize,
        width_margin: usize,
    ) -> (i32, i32, i32, i32) { // (x, y, w, h)
        let width = width as i32;
        let height = height as i32;
        let width_margin = width_margin as i32;
        let height_margin = height_margin as i32;

        let (w, h) = (
            width - width_margin,
            height - height_margin,
        );

        let x = match self {
            ExpandJustification::TopLeft |
            ExpandJustification::MiddleLeft |
            ExpandJustification::BottomLeft => 0,
            ExpandJustification::TopCenter |
            ExpandJustification::MiddleCenter |
            ExpandJustification::BottomCenter => width_margin / 2,
            ExpandJustification::TopRight |
            ExpandJustification::MiddleRight |
            ExpandJustification::BottomRight => width_margin,
        };

        let y = match self {
            ExpandJustification::TopLeft |
            ExpandJustification::TopCenter |
            ExpandJustification::TopRight => 0,
            ExpandJustification::MiddleLeft |
            ExpandJustification::MiddleCenter |
            ExpandJustification::MiddleRight => height_margin / 2,
            ExpandJustification::BottomLeft |
            ExpandJustification::BottomCenter |
            ExpandJustification::BottomRight => height_margin,
        };

        (x, y, w, h)
    }
}

#[derive(Clone)]
pub struct ExpandUndoInfo {
    old_w: usize,
    old_h: usize,
}

#[derive(Clone)]
pub struct Expand {
    added_w: usize,
    added_h: usize,
    justification: ExpandJustification,
    new_pix_color: RGBA,
}

impl Expand {
    pub fn new(
        added_w:usize,
        added_h: usize,
        justification: ExpandJustification,
        new_pix_color: RGBA
    ) -> Self {
        Expand {
            added_h,
            added_w,
            justification,
            new_pix_color,
        }
    }
}

impl MultiLayerAction for Expand {
    type LayerData = Option<ExpandUndoInfo>;

    fn new_layer_data(&self, image: &mut Image) -> Self::LayerData {
        None
    }

    fn name(&self) -> ActionName {
        ActionName::Expand
    }

    fn exec(&mut self, undo_info: &mut Option<ExpandUndoInfo>, image: &mut Image) {
        let old_w = image.width;
        let old_h = image.height;

        if let None = undo_info {
            *undo_info = Some(ExpandUndoInfo {
                old_h,
                old_w,
            });
        }

        let new_w = image.width + self.added_w;
        let new_h = image.height + self.added_h;
        let new_sz = new_w * new_h;

        let mut new_pix = Vec::with_capacity(new_sz);
        let mut old_idx = 0;

        for idx in 0..new_sz {
            if self.justification.take_idx(idx, old_w, old_h, new_w, new_h) {
                new_pix.push(image.pixels[old_idx].clone());
                old_idx += 1;
            } else {
                new_pix.push(Pixel::from_rgba_struct(self.new_pix_color));
            }
        }

        assert!(new_w * new_h == new_pix.len());
        image.pixels = new_pix;
        image.width = new_w;
        image.height = new_h;
    }

    fn undo(&mut self, undo_info: &mut Option<ExpandUndoInfo>, image: &mut Image) {
        let undo_info = undo_info.as_ref().unwrap();
        let old_w = undo_info.old_w;
        let old_h = undo_info.old_h;
        let old_sz = old_w * old_h;
        let mut old_pix = Vec::with_capacity(old_sz);

        for idx in 0..(image.height * image.width) {
            if self.justification.take_idx(idx, old_w, old_h, image.width, image.height) {
                old_pix.push(image.pixels[idx].clone());
            }
        }

        assert!(old_sz == old_pix.len());
        image.height = old_h;
        image.width = old_w;
        image.pixels = old_pix;
    }
}

pub struct CropUndoInfo {
    old_w: usize,
    old_h: usize,
    scrapped_pixels: Vec<Pixel>,
}

pub struct Crop {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Crop {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Crop {
            x, y, w, h,
        }
    }

    // Whether an index of the flat pixel array should be removed in the crop
    #[inline]
    fn should_keep_pix_at_idx(&self, old_w: usize, idx: usize) -> bool {
        let (i, j) = (idx / old_w, idx % old_w);
        j >= self.x && j < self.x + self.w && i >= self.y && i < self.y + self.h
    }
}

impl MultiLayerAction for Crop {
    type LayerData = Option<CropUndoInfo>;

    fn new_layer_data(&self, image: &mut Image) -> Self::LayerData {
        None
    }

    fn name(&self) -> ActionName {
        ActionName::Crop
    }

    fn exec(&mut self, undo_info: &mut Option<CropUndoInfo>, image: &mut Image) {
        let kept_pixels = if let None = undo_info {
            // only record undo_info on the first execution

            let old_h = image.height;
            let old_w = image.width;
            let (scrapped_pixels, kept_pixels): (Vec<_>, Vec<_>) = image.pixels.iter()
                .enumerate()
                .partition_map(|(idx, pix)| {
                    if self.should_keep_pix_at_idx(old_w, idx) {
                        Either::Right(pix.clone())
                    } else {
                        Either::Left(pix.clone())
                    }
                });

            *undo_info = Some(CropUndoInfo {
                old_h,
                old_w,
                scrapped_pixels,
            });

            kept_pixels
        } else {
            let old_w = undo_info.as_ref().unwrap().old_w;

            image.pixels.iter()
                .enumerate()
                .filter_map(|(idx, pix)| {
                    if self.should_keep_pix_at_idx(old_w, idx) {
                        Some(pix.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        image.width = self.w;
        image.height = self.h;
        image.pixels = kept_pixels;
    }

    fn undo(&mut self, undo_info: &mut Option<CropUndoInfo>, image: &mut Image) {
        let undo_info = undo_info.as_ref().unwrap();
        let old_sz = undo_info.old_h * undo_info.old_w;

        let mut old_pix = Vec::with_capacity(old_sz);

        let mut scrapped_idx = 0;
        let mut kept_idx = 0;

        for idx in 0..old_sz {
            if self.should_keep_pix_at_idx(undo_info.old_w, idx) {
                old_pix.push(image.pixels[kept_idx].clone());
                kept_idx += 1;
            } else {
                old_pix.push(undo_info.scrapped_pixels[scrapped_idx].clone());
                scrapped_idx += 1;
            };
        }

        image.width = undo_info.old_w;
        image.height = undo_info.old_h;
        image.pixels = old_pix;
    }
}
