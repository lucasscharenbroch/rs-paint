use crate::ui::dialog::close_dialog;

use super::{Canvas, FreeTransformState, MouseMode, Toolbar};
use crate::ui::form::{Form, FormBuilderIsh};
use crate::transformable::Transformable;
use crate::image::undo::action::ActionName;
use crate::ui::UiState;

use std::rc::Rc;
use std::cell::RefCell;
use gtk::{gdk, pango, cairo, prelude::*, TextView};
use gdk::{ModifierType, RGBA};
use glib_macros::clone;

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

type TextSpecs = (String, Option<cairo::FontOptions>);
fn mk_text_insertion_dialog(ui_p: &Rc<RefCell<UiState>>) -> (Form, Rc<dyn Fn() -> TextSpecs>) {
    let text_box = gtk::TextView::builder()
        .build();

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
        .level(gtk::FontLevel::Family)
        .build();

    let form = Form::builder()
        .with_field(&font_button)
        .with_field(&text_box)
        .build();

    let get = move || {
        fn string_from_text_view(text_view: &gtk::TextView) -> String {
            let buffer = text_view.buffer();
            buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).into()
        }

        (
            string_from_text_view(&text_box),
            font_button.font_options()
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
        let (text, font_options) = (*self.get_text_specs)();
        cr.set_source_rgba(
            self.color.red() as f64,
            self.color.green() as f64,
            self.color.blue() as f64,
            self.color.alpha() as f64,
        );
        if let Some(font_options) = font_options {
            cr.set_font_options(&font_options);
        }

        let (widths_and_bearings, heights_and_bearings): (Vec<(f64, f64)>, Vec<(f64, f64)>) = text.lines()
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
            .map(|(width, bearing)| *width - *bearing)
            .max_by(|a, b| {
                a.partial_cmp(b).unwrap()
            }).unwrap_or(0.0);
        let net_height: f64 = heights_and_bearings.iter()
            .map(|(height, bearing)| height)
            .sum::<f64>();

        let _ = cr.save();
        {
            const EPSILON: f64 = 1e-6; // prevent from scaling to zero
            cr.scale(1.0 / net_width.max(EPSILON), 1.0 / net_height.max(EPSILON));

            text.lines().zip(heights_and_bearings)
                .zip(widths_and_bearings)
                .for_each(|((line, (height, y_bearing)), (width, x_bearing))| {
                cr.translate(-x_bearing, -y_bearing);
                let r = cr.show_text(line);
                cr.translate(x_bearing, y_bearing + height);
                cr.new_path();
            });
        }
        let _ = cr.restore();
    }

    fn gen_sampleable(&mut self, pixel_width: f64, pixel_height: f64) -> Box<dyn crate::transformable::Samplable> {
        todo!()
    }

    fn try_image_ref(&self) -> Option<&crate::image::Image> {
        None
    }
}

impl super::MouseModeState for TextState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        if let Self::Ready = self {
            let (form, get_text_specs) = mk_text_insertion_dialog(canvas.ui_p());

            close_dialog(
                canvas.ui_p().borrow().window(),
                "Add Text",
                form.widget(),
                || crate::ui::dialog::CloseDialog::Yes,
                || (),
            );

            let (cx, cy) = canvas.cursor_pos_pix_f();

            let transformable = TransformableText {
                get_text_specs,
                color: toolbar.primary_color(),
            };

            let mut matrix = cairo::Matrix::identity();
            matrix.translate(cx, cy);

            *canvas.transformation_selection().borrow_mut() = Some(super::TransformationSelection::new(
                Box::new(transformable),
                matrix,
                ActionName::InsertShape,
            ));

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