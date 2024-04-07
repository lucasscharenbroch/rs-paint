mod cursor;
mod pencil;
mod rectangle_select;

use super::canvas::Canvas;

use cursor::CursorState;
use pencil::PencilState;
use self::rectangle_select::RectangleSelectState;
use gtk::cairo::Context;
use gtk::gdk::ModifierType;


#[derive(Clone, Copy)]
pub enum MouseMode {
    Cursor(cursor::CursorState),
    Pencil(pencil::PencilState),
    RectangleSelect(rectangle_select::RectangleSelectState),
}

trait MouseModeState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
    fn handle_drag_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
    fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
    fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas);
    fn draw(&self, canvas: &Canvas, cr: &Context);
}

impl PartialEq<MouseMode> for MouseMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MouseMode::Cursor(_), MouseMode::Cursor(_)) => true,
            (MouseMode::Pencil(_), MouseMode::Pencil(_)) => true,
            (MouseMode::RectangleSelect(_), MouseMode::RectangleSelect(_)) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl MouseMode {
    pub fn cursor(canvas: &Canvas) -> MouseMode {
        MouseMode::Cursor(CursorState::default(canvas))
    }

    pub fn cursor_default() -> MouseMode {
        MouseMode::Cursor(CursorState::default_no_canvas())
    }

    pub fn pencil(canvas: &Canvas) -> MouseMode {
        MouseMode::Pencil(PencilState::default(canvas))
    }

    pub fn pencil_default() -> MouseMode {
        MouseMode::Pencil(PencilState::default_no_canvas())
    }

    pub fn rectangle_select(canvas: &Canvas) -> MouseMode {
        MouseMode::RectangleSelect(RectangleSelectState::default(canvas))
    }

    pub fn rectangle_select_default() -> MouseMode {
        MouseMode::RectangleSelect(RectangleSelectState::default_no_canvas())
    }

    fn get_state(&mut self) -> &mut dyn MouseModeState {
        match self {
            MouseMode::Cursor(ref mut s) => s,
            MouseMode::Pencil(ref mut s) => s,
            MouseMode::RectangleSelect(ref mut s) => s,
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

    pub fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        self.get_state().handle_mod_key_update(mod_keys, canvas);
    }

    pub fn draw(&mut self, canvas: &Canvas, context: &Context) {
        self.get_state().draw(canvas, context);
    }
}
