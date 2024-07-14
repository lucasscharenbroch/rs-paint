use std::sync::Arc;

use crate::{geometry::matrix_width_height, image::DrawableImage, transformable::Transformable};

use gtk::{cairo, gdk::RGBA};

pub struct Shape {
    shape_type: ShapeType,
    border_thickness: u8,
    outline_color: RGBA,
    fill_color: RGBA,
}

impl Shape {
    pub fn new(shape_type: ShapeType, border_thickness: u8, outline_color: RGBA, fill_color: RGBA) -> Self {
        Self {
            shape_type,
            border_thickness,
            outline_color,
            fill_color,
        }
    }
}

impl Transformable for Shape {
    fn draw(&mut self, cr: &cairo::Context, pixel_width: f64, pixel_height: f64) {
        let line_width = (self.border_thickness as f64 / pixel_width).min(1.0).min(pixel_height / pixel_width);
        cr.set_line_width(line_width);
        cr.set_line_join(cairo::LineJoin::Round);
        let aspect_ratio = pixel_height / pixel_width;

        // draw in unit square without regaurd to aspect ratio,
        // but scale before stroking (ensuring borders are uniform
        // in thickness)
        self.shape_type.draw(cr, line_width, pixel_width, pixel_height);

        let _ = cr.save();
        {
            cr.scale(1.0, 1.0 / aspect_ratio);

            cr.set_source_rgba(
                self.outline_color.red() as f64,
                self.outline_color.green() as f64,
                self.outline_color.blue() as f64,
                self.outline_color.alpha() as f64,
            );

            let _ = cr.stroke_preserve();

            cr.set_source_rgba(
                self.fill_color.red() as f64,
                self.fill_color.green() as f64,
                self.fill_color.blue() as f64,
                self.fill_color.alpha() as f64,
            );

            let _ = cr.fill();
        }
        let _ = cr.restore();
    }

    fn gen_sampleable(&mut self, pixel_width: f64, pixel_height: f64) -> Box<dyn crate::transformable::Samplable> {
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
}

#[derive(Clone, Copy)]
pub enum ShapeType {
    Square,
    TriangleI,
    Circle,
    Arrow,
    Star5,
    Hexagon,
    Diamond,
    TriangleE,
    Heart,
    SpeachBubble,
    Star4,
    Pentagon,
}

impl ShapeType {
    pub fn iter_variants() -> impl Iterator<Item = Self> {
        [
            Self::Square,
            Self::TriangleI,
            Self::Circle,
            Self::Arrow,
            Self::Star5,
            Self::Hexagon,
            Self::Diamond,
            Self::TriangleE,
            Self::Heart,
            Self::SpeachBubble,
            Self::Star4,
            Self::Pentagon,
        ].iter().map(|x| x.clone())
    }

    fn draw(&self, cr: &cairo::Context, line_width: f64, pixel_width: f64, pixel_height: f64) {
        let aspect_ratio = pixel_height / pixel_width;
        // a utility matrix to more easily calculate where
        // adjusted control points go
        let mut calc_matrix = cairo::Matrix::identity();

        let line_width_x_offset = line_width / 2.0;
        let line_width_y_offset = line_width / 2.0 / aspect_ratio;
        calc_matrix.translate(line_width_x_offset, line_width_y_offset);

        let line_width_x_scale = (1.0 - line_width).max(0.001);
        let line_width_y_scale = (1.0 - line_width / aspect_ratio).max(0.001);
        calc_matrix.scale(line_width_x_scale, line_width_y_scale);

        // ideally we could lazily compute these; perhaps that is automatically optimized (?)
        // an alternative is to make them inline functions
        let (x0, y0) = calc_matrix.transform_point(0.0, 0.0);
        let (dx1, dy1) =  calc_matrix.transform_distance(1.0, 1.0);
        let (x1, y1) = calc_matrix.transform_point(1.0, 1.0);
        let (x01, y01) = calc_matrix.transform_point(0.1, 0.1);
        let (x02, y02) = calc_matrix.transform_point(0.2, 0.2);
        let (x025, y025) = calc_matrix.transform_point(0.25, 0.25);
        let (x033, y033) = calc_matrix.transform_point(0.333, 0.333);
        let (x04, y04) = calc_matrix.transform_point(0.4, 0.4);
        let (x05, y05) = calc_matrix.transform_point(0.5, 0.5);
        let (x066, y066) = calc_matrix.transform_point(0.666, 0.666);
        let (x06, y06) = calc_matrix.transform_point(0.6, 0.6);
        let (x075, y075) = calc_matrix.transform_point(0.75, 0.75);

        match self {
            Self::Square => {
                cr.rectangle(x0, y0, dx1, dy1);
            },
            Self::TriangleI => {
                cr.move_to(x05, y0);
                cr.line_to(x1, y1);
                cr.line_to(x0, y1);
                cr.close_path();
            },
            Self::Circle => {
                // this gets mangled in extreme dimensions: not sure if
                // it's possible to prevent this; this entire tool is
                // pretty scuffed in the first place, though, so it's okay

                let _ = cr.save();
                {
                    let y_factor = dy1 / dx1;
                    cr.scale(1.0, y_factor);
                    let y05p = y05 / y_factor;
                    cr.arc(x05, y05p, dx1 / 2.0, 0.0, 2.0 * 3.141592);
                }
                let _ = cr.restore();
            },
            Self::Arrow => {
                cr.move_to(x05, y0);
                cr.line_to(x1, y05);
                cr.line_to(x05, y1);
                cr.line_to(x05, y066);
                cr.line_to(x0, y066);
                cr.line_to(x0, y033);
                cr.line_to(x05, y033);
                cr.close_path();
            },
            Self::Star5 => {
                // draw the star like a human would (winding fill should work with this)
                // not because it's better, but because it's more scuffed (and easier, of course)
                // nobody uses the shape tool seriously anyway
                cr.move_to(x025, y1);
                cr.line_to(x05, y0);
                cr.line_to(x075, y1);
                cr.line_to(x0, y033);
                cr.line_to(x1, y033);
                cr.close_path();
            },
            Self::Hexagon => {
                cr.move_to(x025, y0);
                cr.line_to(x075, y0);
                cr.line_to(x1, y05);
                cr.line_to(x075, y1);
                cr.line_to(x025, y1);
                cr.line_to(x0, y05);
                cr.close_path();
            },
            Self::Diamond => {
                cr.move_to(x05, y0);
                cr.line_to(x1, y05);
                cr.line_to(x05, y1);
                cr.line_to(x0, y05);
                cr.close_path();
            },
            Self::TriangleE => {
                cr.move_to(x0, y0);
                cr.line_to(x1, y1);
                cr.line_to(x0, y1);
                cr.close_path();
            },
            Self::Heart => {
                let (x0125, y0125) = calc_matrix.transform_point(0.125, 0.125);

                cr.move_to(x05, y1);
                cr.line_to(x0125, y05);
                cr.arc(x025, y025, dx1 / 4.0, -1.2 * 3.14, -0.15 * 3.14);
                cr.line_to(x05, y02);
                cr.arc(x075, y025, dx1 / 4.0, 1.2 * 3.14, 0.15 * 3.14);
                cr.close_path();
            },
            Self::SpeachBubble => {
                cr.move_to(x0, y0);
                cr.line_to(x1, y0);
                cr.line_to(x1, y075);
                cr.line_to(x04, y075);
                cr.line_to(x033, y1);
                cr.line_to(x025, y075);
                cr.line_to(x0, y075);
                cr.close_path();
            },
            Self::Star4 => {
                cr.move_to(x05, y0);
                cr.line_to(x06, y04);
                cr.line_to(x1, y05);
                cr.line_to(x06, y06);
                cr.line_to(x05, y1);
                cr.line_to(x04, y06);
                cr.line_to(x0, y05);
                cr.line_to(x04, y04);
                cr.close_path();
            },
            Self::Pentagon => {
                cr.move_to(x0, y04);
                cr.line_to(x05, y0);
                cr.line_to(x1, y04);
                cr.line_to(x075, y1);
                cr.line_to(x025, y1);
                cr.close_path();
            },
        }
    }
}
