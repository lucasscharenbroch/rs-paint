use crate::transformable::Transformable;

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
        let line_width = self.border_thickness as f64 / pixel_width;
        cr.set_line_width(line_width);
        let aspect_ratio = pixel_height / pixel_width;

        let _ = cr.save();
        {
            cr.scale(1.0, 1.0 / aspect_ratio);

            cr.set_source_rgba(
                self.outline_color.red() as f64,
                self.outline_color.green() as f64,
                self.outline_color.blue() as f64,
                self.outline_color.alpha() as f64,
            );

            self.shape_type.outline(cr, line_width, pixel_width, pixel_height);

            cr.set_source_rgba(
                self.fill_color.red() as f64,
                self.fill_color.green() as f64,
                self.fill_color.blue() as f64,
                self.fill_color.alpha() as f64,
            );

            self.shape_type.fill(cr, line_width, pixel_width, pixel_height);
        }
        let _ = cr.restore();
    }

    fn gen_sampleable(&self) -> Box<dyn crate::transformable::Samplable> {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub enum ShapeType {
    Square,
    Triangle,
}

impl ShapeType {
    fn outline(&self, cr: &cairo::Context, line_width: f64, pixel_width: f64, pixel_height: f64) {
        self.draw(cr, line_width, pixel_width , pixel_height);
        let _ = cr.stroke();
    }

    fn fill(&self, cr: &cairo::Context, line_width: f64, pixel_width: f64, pixel_height: f64) {
        self.draw(cr, line_width, pixel_width, pixel_height);
        let _ = cr.fill();
    }

    fn draw(&self, cr: &cairo::Context, line_width: f64, pixel_width: f64, pixel_height: f64) {
        let aspect_ratio = pixel_height / pixel_width;
        // a utility matrix to more easily calculate where
        // adjusted control points go
        let mut calc_matrix = cairo::Matrix::identity();
        calc_matrix.scale(1.0, aspect_ratio); // invert the scale done to the context

        /*
        let line_width_tr = line_width / pixel_width;
        calc_matrix.translate(line_width_tr, line_width_tr);

        let line_width_scale = ((pixel_width - (line_width * 2.0)) / pixel_width).max(0.0);
        calc_matrix.scale(line_width_scale, line_width_scale);
        */

        let (x0, y0) = calc_matrix.transform_point(0.0, 0.0);
        let (dx1, dy1) =  calc_matrix.transform_distance(1.0, 1.0);
        let (x1, y1) = calc_matrix.transform_point(1.0, 1.0);
        let (x05, y05) = calc_matrix.transform_point(0.5, 0.5);

        match self {
            Self::Square => {
                cr.rectangle(x0, y0, dx1, dy1);
            },
            Self::Triangle => {
                cr.move_to(x05, y0);
                cr.line_to(x1, y1);
                cr.line_to(y0, y1);
                cr.close_path();
            },
        }
    }
}
