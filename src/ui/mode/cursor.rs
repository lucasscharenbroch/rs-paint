use super::Canvas;

// Cursor Mode: drag => pan

#[derive(Clone, Copy)]
pub struct CursorState {
    last_cursor_pos: (f64, f64),
}

impl CursorState {
    pub const fn default() -> CursorState {
        CursorState {
            last_cursor_pos: (0.0, 0.0),
        }
    }
}

impl super::MouseModeState for CursorState {
    fn handle_drag_start(&mut self, canvas: &mut Canvas) {
        self.last_cursor_pos = *canvas.cursor_pos();
    }

    fn handle_drag_update(&mut self, canvas: &mut Canvas) {
    }

    fn handle_drag_end(&mut self, canvas: &mut Canvas) {
    }
}
