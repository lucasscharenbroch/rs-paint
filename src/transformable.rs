use crate::image::{undo::action::{ActionName, AutoDiffAction}, DrawableImage, Image, ImageLike, ImageLikeUnchecked, Pixel};

use gtk::cairo;

pub trait Transformable {
    /// Draw the untransformed thing within the unit
    /// square: (0.0, 0.0) (1.0, 1.0)
    fn draw(&mut self, cr: &cairo::Context);
    // TODO: avoid the generation/make an accessor (?)
    fn gen_sampleable(&self) -> Box<dyn Samplable>;
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
    fn draw(&mut self, cr: &cairo::Context) {
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

    fn gen_sampleable(&self) -> Box<dyn Samplable> {
        self.image.clone()
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

pub struct SampleableCommit<'s> {
    matrix: cairo::Matrix,
    sampleable: &'s Box<dyn Samplable>,
}

impl<'s> SampleableCommit<'s> {
    pub fn new(sampleable: &'s Box<dyn Samplable>, matrix: cairo::Matrix) -> Self {
        SampleableCommit {
            matrix,
            sampleable,
        }
    }
}

impl<'s> AutoDiffAction for SampleableCommit<'s> {
    fn name(&self) -> ActionName {
        ActionName::Transform
    }

    fn exec(self, image: &mut impl crate::image::TrackedLayeredImage) {
        let inverse = self.matrix.try_invert().unwrap();

        // get the tranformation's corners
        let corners =  vec![
            inverse.transform_point(0.0, 0.0),
            inverse.transform_point(1.0, 0.0),
            inverse.transform_point(0.0, 1.0),
            inverse.transform_point(1.0, 1.0),
        ];

        // compute bounding box/ extreme coordinates
        let min_x = corners.iter().map(|c| c.0).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_x = corners.iter().map(|c| c.0).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_y = corners.iter().map(|c| c.1).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_y = corners.iter().map(|c| c.1).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        // cast them into existant pixel coordinates
        let min_x = (min_x.floor() as usize).max(0);
        let max_x = (max_x.ceil() as usize).max(image.width() as usize - 1);
        let min_y = (min_y.floor() as usize).max(0);
        let max_y = (max_y.ceil() as usize).max(image.height() as usize - 1);

        // iterate all pixels in the bounding box
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let (xp, yp) = inverse.transform_point(x as f64 + 0.5, y as f64 + 0.5);
                if xp < 0.0 || xp > 1.0 || yp < 0.0 || yp > 1.0 {
                    continue; // skip if out of selection
                }

                let s = self.sampleable.sample(xp, yp);
                let p = image.pix_at_mut(y as i32, x as i32);

                *p = Pixel::blend(&s, p);
            }
        }
    }
}
