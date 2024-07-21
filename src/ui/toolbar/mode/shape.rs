use crate::image::undo::action::ActionName;
use crate::shape::{Shape, ShapeType};
use super::{Canvas, MouseMode, FreeTransformState, Toolbar};

use gtk::gdk::ModifierType;
use gtk::cairo;

#[derive(Clone, Copy)]
pub enum ShapeState {
    Neutral,
    /// Transfer ASAP
    TransferToFreeTransform(f64, f64),
}

impl ShapeState {
    pub fn default(_canvas: &Canvas) -> ShapeState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> ShapeState {
        ShapeState::Neutral
    }
}

impl super::MouseModeState for ShapeState {
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

        let mut matrix = cairo::Matrix::identity();
        matrix.translate(x, y);

        *canvas.transformation_selection().borrow_mut() = Some(super::TransformationSelection::new(
            Box::new(shape),
            matrix,
            ActionName::InsertShape,
        ));

        *self = Self::TransferToFreeTransform(x, y);
    }

    fn try_transfer(&self) -> Result<MouseMode, ()> {
        if let Self::TransferToFreeTransform(x, y) = self {
            Ok(MouseMode::FreeTransform(
                FreeTransformState::from_coords(*x, *y)
            ))
        } else {
            Err(())
        }
    }
}
