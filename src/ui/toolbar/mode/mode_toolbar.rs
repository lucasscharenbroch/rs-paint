use gtk::glib::object::IsA;
use gtk::prelude::*;

use super::MouseModeVariant;
use super::{CursorState, MagicWandState, PencilState, EyedropperState, RectangleSelectState};
use crate::ui::form::Form;

type PencilSettings = ();
fn mk_pencil_toolbar() -> (Form, Box<dyn Fn() -> PencilSettings>) {
    let form = Form::builder()
        .build();

    (
        form,
        Box::new(|| ())
    )
}

pub struct ModeToolbar {
    active_variant: Option<MouseModeVariant>,
    widget_wrapper: gtk::Box,
    empty_form: Form,
    pencil_form: Form,
    get_pencil_settings: Box<dyn Fn() -> PencilSettings>,
}

impl ModeToolbar {
    pub fn new(widget_wrapper: &gtk::Box) -> Self {
        let (pencil_form, get_pencil_settings) = mk_pencil_toolbar();

        ModeToolbar {
            active_variant: None,
            widget_wrapper: widget_wrapper.clone(),
            empty_form: Form::builder().build(),
            pencil_form,
            get_pencil_settings,
        }
    }

    fn variant_to_form(&self, variant: &MouseModeVariant) -> &Form {
        match variant {
            MouseModeVariant::Cursor => &self.empty_form,
            MouseModeVariant::Eyedropper => &self.empty_form,
            MouseModeVariant::MagicWand => &self.empty_form,
            MouseModeVariant::Pencil => &self.pencil_form,
            MouseModeVariant::RectangleSelect => &self.empty_form,
        }
    }

    pub fn active_form(&self) -> Option<&Form> {
        self.active_variant.as_ref().map(|variant| {
            self.variant_to_form(variant)
        })
    }

    pub fn set_to_variant(&mut self, variant: MouseModeVariant) {
        if let Some(form) = self.active_form() {
            self.widget_wrapper.remove(form.widget());
        }

        self.widget_wrapper.append(self.variant_to_form(&variant).widget());
        self.active_variant = Some(variant);
    }

    pub fn get_pencil_settings(&self) -> PencilSettings {
        self.get_pencil_settings()
    }
}
