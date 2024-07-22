use crate::ui::dialog::close_dialog;

use super::{Canvas, FreeTransformState, MouseMode, Toolbar};
use crate::ui::form::{Form, FormBuilderIsh};

use gtk::{gdk::ModifierType, prelude::*, FontChooserLevel, pango};

#[derive(Clone, Copy)]
pub enum TextState {
    /// No marker placed, but ready to place one upon a click
    Ready,
    /// Typing dialog is up; insert transformable @ (x, y)
    TransferToFreeTransform(f64, f64), // (x, y)
}

impl TextState {
    pub fn default(_canvas: &Canvas) -> TextState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> TextState {
        TextState::Ready
    }
}

type TextSettings = ();
fn mk_text_insertion_dialog() -> (Form, Box<dyn Fn() -> TextSettings>) {
    let font_dialog = gtk::FontDialog::builder()
        .language(&pango::Language::default())
        .build();

    let font_button = gtk::FontDialogButton::builder()
        .dialog(&font_dialog)
        .build();

    let form = Form::builder()
        .with_field(&font_button)
        .build();

    let get = || {
    };

    (form, Box::new(get))
}

impl super::MouseModeState for TextState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let Self::Ready = self {
            let (form, get_text_settings) = mk_text_insertion_dialog();

            close_dialog(
                canvas.ui_p().borrow().window(),
                "Add Text",
                form.widget(),
                || crate::ui::dialog::CloseDialog::Yes,
                || (),
            );

            let (cx, cy) = canvas.cursor_pos_pix_f();
            *self = Self::TransferToFreeTransform(cx, cy);
        }
    }

    fn try_transfer(&self) -> Result<MouseMode, ()> {
        if let Self::TransferToFreeTransform(x, y) = self {
            Ok(MouseMode::FreeTransform(
                FreeTransformState::from_coords(*x, *y)
            ))
        } else {
            Err(())
        }
    }
}
