use super::{Canvas, Toolbar};

use gtk::gdk::ModifierType;

#[derive(Clone, Copy)]
pub struct TextState;

impl TextState {
    pub fn default(_canvas: &Canvas) -> TextState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> TextState {
        TextState {
        }
    }
}

impl super::MouseModeState for TextState {
}
