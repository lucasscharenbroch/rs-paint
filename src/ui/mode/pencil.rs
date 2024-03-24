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

    fn handle_drag_start(&mut self, _canvas: &mut Canvas) {
    }

    fn handle_drag_update(&mut self, canvas: &mut Canvas) {
    }

    fn handle_drag_end(&mut self, canvas: &mut Canvas) {
    }
}
