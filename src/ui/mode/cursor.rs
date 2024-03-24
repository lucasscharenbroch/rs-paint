use super::Canvas;

// Cursor Mode: drag => pan

#[derive(Clone, Copy)]
pub struct CursorState {
}

impl CursorState {
    pub const fn default() -> CursorState {
        CursorState {
        }
    }
}

impl super::MouseModeState for CursorState {

    fn handle_drag_start(&self, _canvas: &mut Canvas) {
    }

    fn handle_drag_update(&self, canvas: &mut Canvas) {
    }

    fn handle_drag_end(&self, canvas: &mut Canvas) {
    }
}
