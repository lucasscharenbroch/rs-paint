use super::MouseModeVariant;
use crate::image::blend::BlendingMode;
use crate::image::brush::BrushType;
use crate::ui::form::gadget::NumberedSliderGadget;
use crate::ui::form::{DropdownField, Form, FormBuilderIsh, NaturalField};

use gtk::prelude::*;

type PencilSettings = (BrushType, BlendingMode, u8);
fn mk_pencil_toolbar() -> (Form, Box<dyn Fn() -> PencilSettings>) {
    let brush_types = vec![
        ("Round", BrushType::Round),
        ("Square", BrushType::Square),
        ("Caligraphy", BrushType::Caligraphy),
        ("Dither", BrushType::Dither),
        ("Pen", BrushType::Pen),
        ("Crayon", BrushType::Crayon),
    ];

    let blending_modes = vec![
        ("Overwrite", BlendingMode::Overwrite),
        ("Paint", BlendingMode::Paint),
        ("Average", BlendingMode::Average),
    ];

    let type_dropdown = DropdownField::new(Some("Brush Type"), brush_types, 0);
    type_dropdown.set_orientation(gtk::Orientation::Vertical);
    let blending_mode_dropdown = DropdownField::new(Some("Blending Mode"), blending_modes, 0);
    blending_mode_dropdown.set_orientation(gtk::Orientation::Vertical);
    let radius_selector = NaturalField::new(Some("Brush Radius"), 1, 255, 1, 5);
    radius_selector.set_orientation(gtk::Orientation::Vertical);

    let form = Form::builder()
        .orientation(gtk::Orientation::Horizontal)
        .with_field(&type_dropdown)
        .with_field(&blending_mode_dropdown)
        .with_field(&radius_selector)
        .spacing(20)
        .build();

    let get = move || {
        (
            type_dropdown.value().clone(),
            blending_mode_dropdown.value().clone(),
            radius_selector.value() as u8,
        )
    };

    (form, Box::new(get))
}

type MagicWandSettings = f64;
fn mk_magic_wand_toolbar() -> (Form, Box<dyn Fn() -> MagicWandSettings>) {
    let threshold_slider_gadget_p = NumberedSliderGadget::new_p(
        Some("Tolerance"),
        gtk::Orientation::Horizontal,
        true,
        0,
        100,
        1,
        50,
        String::from("%"),
    );

    let form = Form::builder()
        .with_gadget(&*threshold_slider_gadget_p.borrow())
        .build();

    let get = move || {
        threshold_slider_gadget_p.borrow().value() as f64 / 100.0
    };

    (form, Box::new(get))
}

type FillSettings = f64;
fn mk_fill_toolbar() -> (Form, Box<dyn Fn() -> FillSettings>) {
    let threshold_slider_gadget_p = NumberedSliderGadget::new_p(
        Some("Tolerance"),
        gtk::Orientation::Horizontal,
        true,
        0,
        100,
        1,
        50,
        String::from("%"),
    );

    let form = Form::builder()
        .with_gadget(&*threshold_slider_gadget_p.borrow())
        .build();

    let get = move || {
        threshold_slider_gadget_p.borrow().value() as f64 / 100.0
    };

    (form, Box::new(get))
}

pub struct ModeToolbar {
    active_variant: Option<MouseModeVariant>,
    widget_wrapper: gtk::Box,
    empty_form: Form,
    pencil_form: Form,
    get_pencil_settings_p: Box<dyn Fn() -> PencilSettings>,
    magic_wand_form: Form,
    get_magic_wand_settings_p: Box<dyn Fn() -> MagicWandSettings>,
    fill_form: Form,
    get_fill_settings_p: Box<dyn Fn() -> FillSettings>,
}

impl ModeToolbar {
    pub fn new(widget_wrapper: &gtk::Box, active_variant: Option<MouseModeVariant>) -> Self {
        let (pencil_form, get_pencil_settings_p) = mk_pencil_toolbar();
        let (magic_wand_form, get_magic_wand_settings_p) = mk_magic_wand_toolbar();
        let (fill_form, get_fill_settings_p) = mk_fill_toolbar();

        let mut res = ModeToolbar {
            active_variant: None,
            widget_wrapper: widget_wrapper.clone(),
            empty_form: Form::builder().build(),
            pencil_form,
            get_pencil_settings_p,
            magic_wand_form,
            get_magic_wand_settings_p,
            fill_form,
            get_fill_settings_p,
        };

        active_variant.map(|v| res.set_to_variant(v));
        res
    }

    fn variant_to_form(&self, variant: &MouseModeVariant) -> &Form {
        match variant {
            MouseModeVariant::Cursor => &self.empty_form,
            MouseModeVariant::Eyedropper => &self.empty_form,
            MouseModeVariant::MagicWand => &self.magic_wand_form,
            MouseModeVariant::Pencil => &self.pencil_form,
            MouseModeVariant::RectangleSelect => &self.empty_form,
            MouseModeVariant::Fill => &self.fill_form,
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
        (self.get_pencil_settings_p)()
    }

    pub fn get_magic_wand_settings(&self) -> MagicWandSettings {
        (self.get_magic_wand_settings_p)()
    }

    pub fn get_fill_settings(&self) -> MagicWandSettings {
        (self.get_fill_settings_p)()
    }
}
