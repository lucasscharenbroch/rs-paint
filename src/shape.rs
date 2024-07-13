use crate::transformable::Transformable;

use gtk::cairo;

pub trait Shape {
    /// Draw the untransformed shape within the unit
    /// square: (0.0, 0.0) (1.0, 1.0)
    fn draw(&mut self, cr: &cairo::Context);
}

impl<S: Shape> Transformable for S {
    fn draw(&mut self, cr: &cairo::Context) {
        Shape::draw(self, cr);
    }

    fn gen_sampleable(&self) -> Box<dyn crate::transformable::Samplable> {
        todo!()
    }
}

struct Square;

impl Shape for Square {
    fn draw(&mut self, cr: &cairo::Context) {
        cr.rectangle(0.0, 0.0, 1.0, 1.0);
        let _ = cr.stroke();
    }
}

struct Triangle;

impl Shape for Triangle {
    fn draw(&mut self, cr: &cairo::Context) {
        cr.move_to(0.5, 0.0);
        cr.line_to(1.0, 1.0);
        cr.line_to(0.0, 1.0);
        cr.close_path();
        let _ = cr.stroke();
    }
}

pub enum ShapeType {
    Square,
    Triangle,
}

impl ShapeType {
    fn to_shape(&self) -> Box<dyn Shape> {
        match self {
            Self::Square => Box::new(Square),
            Self::Triangle => Box::new(Triangle),
        }
    }
}
