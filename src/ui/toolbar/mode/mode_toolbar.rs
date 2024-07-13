use super::{FreeTransformState, MouseModeVariant};
use crate::icon_file;
use crate::image::blend::BlendingMode;
use crate::image::brush::BrushType;
use crate::ui::form::gadget::NumberedSliderGadget;
use crate::ui::form::{DropdownField, Form, FormBuilderIsh, NaturalField};
use crate::ui::UiState;

use std::rc::Rc;
use std::cell::RefCell;
use gtk::prelude::*;
use glib_macros::clone;

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

type FreeTransformSettings = ();
fn mk_free_transform_toolbar(ui_p: Rc<RefCell<UiState>>) -> (Form, Box<dyn Fn() -> FreeTransformSettings>) {
    let commit_image = gtk::Image::builder()
        .file(icon_file!("checkmark"))
        .vexpand(true)
        .build();

    let commit_and_keep_image = gtk::Image::builder()
        .file(icon_file!("dotted-checkmark"))
        .vexpand(true)
        .build();

    let scrap_image = gtk::Image::builder()
        .file(icon_file!("big-red-x"))
        .vexpand(true)
        .build();

    let commit_inner = gtk::Box::new(gtk::Orientation::Vertical, 4);
    commit_inner.append(&commit_image);
    commit_inner.append(&gtk::Label::new(Some("Commit")));

    let commit_and_keep_inner = gtk::Box::new(gtk::Orientation::Vertical, 4);
    commit_and_keep_inner.append(&commit_and_keep_image);
    commit_and_keep_inner.append(&gtk::Label::new(Some("Commit and Keep")));

    let scrap_inner = gtk::Box::new(gtk::Orientation::Vertical, 4);
    scrap_inner.append(&scrap_image);
    scrap_inner.append(&gtk::Label::new(Some("Scrap")));

    let commit_button = gtk::Button::builder()
        .child(&commit_inner)
        .width_request(75)
        .height_request(75)
        .build();

    let commit_and_keep_button = gtk::Button::builder()
        .child(&commit_and_keep_inner)
        .width_request(75)
        .height_request(75)
        .build();

    let scrap_button = gtk::Button::builder()
        .child(&scrap_inner)
        .width_request(75)
        .height_request(75)
        .build();

    let form = Form::builder()
        .orientation(gtk::Orientation::Horizontal)
        .with_field(&commit_and_keep_button)
        .with_field(&commit_button)
        .with_field(&scrap_button)
        .build();

    let get = move || {
        ()
    };

    scrap_button.connect_clicked(clone!(@strong ui_p => move |_| {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().scrap_transformable();
        }
    }));

    (form, Box::new(get))
}

/// Contains members of `ModeToolbars` whose construction
/// must be deferred until there's access to a `ui_p`
struct DeferredFormsAndSettings {
    free_transform_form: Form,
    get_free_transform_settings_p: Box<dyn Fn() -> FreeTransformSettings>,
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
    deferred: Option<DeferredFormsAndSettings>,
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
            deferred: None,
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
            MouseModeVariant::FreeTransform => &self.deferred.as_ref().unwrap().free_transform_form,
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

    pub fn get_free_transform_settings(&self) -> FreeTransformSettings {
        (self.deferred.as_ref().unwrap().get_free_transform_settings_p)()
    }

    pub fn init_ui_hooks(&mut self, ui_p: &Rc<RefCell<UiState>>) {
        let (free_transform_form, get_free_transform_settings_p) = mk_free_transform_toolbar(ui_p.clone());

        self.deferred = Some(DeferredFormsAndSettings {
            free_transform_form,
            get_free_transform_settings_p,
        });
    }
}
