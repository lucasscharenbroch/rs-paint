mod cursor;
mod pencil;

use cursor::CursorState;
use pencil::PencilState;
use super::canvas::Canvas;

#[derive(Clone, Copy)]
pub enum MouseMode {
    Cursor(cursor::CursorState),
    Pencil(pencil::PencilState),
}

trait MouseModeState {
    fn handle_drag_start(&mut self, canvas: &mut Canvas);
    fn handle_drag_update(&mut self, canvas: &mut Canvas);
    fn handle_drag_end(&mut self, canvas: &mut Canvas);
}

impl PartialEq<MouseMode> for MouseMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MouseMode::Cursor(_), MouseMode::Cursor(_)) => true,
            (MouseMode::Pencil(_), MouseMode::Pencil(_)) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl MouseMode {
    pub const fn cursor() -> MouseMode {
        MouseMode::Cursor(CursorState::default())
    }

    pub const fn pencil() -> MouseMode {
        MouseMode::Pencil(PencilState::default())
    }

    fn get_state(&self) -> Box<dyn MouseModeState> {
        match self {
            MouseMode::Cursor(s) => Box::new(*s),
            MouseMode::Pencil(s) => Box::new(*s),
        }
    }

    pub fn handle_drag_start(&mut self, canvas: &mut Canvas) {
        self.get_state().handle_drag_start(canvas);
    }

    pub fn handle_drag_update(&mut self, canvas: &mut Canvas) {
        self.get_state().handle_drag_update(canvas);
    }

    pub fn handle_drag_end(&mut self, canvas: &mut Canvas) {
        self.get_state().handle_drag_end(canvas);
    }
}
