use super::canvas::Canvas;

mod cursor;
mod pencil;

#[derive(Clone, Copy, PartialEq)]
pub enum MouseMode {
    Cursor,
    Pencil,
}

impl MouseMode {
    pub fn handle_drag_start(&self, canvas: &mut Canvas) {
        match self {
            MouseMode::Cursor => cursor::handle_drag_start(canvas),
            MouseMode::Pencil => pencil::handle_drag_start(canvas),
        }
    }

    pub fn handle_drag_update(&self, canvas: &mut Canvas) {
        match self {
            MouseMode::Cursor => cursor::handle_drag_update(canvas),
            MouseMode::Pencil => pencil::handle_drag_update(canvas),
        }
    }

    pub fn handle_drag_end(&self, canvas: &mut Canvas) {
        match self {
            MouseMode::Cursor => cursor::handle_drag_end(canvas),
            MouseMode::Pencil => pencil::handle_drag_end(canvas),
        }
    }
}
