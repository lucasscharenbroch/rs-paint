use crate::image::undo::action::ActionName;
use crate::shape::{Shape, ShapeType};
use super::{Canvas, MouseMode, FreeTransformState, TransformMode, Toolbar};

use gtk::gdk::ModifierType;
use gtk::cairo;

#[derive(Clone, Copy)]
pub enum InsertShapeState {
    Neutral,
    /// Transfer ASAP
    TransferToFreeTransform(f64, f64, cairo::Matrix),
}

impl InsertShapeState {
    pub fn default(_canvas: &Canvas) -> InsertShapeState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> InsertShapeState {
        InsertShapeState::Neutral
    }
}

impl super::MouseModeState for InsertShapeState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let (x, y) = canvas.cursor_pos_pix_f();
        let shape_type = toolbar.get_shape_type();
        let border_width = toolbar.get_shape_border_width();
        let shape = Shape::new(
            shape_type,
            border_width,
            toolbar.primary_color(),
            toolbar.secondary_color(),
        );

        *canvas.transformable_and_culprit().borrow_mut() = Some((Box::new(shape), ActionName::InsertShape));

        let mut matrix = cairo::Matrix::identity();
        matrix.translate(x, y);
        *self = Self::TransferToFreeTransform(x, y, matrix);
    }

    fn try_transfer(&self) -> Result<MouseMode, ()> {
        if let Self::TransferToFreeTransform(x, y, matrix) = self {
            Ok(MouseMode::FreeTransform(
                FreeTransformState::from_transform_mode_and_coords(TransformMode::Transforming(matrix.clone()), *x, *y)
            ))
        } else {
            Err(())
        }
    }
}
