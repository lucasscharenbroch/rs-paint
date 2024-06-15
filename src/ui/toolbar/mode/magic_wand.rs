use crate::ui::form::Form;
use super::{Canvas, MouseModeVariant, Toolbar};

use gtk::gdk::ModifierType;

#[derive(Clone, Copy)]
pub struct MagicWandState {
}

impl MagicWandState {
    pub fn default(_canvas: &Canvas) -> MagicWandState {
        Self::default_no_canvas() // TODO (?)
    }

    pub const fn default_no_canvas() -> MagicWandState {
        MagicWandState {
        }
    }
}

impl super::MouseModeState for MagicWandState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        todo!()
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        todo!()
    }

    fn handle_drag_end(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        todo!()
    }
}
