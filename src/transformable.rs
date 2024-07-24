use crate::image::{resize::ScaleMethod, undo::action::{ActionName, AutoDiffAction}, DrawableImage, DrawablePixel, Image, ImageLike, ImageLikeUnchecked, Pixel};

use gtk::cairo;

pub trait Transformable {
    /// Draw the untransformed thing within the unit
    /// square: (0.0, 0.0) (1.0, 1.0)
    fn draw(&mut self, cr: &cairo::Context, pixel_width: f64, pixel_height: f64);
    fn gen_sampleable(&mut self, pixel_width: f64, pixel_height: f64) -> Box<dyn crate::transformable::Samplable> {
        // default implementation: ride off the back of cairo,
        // use the `draw` method, then sample off of the resulting context

        let width = pixel_width.ceil() as usize;
        let height = pixel_height.ceil() as usize;

        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            width as i32,
            height as i32,
        ).unwrap();

        let cr = cairo::Context::new(&surface).unwrap();
        cr.scale(pixel_width, pixel_height);

        self.draw(&cr, pixel_width, pixel_height);

        std::mem::drop(cr);
        let raw_data = surface.take_data().unwrap();
        let drawable_image = DrawableImage::from_raw_data(width, height, raw_data);

        Box::new(drawable_image)
    }
    /// yeilds a reference to the underlying image (if that's what's begin encapsulated) -
    /// this allows for point-wise interpolation (in commits) and transformation-selection-copying
    fn try_image_ref(&self) -> Option<&Image> {
        None
    }
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

    fn try_image_ref(&self) -> Option<&Image> {
        Some(&*self.image)
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

pub struct SampleableCommit<'s, 'i> {
    matrix: cairo::Matrix,
    sampleable: &'s dyn Samplable,
    scale_method: ScaleMethod,
    image_option: Option<&'i Image>,
    culprit: ActionName,
}

impl<'s, 'i> SampleableCommit<'s, 'i> {
    pub fn new(
        sampleable: &'s dyn Samplable,
        matrix: cairo::Matrix,
        scale_method: ScaleMethod,
        image_option: Option<&'i Image>,
        culprit: ActionName
    ) -> Self {
        SampleableCommit {
            matrix,
            sampleable,
            scale_method,
            image_option,
            culprit,
        }
    }
}

impl<'s, 'i> AutoDiffAction for SampleableCommit<'s, 'i> {
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

        let sample_fn: Box<dyn Fn(f64, f64) -> Pixel> = if let Some(image) = self.image_option {
            let interpolation_fn = self.scale_method.interpolation_fn::<Image>();
            let width = image.width() as f32;
            let height = image.height() as f32;
            Box::new(move |x, y| interpolation_fn(image, x as f32 * width, y as f32 * height))
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
