mod cursor;
mod pencil;

use cursor::CursorState;
use pencil::PencilState;
use super::canvas::Canvas;

use gtk::gdk::{ModifierType};

#[derive(Clone, Copy)]
pub enum MouseMode {
    Cursor(cursor::CursorState),
    Pencil(pencil::PencilState),
}

trait MouseModeState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
    fn handle_drag_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
    fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
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

    fn get_state(&mut self) -> &mut dyn MouseModeState {
        match self {
            MouseMode::Cursor(ref mut s) => s,
            MouseMode::Pencil(ref mut s) => s,
        }
    }

    pub fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        self.get_state().handle_drag_start(mod_keys, canvas);
    }

    pub fn handle_drag_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        self.get_state().handle_drag_update(mod_keys, canvas);
    }

    pub fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        self.get_state().handle_drag_end(mod_keys, canvas);
    }

    pub fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        self.get_state().handle_motion(mod_keys, canvas);
    }
}
