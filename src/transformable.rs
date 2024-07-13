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
    fn sample(&self, x: f32, y: f32) -> Pixel;
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
    fn sample(&self, x: f32, y: f32) -> Pixel {
        let h = self.height() as f32;
        let w = self.width() as f32;

        self.pix_at(
            (y * h).floor().min(h - 1.0).max(0.0) as usize,
            (x * w).floor().min(w - 1.0).max(0.0) as usize,
        ).clone()
    }
}

pub struct SampleableCommit<'t> {
    matrix: cairo::Matrix,
    sampleable: &'t Box<dyn Samplable>,
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
        println!("do the thing")
    }
}
