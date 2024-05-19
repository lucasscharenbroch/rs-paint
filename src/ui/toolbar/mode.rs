mod cursor;
mod pencil;
mod rectangle_select;

use crate::ui::{canvas::Canvas, toolbar::Toolbar};

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

#[derive(PartialEq)]
pub enum MouseModeVariant {
    Cursor,
    Pencil,
    RectangleSelect,
}

trait MouseModeState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar);
    fn handle_drag_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar);
    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar);
    fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar);
    fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar);
    fn draw(&self, canvas: &Canvas, cr: &Context);
}

impl MouseMode {
    pub fn cursor(canvas: &Canvas) -> MouseMode {
        MouseMode::Cursor(CursorState::default(canvas))
    }

    pub const fn cursor_default() -> MouseMode {
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

    fn get_state_immutable(&self) -> &dyn MouseModeState {
        match self {
            MouseMode::Cursor(ref s) => s,
            MouseMode::Pencil(ref s) => s,
            MouseMode::RectangleSelect(ref s) => s,
        }
    }

    pub fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_drag_start(mod_keys, canvas, toolbar);
    }

    pub fn handle_drag_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_drag_update(mod_keys, canvas, toolbar);
    }

    pub fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_drag_end(mod_keys, canvas, toolbar);
    }

    pub fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_motion(mod_keys, canvas, toolbar);
    }

    pub fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_mod_key_update(mod_keys, canvas, toolbar);
    }

    pub fn draw(&self, canvas: &Canvas, context: &Context) {
        self.get_state_immutable().draw(canvas, context);
    }

    pub fn variant(&self) -> MouseModeVariant {
        match self {
            MouseMode::Cursor(_) => MouseModeVariant::Cursor,
            MouseMode::Pencil(_) => MouseModeVariant::Pencil,
            MouseMode::RectangleSelect(_) => MouseModeVariant::RectangleSelect,
        }
    }
}
