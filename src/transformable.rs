use crate::image::{resize::ScaleMethod, undo::action::{ActionName, AutoDiffAction}, DrawableImage, DrawablePixel, Image, ImageLike, ImageLikeUnchecked, Pixel};

use gtk::cairo;
use std::cell::{RefCell, UnsafeCell};

pub trait Transformable {
    /// Draw the untransformed thing within the unit
    /// square: (0.0, 0.0) (1.0, 1.0)
    fn draw(&mut self, cr: &cairo::Context, pixel_width: f64, pixel_height: f64);
    fn gen_sampleable(&mut self, pixel_width: f64, pixel_height: f64) -> Box<dyn Samplable>;
    /// returns Some((width, height)) if the underlying samplable is
    /// made up of discrete units (like an image), otherwise None
    /// (this is used for interpolation)
    fn try_size(&self) -> Option<(usize, usize)>;
}

pub trait Samplable {
    /// Get a pixel value at given (x, y)
    /// (coords should be in the unit square)
    fn sample(&self, x: f64, y: f64) -> Pixel;
}

pub struct TransformableImage {
    image: Box<Image>,
    drawable: DrawableImage,
}

impl TransformableImage {
    pub fn from_image(image: Image) -> Self {
        let drawable = DrawableImage::from_image(&image);

        TransformableImage {
            image: Box::new(image),
            drawable,
        }
    }
}

impl Transformable for TransformableImage {
    fn draw(&mut self, cr: &cairo::Context, _pixel_width: f64, _pixel_height: f64) {
        let img_width = self.image.width() as f64;
        let img_height = self.image.height() as f64;

        let _ = cr.save();
        {
            cr.scale(1.0 / img_width, 1.0 / img_height);
            let _ = cr.set_source(self.drawable.to_surface_pattern());
            cr.rectangle(0.0, 0.0, img_width, img_height);
            let _ = cr.fill();
        }
        let _ = cr.restore();
    }

    fn gen_sampleable(&mut self, _pixel_width: f64, _pixel_height: f64) -> Box<dyn Samplable> {
        self.image.clone()
    }

    fn try_size(&self) -> Option<(usize, usize)> {
        Some((self.image.width(), self.image.height()))
    }
}

impl Samplable for Image {
    fn sample(&self, x: f64, y: f64) -> Pixel {
        let h = self.height() as f64;
        let w = self.width() as f64;

        self.pix_at(
            (y * h).floor().min(h - 1.0).max(0.0) as usize,
            (x * w).floor().min(w - 1.0).max(0.0) as usize,
        ).clone()
    }
}

// This is only used for shapes (which ride off the back
// of cairo's drawing, using the context to retrive a DrawableImage).
// Pixel colors might be slightly off because of the lossy inversion
// from pre-multiplied alpha.
impl Samplable for DrawableImage {
    fn sample(&self, x: f64, y: f64) -> Pixel {
        #[inline]
        fn pix_at(drawable: &DrawableImage, i: usize, j: usize) -> &DrawablePixel {
            &drawable.pixels()[i * drawable.width() + j]
        }

        let h = self.height() as f64;
        let w = self.width() as f64;

        pix_at(
            self,
            (y * h).floor().min(h - 1.0).max(0.0) as usize,
            (x * w).floor().min(w - 1.0).max(0.0) as usize,
        ).to_pixel_lossy()
    }
}

/// A wrapper for a Samplable (reference) with
/// an underlying size, allowing for pixel accesses
/// (and therefore interpolation)
struct SizedSampleable<'s> {
    sampleable: &'s dyn Samplable,
    // the width/height are used to determine the interpolation
    // points (still within the unit square)
    width: usize,
    height: usize,
    /// hack to allow `pix_at()` to return a reference (even though
    /// an owned `Pixel` is produced) -- this is necessary because the
    /// `Samplealbe` abstracts away the ability to reference the underlying
    /// pixel vector
    // TODO refactor by removing a little generality (in Sampleable, then here)?
    memo_table: UnsafeCell<Vec<Pixel>>,
    memo_mask: RefCell<Vec<bool>>,
}

impl<'s> SizedSampleable<'s> {
    fn new(sampleable: &'s dyn Samplable, width: usize, height: usize) -> SizedSampleable<'s> {
        let memo_table = UnsafeCell::new(Vec::with_capacity(width * height));
        unsafe {
            (*memo_table.get()).set_len(width * height);
        }
        let memo_mask = RefCell::new(vec![false; width * height]);

        SizedSampleable {
            sampleable,
            width,
            height,
            memo_table,
            memo_mask,
        }
    }
}

impl<'s> ImageLike for SizedSampleable<'s> {
    fn height(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn try_pix_at(&self, r: usize, c: usize) -> Option<&Pixel> {
        if r < self.height && c < self.width {
            Some(self.pix_at(r, c))
        } else {
            None
        }
    }
}

impl<'s> ImageLikeUnchecked for SizedSampleable<'s> {
    fn pix_at(&self, r: usize, c: usize) -> &Pixel {
        let i = r * self.width + c;
        let x = (c as f64 + 0.5) / self.width as f64;
        let y = (r as f64 + 0.5) / self.height as f64;

        unsafe {
            let mut mask = self.memo_mask.borrow_mut();
            let table = &mut *self.memo_table.get();

            if !mask[i] {
                table[i] = self.sampleable.sample(x, y);
                mask[i] = true;
            }

            &table[i]
        }
    }

    fn pix_at_flat(&self, i:usize) -> &Pixel {
        self.pix_at(i / self.width, i % self.width)
    }
}

pub struct SampleableCommit<'s> {
    matrix: cairo::Matrix,
    sampleable: &'s dyn Samplable,
    scale_method: ScaleMethod,
    size_option: Option<(usize, usize)>,
    culprit: ActionName,
}

impl<'s> SampleableCommit<'s> {
    pub fn new(
        sampleable: &'s dyn Samplable,
        matrix: cairo::Matrix,
        scale_method: ScaleMethod,
        size_option: Option<(usize, usize)>,
        culprit: ActionName
    ) -> Self {
        SampleableCommit {
            matrix,
            sampleable,
            scale_method,
            size_option,
            culprit,
        }
    }
}

impl<'s> AutoDiffAction for SampleableCommit<'s> {
    fn name(&self) -> ActionName {
        self.culprit
    }

    fn exec(self, image: &mut impl crate::image::TrackedLayeredImage) {
        let inverse = self.matrix.try_invert().unwrap();

        // get the tranformation's corners
        let corners =  vec![
            self.matrix.transform_point(0.0, 0.0),
            self.matrix.transform_point(1.0, 0.0),
            self.matrix.transform_point(0.0, 1.0),
            self.matrix.transform_point(1.0, 1.0),
        ];

        // compute bounding box/ extreme coordinates
        let min_x = corners.iter().map(|c| c.0).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_x = corners.iter().map(|c| c.0).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_y = corners.iter().map(|c| c.1).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_y = corners.iter().map(|c| c.1).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        // cast them into existant pixel coordinates
        let min_x = (min_x.floor() as usize).max(0);
        let max_x = (max_x.ceil() as usize).min(image.width() as usize - 1);
        let min_y = (min_y.floor() as usize).max(0);
        let max_y = (max_y.ceil() as usize).min(image.height() as usize - 1);

        let sample_fn: Box<dyn Fn(f64, f64) -> Pixel> = if let Some((width, height)) = self.size_option {
            let sized_samplable = SizedSampleable::new(self.sampleable, width, height);
            let interpolation_fn = self.scale_method.interpolation_fn::<SizedSampleable>();
            Box::new(move |x, y| interpolation_fn(&sized_samplable, x as f32 * width as f32, y as f32 * height as f32))
        } else {
            Box::new(|x, y| self.sampleable.sample(x, y))
        };

        // iterate all pixels in the bounding box
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let (xp, yp) = inverse.transform_point(x as f64 + 0.5, y as f64 + 0.5);
                if xp < 0.0 || xp > 1.0 || yp < 0.0 || yp > 1.0 {
                    continue; // skip if out of selection
                }

                let s = sample_fn(xp, yp);
                let p = image.pix_at_mut(y as i32, x as i32);

                *p = Pixel::blend(&s, p);
            }
        }
    }
}
