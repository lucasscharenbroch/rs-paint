use super::{Canvas, Toolbar};

use gtk::gdk::ModifierType;
use gtk::cairo::Context;

#[derive(Clone, Copy)]
pub struct EyedropperState {
}

impl EyedropperState {
    pub fn default(_canvas: &Canvas) -> Self {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> Self {
        EyedropperState {
        }
    }
}

impl super::MouseModeState for EyedropperState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let (x, y) = canvas.cursor_pos_pix();
        if let Some(pix) = canvas.image().try_pix_at(y as i32, x as i32) {
           toolbar.set_primary_color(pix.to_rgba_struct());
        }
    }

    // drag_start is the only handled event
    // TODO default to empty?

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
    }

    fn handle_drag_end(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
    }

    fn handle_motion(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
    }

    fn handle_mod_key_update(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
    }

    fn draw(&self, _canvas: &Canvas, _cr: &Context) {
    }
}
