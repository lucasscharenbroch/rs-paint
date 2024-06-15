mod mode_toolbar;
mod rectangle_select;
mod cursor;
mod pencil;
mod eyedropper;
mod magic_wand;

use crate::ui::form::Form;
use crate::ui::{canvas::Canvas, toolbar::Toolbar};
pub use mode_toolbar::ModeToolbar;

use cursor::CursorState;
use magic_wand::MagicWandState;
use pencil::PencilState;
use self::eyedropper::EyedropperState;
pub use self::rectangle_select::RectangleSelectState;

use gtk::cairo::Context;
use gtk::gdk::ModifierType;

#[derive(Clone, Copy)]
pub enum MouseMode {
    Cursor(cursor::CursorState),
    Pencil(pencil::PencilState),
    RectangleSelect(rectangle_select::RectangleSelectState),
    Eyedropper(eyedropper::EyedropperState),
    MagicWand(magic_wand::MagicWandState),
}

#[derive(PartialEq)]
pub enum MouseModeVariant {
    Cursor,
    Pencil,
    RectangleSelect,
    Eyedropper,
    MagicWand,
}

trait MouseModeState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_drag_end(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_motion(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_mod_key_update(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn draw(&self, _canvas: &Canvas, _cr: &Context) {}
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

    pub fn eyedropper(canvas: &Canvas) -> MouseMode {
        MouseMode::Eyedropper(EyedropperState::default(canvas))
    }

    pub fn eyedropper_default() -> MouseMode {
        MouseMode::Eyedropper(EyedropperState::default_no_canvas())
    }

    pub fn magic_wand(canvas: &Canvas) -> MouseMode {
        MouseMode::MagicWand(MagicWandState::default(canvas))
    }

    pub fn magic_wand_default() -> MouseMode {
        MouseMode::MagicWand(MagicWandState::default_no_canvas())
    }

    fn get_state(&mut self) -> &mut dyn MouseModeState {
        match self {
            MouseMode::Cursor(ref mut s) => s,
            MouseMode::Pencil(ref mut s) => s,
            MouseMode::RectangleSelect(ref mut s) => s,
            MouseMode::Eyedropper(ref mut s) => s,
            MouseMode::MagicWand(ref mut s) => s,
        }
    }

    fn get_state_immutable(&self) -> &dyn MouseModeState {
        match self {
            MouseMode::Cursor(ref s) => s,
            MouseMode::Pencil(ref s) => s,
            MouseMode::RectangleSelect(ref s) => s,
            MouseMode::Eyedropper(ref s) => s,
            MouseMode::MagicWand(ref s) => s,
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
            MouseMode::Eyedropper(_) => MouseModeVariant::Eyedropper,
            MouseMode::MagicWand(_) => MouseModeVariant::MagicWand,
        }
    }
}
