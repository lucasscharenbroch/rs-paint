use super::{Canvas, Toolbar};

use gtk::gdk::ModifierType;

#[derive(Clone, Copy)]
pub struct InsertShapeState;

impl InsertShapeState {
    pub fn default(_canvas: &Canvas) -> InsertShapeState {
        InsertShapeState {
        }
    }

    pub const fn default_no_canvas() -> InsertShapeState {
        InsertShapeState {
        }
    }
}

impl super::MouseModeState for InsertShapeState {
}
