mod mode_toolbar;
mod rectangle_select;
mod cursor;
mod pencil;
mod eyedropper;
mod magic_wand;
mod fill;
mod free_transform;
mod shape;
mod text;

use crate::ui::{canvas::Canvas, toolbar::Toolbar};
pub use mode_toolbar::ModeToolbar;

pub use cursor::CursorState;
use magic_wand::MagicWandState;
use pencil::PencilState;
use fill::FillState;
use self::eyedropper::EyedropperState;
pub use self::rectangle_select::{RectangleSelectState, RectangleSelectMode};
pub use free_transform::{FreeTransformState, TransformationSelection};
use shape::ShapeState;
use text::TextState;

use gtk::cairo::Context;
use gtk::gdk::ModifierType;

#[derive(Clone)]
pub enum MouseMode {
    Cursor(CursorState),
    Pencil(PencilState),
    RectangleSelect(RectangleSelectState),
    Eyedropper(EyedropperState),
    MagicWand(MagicWandState),
    Fill(FillState),
    FreeTransform(FreeTransformState),
    Shape(ShapeState),
    Text(TextState)
}

#[derive(PartialEq, Clone, Copy)]
pub enum MouseModeVariant {
    Cursor,
    Pencil,
    RectangleSelect,
    Eyedropper,
    MagicWand,
    Fill,
    FreeTransform,
    Shape,
    Text,
}

trait MouseModeState {
    // left-click drags
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_drag_end(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}

    // right-click drags
    fn handle_right_drag_start(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_right_drag_update(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_right_drag_end(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}

    fn handle_motion(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn handle_mod_key_update(&mut self, _mod_keys: &ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {}
    fn draw(&self, _canvas: &Canvas, _cr: &Context, _toolbar: &mut Toolbar) {}

    /// Hack to transfer from one MouseMode to another
    /// within a handler (which only allow homogeneous mutation)
    /// (call this after each handler)
    /// Err(()) => don't transfer
    fn try_transfer(&self) -> Result<MouseMode, ()> {
        Err(())
    }

    /// Called when the mode is changed away (useful for cleaning up
    /// any changes to the canvas state, e.g. the cursor visual)
    fn handle_close(&self, canvas: &mut Canvas, toolbar: &Toolbar, new_mode: &MouseMode) {}
    fn handle_selection_deleted(&mut self) {}
}

impl MouseMode {
    pub fn cursor(canvas: &mut Canvas) -> MouseMode {
        MouseMode::Cursor(CursorState::default(canvas))
    }

    pub const fn cursor_default() -> MouseMode {
        MouseMode::Cursor(CursorState::default_no_canvas())
    }

    pub fn pencil(canvas: &mut Canvas) -> MouseMode {
        MouseMode::Pencil(PencilState::default(canvas))
    }

    pub fn pencil_default() -> MouseMode {
        MouseMode::Pencil(PencilState::default_no_canvas())
    }

    pub fn rectangle_select(canvas: &mut Canvas) -> MouseMode {
        MouseMode::RectangleSelect(RectangleSelectState::default(canvas))
    }

    pub fn rectangle_select_default() -> MouseMode {
        MouseMode::RectangleSelect(RectangleSelectState::default_no_canvas())
    }

    pub fn eyedropper(canvas: &mut Canvas) -> MouseMode {
        MouseMode::Eyedropper(EyedropperState::default(canvas))
    }

    pub fn eyedropper_default() -> MouseMode {
        MouseMode::Eyedropper(EyedropperState::default_no_canvas())
    }

    pub fn magic_wand(canvas: &mut Canvas) -> MouseMode {
        MouseMode::MagicWand(MagicWandState::default(canvas))
    }

    pub fn magic_wand_default() -> MouseMode {
        MouseMode::MagicWand(MagicWandState::default_no_canvas())
    }

    pub fn fill(canvas: &mut Canvas) -> MouseMode {
        MouseMode::Fill(FillState::default(canvas))
    }

    pub fn fill_default() -> MouseMode {
        MouseMode::Fill(FillState::default_no_canvas())
    }

    pub fn free_transform(canvas: &mut Canvas) -> MouseMode {
        MouseMode::FreeTransform(FreeTransformState::default(canvas))
    }

    pub fn free_transform_default() -> MouseMode {
        MouseMode::FreeTransform(FreeTransformState::default_no_canvas())
    }

    pub fn shape(canvas: &mut Canvas) -> MouseMode {
        MouseMode::Shape(ShapeState::default(canvas))
    }

    pub fn shape_default() -> MouseMode {
        MouseMode::Shape(ShapeState::default_no_canvas())
    }

    pub fn text(canvas: &mut Canvas) -> MouseMode {
        MouseMode::Text(TextState::default(canvas))
    }

    pub fn text_default() -> MouseMode {
        MouseMode::Text(TextState::default_no_canvas())
    }

    fn get_state(&mut self) -> &mut dyn MouseModeState {
        match self {
            MouseMode::Cursor(ref mut s) => s,
            MouseMode::Pencil(ref mut s) => s,
            MouseMode::RectangleSelect(ref mut s) => s,
            MouseMode::Eyedropper(ref mut s) => s,
            MouseMode::MagicWand(ref mut s) => s,
            MouseMode::Fill(ref mut s) => s,
            MouseMode::FreeTransform(ref mut s) => s,
            MouseMode::Shape(ref mut s) => s,
            MouseMode::Text(ref mut s) => s,
        }
    }

    fn get_state_immutable(&self) -> &dyn MouseModeState {
        match self {
            MouseMode::Cursor(ref s) => s,
            MouseMode::Pencil(ref s) => s,
            MouseMode::RectangleSelect(ref s) => s,
            MouseMode::Eyedropper(ref s) => s,
            MouseMode::MagicWand(ref s) => s,
            MouseMode::Fill(ref s) => s,
            MouseMode::FreeTransform(ref s) => s,
            MouseMode::Shape(ref s) => s,
            MouseMode::Text(ref s) => s,
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

    pub fn handle_right_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_right_drag_start(mod_keys, canvas, toolbar);
    }

    pub fn handle_right_drag_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_right_drag_update(mod_keys, canvas, toolbar);
    }

    pub fn handle_right_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_right_drag_end(mod_keys, canvas, toolbar);
    }

    pub fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_motion(mod_keys, canvas, toolbar);
    }

    pub fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.get_state().handle_mod_key_update(mod_keys, canvas, toolbar);
    }

    pub fn draw(&self, canvas: &Canvas, context: &Context, toolbar: &mut Toolbar) {
        self.get_state_immutable().draw(canvas, context, toolbar);
    }

    pub fn variant(&self) -> MouseModeVariant {
        match self {
            MouseMode::Cursor(_) => MouseModeVariant::Cursor,
            MouseMode::Pencil(_) => MouseModeVariant::Pencil,
            MouseMode::RectangleSelect(_) => MouseModeVariant::RectangleSelect,
            MouseMode::Eyedropper(_) => MouseModeVariant::Eyedropper,
            MouseMode::MagicWand(_) => MouseModeVariant::MagicWand,
            MouseMode::Fill(_) => MouseModeVariant::Fill,
            MouseMode::FreeTransform(_) => MouseModeVariant::FreeTransform,
            MouseMode::Shape(_) => MouseModeVariant::Shape,
            MouseMode::Text(_) => MouseModeVariant::Text,
        }
    }

    pub fn disable_when_locked(&self) -> bool {
        match self {
            MouseMode::Cursor(_) => false,
            MouseMode::Pencil(_) => true,
            MouseMode::RectangleSelect(_) => false,
            MouseMode::Eyedropper(_) => false,
            MouseMode::MagicWand(_) => false,
            MouseMode::Fill(_) => true,
            MouseMode::FreeTransform(_) => true,
            MouseMode::Shape(_) => true,
            MouseMode::Text(_) => true,
        }
    }

    pub fn from_variant(variant: MouseModeVariant, canvas: &mut Canvas) -> Self {
        match variant {
            MouseModeVariant::Cursor => Self::cursor(canvas),
            MouseModeVariant::Pencil => Self::pencil(canvas),
            MouseModeVariant::RectangleSelect => Self::rectangle_select(canvas),
            MouseModeVariant::Eyedropper => Self::eyedropper(canvas),
            MouseModeVariant::MagicWand => Self::magic_wand(canvas),
            MouseModeVariant::Fill => Self::fill(canvas),
            MouseModeVariant::FreeTransform => Self::free_transform(canvas),
            MouseModeVariant::Shape => Self::shape(canvas),
            MouseModeVariant::Text => Self::text(canvas),
        }
    }

    pub fn updated_after_hook(self) -> Self {
        match self.get_state_immutable().try_transfer() {
            Ok(new_mode) => new_mode,
            _ => self,
        }
    }

    pub fn handle_close(&self, canvas: &mut Canvas, toolbar: &Toolbar, new_mode: &MouseMode) {
        self.get_state_immutable().handle_close(canvas, toolbar, new_mode);
    }

    pub fn handle_selection_deleted(&mut self) {
        self.get_state().handle_selection_deleted();
    }
}
