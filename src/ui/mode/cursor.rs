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
        let (x, y) = self.last_cursor_pos;
        let (xp, yp) = canvas.cursor_pos();
        let (dx, dy) = (xp - x, yp - y);

        const DRAG_PAN_FACTOR: f64 = 0.02;

        canvas.inc_pan(dx * DRAG_PAN_FACTOR, dy * DRAG_PAN_FACTOR);
        canvas.update();
        self.last_cursor_pos = *canvas.cursor_pos();
    }

    fn handle_drag_end(&mut self, canvas: &mut Canvas) {
        self.handle_drag_update(canvas)
    }
}
