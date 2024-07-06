use super::{Canvas, Toolbar};

use gtk::gdk::ModifierType;

// Cursor Mode: drag => pan

#[derive(Clone, Copy)]
pub struct CursorState {
    last_cursor_pos: (f64, f64),
}

impl CursorState {
    pub fn default(_canvas: &Canvas) -> CursorState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> CursorState {
        CursorState {
            last_cursor_pos: (0.0, 0.0),
        }
    }
}

impl super::MouseModeState for CursorState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        self.last_cursor_pos = *canvas.cursor_pos();
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        let (x, y) = self.last_cursor_pos;
        let (xp, yp) = canvas.cursor_pos();
        let (dx, dy) = (xp - x, yp - y);

        const DRAG_PAN_FACTOR: f64 = 0.02;

        canvas.inc_pan(dx * DRAG_PAN_FACTOR, dy * DRAG_PAN_FACTOR);
        canvas.update();
        self.last_cursor_pos = *canvas.cursor_pos();
    }

    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.handle_drag_update(mod_keys, canvas, toolbar)
    }
}
