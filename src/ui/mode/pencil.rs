use super::Canvas;

#[derive(Clone, Copy)]
pub struct PencilState {
}

impl PencilState {
    pub const fn default() -> PencilState {
        PencilState {
        }
    }
}

impl super::MouseModeState for PencilState {

    fn handle_drag_start(&self, _canvas: &mut Canvas) {
    }

    fn handle_drag_update(&self, canvas: &mut Canvas) {
    }

    fn handle_drag_end(&self, canvas: &mut Canvas) {
    }
}
