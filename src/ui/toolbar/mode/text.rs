use crate::geometry::xywh_to_matrix_f;
use crate::image::DrawableImage;
use crate::ui::dialog::{close_dialog, no_button_dialog};

use super::{Canvas, FreeTransformState, MouseMode, MouseModeVariant, Toolbar};
use crate::ui::form::{Form, FormBuilderIsh};
use crate::transformable::Transformable;
use crate::image::undo::action::ActionName;
use crate::ui::UiState;

use std::rc::Rc;
use std::cell::RefCell;
use gtk::{gdk, pango, cairo, prelude::*, TextView};
use gdk::{ModifierType, RGBA};
use glib_macros::clone;

#[derive(Clone)]
pub enum TextState {
    /// No marker placed, but ready to place one upon a click
    Ready,
    /// Inserting text without transforming (scale to natural width/height)
    Inserting(f64, f64, Rc<dyn Fn() -> TextSpecs>), // (x, y, get_text_specs)
    /// Typing dialog is up; insert transformable @ (x, y)
    TransferToFreeTransform(f64, f64), // text origin (x, y)
}

impl TextState {
    pub fn default(_canvas: &Canvas) -> TextState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> TextState {
        TextState::Ready
    }
}

pub struct TextSpecs {
    text: String,
    font_face_option: Option<cairo::FontFace>,
    font_size: i32,
}

impl TextSpecs {
    fn new(text: String, font_face_option: Option<cairo::FontFace>, font_size: i32) -> Self {
        TextSpecs {
            text,
            font_face_option,
            font_size,
        }
    }

    fn try_font_face(&self) -> &Option<cairo::FontFace> {
        &self.font_face_option
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn calc_natural_wh(&self) -> (f64, f64) {
        // make a dummy surface and context for calculating the text extents

        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            1,
            1,
        ).unwrap();

        let cr = cairo::Context::new(&surface).unwrap();

        let (widths_and_bearings, heights_and_bearings): (Vec<(f64, f64)>, Vec<(f64, f64)>) = self.text().lines()
            .map(|line| if line.len() == 0 { "_" } else { line }) // give blank-lines the width/height of "_"
            .map(|line| cr.text_extents(line))
            .map(|e| {
                e.map(|extents| (
                    (extents.width(), extents.x_bearing()),
                    (extents.height(), extents.y_bearing()),
                ))
                    .unwrap_or(((0.0, 0.0), (0.0, 0.0)))
            })
            .unzip();

        // determine the effective height/width of the text
        let net_width = widths_and_bearings.iter()
            .map(|(width, _bearing)| *width)
            .max_by(|a, b| {
                a.partial_cmp(b).unwrap()
            }).unwrap_or(0.0);
        let net_height: f64 = heights_and_bearings.iter()
            .map(|(height, _bearing)| *height)
            .sum::<f64>();

        fn font_size_scale_fn(x: i32) -> f64 {
            // x is in "points"
            x as f64 / 12276.0
        }

        let mult = font_size_scale_fn(self.font_size);

        (net_width * mult, net_height * mult)
    }
}

fn mk_text_insertion_dialog(ui_p: &Rc<RefCell<UiState>>) -> (Form, Rc<dyn Fn() -> TextSpecs>) {
    let text_box = gtk::TextView::builder()
        .width_request(300)
        .css_classes(["text-tool-entry"])
        .build();

    text_box.buffer().set_text("Type Text Here");
    text_box.emit_select_all(true);

    text_box.buffer().connect_changed(clone!(@strong ui_p => move |_buffer| {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().update();
        }
    }));

    let font_dialog = gtk::FontDialog::builder()
        .language(&pango::Language::default())
        .build();

    let font_button = gtk::FontDialogButton::builder()
        .dialog(&font_dialog)
        .level(gtk::FontLevel::Font)
        .build();

    font_button.connect_font_desc_notify(clone!(@strong ui_p => move |_| {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().update();
        }
    }));

    let form = Form::builder()
        .with_field(&font_button)
        .with_focused_field(&text_box)
        .build();

    let get = move || {
        fn string_from_text_view(text_view: &gtk::TextView) -> String {
            let buffer = text_view.buffer();
            buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).into()
        }

        TextSpecs::new(
            string_from_text_view(&text_box),
            font_button.font_desc().and_then(|desc| {
                desc.family().map(|family| {
                    cairo::FontFace::toy_create(family.as_str(), cairo::FontSlant::Normal, cairo::FontWeight::Normal)
                        .unwrap()
                })
            }),
            font_button.font_desc().map(|desc| {
                desc.size()
            }).unwrap_or(12),
        )
    };

    (form, Rc::new(get))
}

#[derive(Clone)]
struct TransformableText {
    get_text_specs: Rc<dyn Fn() -> TextSpecs>,
    color: RGBA,
}

impl Transformable for TransformableText {
    fn draw(&mut self, cr: &gtk::cairo::Context, pixel_width: f64, pixel_height: f64) {
        let text_specs = (*self.get_text_specs)();
        cr.set_source_rgba(
            self.color.red() as f64,
            self.color.green() as f64,
            self.color.blue() as f64,
            self.color.alpha() as f64,
        );
        if let Some(font_face) = text_specs.try_font_face() {
            cr.set_font_face(&font_face);
        }

        let (widths_and_bearings, heights_and_bearings): (Vec<(f64, f64)>, Vec<(f64, f64)>) = text_specs.text().lines()
            .map(|line| if line.len() == 0 { "_" } else { line }) // give blank-lines the width/height of "_"
            .map(|line| cr.text_extents(line))
            .map(|e| {
                e.map(|extents| (
                    (extents.width(), extents.x_bearing()),
                    (extents.height(), extents.y_bearing()),
                ))
                    .unwrap_or(((0.0, 0.0), (0.0, 0.0)))
            })
            .unzip();

        // determine the effective height/width of the text
        let net_width = widths_and_bearings.iter()
            .map(|(width, _bearing)| *width)
            .max_by(|a, b| {
                a.partial_cmp(b).unwrap()
            }).unwrap_or(0.0);
        let net_height: f64 = heights_and_bearings.iter()
            .map(|(height, _bearing)| *height)
            .sum::<f64>();

        let _ = cr.save();
        {
            const EPSILON: f64 = 1e-6; // prevent from scaling to zero
            cr.scale(1.0 / net_width.max(EPSILON), 1.0 / net_height.max(EPSILON));

            text_specs.text().lines().zip(heights_and_bearings)
                .zip(widths_and_bearings)
                .for_each(|((line, (height, y_bearing)), (_width, x_bearing))| {
                cr.translate(-x_bearing, -y_bearing);
                // The following sometimes errors when highly zoomed in.
                // It might just be my system.
                // Not sure how to fix it, so we'll turn out backs as the world burns.
                let _ = cr.show_text(line);
                cr.translate(x_bearing, y_bearing + height);
                cr.new_path();
            });
        }
        let _ = cr.restore();
    }
}

impl super::MouseModeState for TextState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        match self {
            Self::Ready => {
                let (form, get_text_specs) = mk_text_insertion_dialog(canvas.ui_p());

                no_button_dialog(
                    canvas.ui_p().borrow().window(),
                    "Add Text",
                    form.widget(),
                ).grab_focus();

                let (cx, cy) = canvas.cursor_pos_pix_f();

                let transformable = TransformableText {
                    get_text_specs: get_text_specs.clone(),
                    color: toolbar.primary_color(),
                };

                toolbar.set_boxed_transformable(Box::new(transformable));

                *self = Self::Inserting(cx, cy, get_text_specs);
                canvas.update();
            },
            Self::Inserting(x, y, get_text_specs) => {
                let (cx, cy) = canvas.cursor_pos_pix_f();
                *self = Self::Inserting(cx, cy, get_text_specs.clone());
                canvas.update();
            },
            _ => (),
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

    fn handle_close(&self, canvas: &mut Canvas, toolbar: &Toolbar, new_mode: &MouseMode) {
        if let Some(transformable) = toolbar.try_take_boxed_transformable() {
            if new_mode.variant() == MouseModeVariant::FreeTransform {
                if let Self::Inserting(x, y, get_text_specs) = self {
                    let (w, h) = get_text_specs().calc_natural_wh();
                    let matrix = xywh_to_matrix_f(*x, *y, w, h);
                    let _ = canvas.try_give_transformable(transformable, matrix);
                }
            }
        }
    }

    fn draw(&self, _canvas: &Canvas, cr: &cairo::Context, toolbar: &mut Toolbar) {

        // visual cue for text origin at (0, 0)
        fn draw_origin_point(cr: &cairo::Context) {
            cr.set_source_rgb(0.0, 0.0, 0.0);

            let origin_offset = 2.0;
            let line_length = 10.0;

            cr.move_to(-origin_offset, -origin_offset);
            cr.line_to(-origin_offset + line_length, -origin_offset);
            cr.move_to(-origin_offset, -origin_offset);
            cr.line_to(-origin_offset, -origin_offset + line_length);
            let _ = cr.stroke();
        }

        match self {
            Self::Inserting(x, y, get_text_specs) => {
                let text_specs = get_text_specs();
                let (w, h) = text_specs.calc_natural_wh();

                let _ = cr.save();
                {
                    cr.translate(*x, *y);
                    draw_origin_point(cr);
                    cr.scale(w, h);
                    if let Some(transformable) = toolbar.get_boxed_transformable().as_mut() {
                        transformable.draw(cr, w, h);
                    } else {
                    }
                }
                let _ = cr.restore();
            },
            _ => (),
        }
    }
}
