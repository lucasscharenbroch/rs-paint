use super::insert_shape::InsertShapeState;
use super::{FreeTransformState, MouseModeVariant};
use crate::icon_file;
use crate::image::blend::BlendingMode;
use crate::image::brush::BrushType;
use crate::transformable::Transformable;
use crate::ui::form::gadget::{NumberedSliderGadget, ToggleButtonsGadget};
use crate::ui::form::{CheckboxField, DropdownField, Form, FormBuilderIsh, NaturalField, RadioField};
use crate::ui::UiState;
use crate::shape::{ShapeType, Shape};

use std::rc::Rc;
use std::cell::RefCell;
use gtk::gdk::RGBA;
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

type MagicWandSettings = (f64, bool);
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

    let relative_tolerance = CheckboxField::new(Some("Relative Tolerance"), false);

    let form = Form::builder()
        .orientation(gtk::Orientation::Horizontal)
        .with_gadget(&*threshold_slider_gadget_p.borrow())
        .with_field(&relative_tolerance)
        .build();

    let get = move || {
        (
            threshold_slider_gadget_p.borrow().value() as f64 / 100.0,
            relative_tolerance.value(),
        )
    };

    (form, Box::new(get))
}

type FillSettings = (f64, bool);
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

    let relative_tolerance = CheckboxField::new(Some("Relative Tolerance"), false);

    let form = Form::builder()
        .orientation(gtk::Orientation::Horizontal)
        .with_gadget(&*threshold_slider_gadget_p.borrow())
        .with_field(&relative_tolerance)
        .build();

    let get = move || {
        (
            threshold_slider_gadget_p.borrow().value() as f64 / 100.0,
            relative_tolerance.value(),
        )
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

    commit_button.connect_clicked(clone!(@strong ui_p => move |_| {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().commit_and_scrap_transformable();
        }
    }));

    let commit_and_keep_button = gtk::Button::builder()
        .child(&commit_and_keep_inner)
        .width_request(75)
        .height_request(75)
        .build();

    commit_and_keep_button.connect_clicked(clone!(@strong ui_p => move |_| {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().commit_transformable();
        }
    }));

    let scrap_button = gtk::Button::builder()
        .child(&scrap_inner)
        .width_request(75)
        .height_request(75)
        .build();

    scrap_button.connect_clicked(clone!(@strong ui_p => move |_| {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().scrap_transformable();
        }
    }));

    let form = Form::builder()
        .orientation(gtk::Orientation::Horizontal)
        .with_field(&commit_and_keep_button)
        .with_field(&commit_button)
        .with_field(&scrap_button)
        .build();

    let get = move || {
        ()
    };

    (form, Box::new(get))
}

type InsertShapeSettings = (ShapeType, u8);
fn mk_insert_shape_toolbar() -> (Form, Box<dyn Fn() -> InsertShapeSettings>) {
    let shape_types = ShapeType::iter_variants().collect::<Vec<_>>();

    let drawing_areas = shape_types.iter()
        .map(|&shape_ty| {
            const HEIGHT: i32 = 30;
            const WIDTH: i32 = 30;

            let area = gtk::DrawingArea::builder()
                .width_request(HEIGHT)
                .height_request(WIDTH)
                .build();

            let black = RGBA::new(0.0, 0.0, 0.0, 1.0);
            let transparent = RGBA::new(0.0, 0.0, 0.0, 0.0);
            let mut shape = Shape::new(shape_ty, 3, black, transparent);

            area.set_draw_func(move |_, cr, width, height| {
                cr.set_line_width(0.1);
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.translate(width as f64 * 0.1, height as f64 * 0.1);
                cr.scale(width as f64 * 0.8, height as f64 * 0.8);

                shape.draw(cr, WIDTH as f64, HEIGHT as f64);
            });

            area
        })
        .collect::<Vec<_>>();

    let variants = shape_types.iter()
        .zip(drawing_areas.iter())
        .map(|(shape_ty, drawing_area)| {
            (drawing_area, *shape_ty)
        })
        .collect::<Vec<_>>();

    let shape_selector_p = ToggleButtonsGadget::new_p(
        None,
        variants,
        0,
        gtk::Orientation::Vertical,
        6,
        gtk::Orientation::Horizontal,
    );

    let border_width_field = NaturalField::new(Some("Border Width"), 1, 255, 1, 5);

    let form = Form::builder()
        .with_gadget(&*shape_selector_p.borrow())
        .with_field(&border_width_field)
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let get = move || {
        (
            *shape_selector_p.borrow().value().unwrap(),
            border_width_field.value() as u8
        )
    };

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
    insert_shape_form: Form,
    get_insert_shape_settings_p: Box<dyn Fn() -> InsertShapeSettings>,
    deferred: Option<DeferredFormsAndSettings>,
}

impl ModeToolbar {
    pub fn new(widget_wrapper: &gtk::Box, active_variant: Option<MouseModeVariant>) -> Self {
        let (pencil_form, get_pencil_settings_p) = mk_pencil_toolbar();
        let (magic_wand_form, get_magic_wand_settings_p) = mk_magic_wand_toolbar();
        let (fill_form, get_fill_settings_p) = mk_fill_toolbar();
        let (insert_shape_form, get_insert_shape_settings_p) = mk_insert_shape_toolbar();

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
            insert_shape_form,
            get_insert_shape_settings_p,
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
            MouseModeVariant::InsertShape => &self.insert_shape_form,
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

    pub fn get_insert_shape_settings(&self) -> InsertShapeSettings {
        (self.get_insert_shape_settings_p)()
    }

    pub fn init_ui_hooks(&mut self, ui_p: &Rc<RefCell<UiState>>) {
        let (free_transform_form, get_free_transform_settings_p) = mk_free_transform_toolbar(ui_p.clone());

        self.deferred = Some(DeferredFormsAndSettings {
            free_transform_form,
            get_free_transform_settings_p,
        });
    }
}
