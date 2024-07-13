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
    fn draw(&mut self, cr: &cairo::Context) {
        cr.set_line_width(0.05);

        cr.set_source_rgba(
            self.outline_color.red() as f64,
            self.outline_color.green() as f64,
            self.outline_color.blue() as f64,
            self.outline_color.alpha() as f64,
        );

        self.shape_type.outline(cr);

        cr.set_source_rgba(
            self.fill_color.red() as f64,
            self.fill_color.green() as f64,
            self.fill_color.blue() as f64,
            self.fill_color.alpha() as f64,
        );

        self.shape_type.fill(cr);
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
    fn outline(&self, cr: &cairo::Context) {
        self.draw(cr);
        let _ = cr.stroke();
    }

    fn fill(&self, cr: &cairo::Context) {
        self.draw(cr);
        let _ = cr.fill();
    }

    fn draw(&self, cr: &cairo::Context)  {
        match self {
            Self::Square => {
                cr.rectangle(0.0, 0.0, 1.0, 1.0);
            },
            Self::Triangle => {
                cr.move_to(0.5, 0.0);
                cr.line_to(1.0, 1.0);
                cr.line_to(0.0, 1.0);
                cr.close_path();
            },
        }
    }
}
