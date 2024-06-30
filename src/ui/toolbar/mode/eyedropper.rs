use super::{Canvas, Toolbar, MouseModeVariant};
use crate::ui::form::Form;
use crate::image::ImageLike;

use gtk::gdk::ModifierType;
use gtk::cairo::Context;

#[derive(Clone, Copy)]
pub struct EyedropperState {
}

impl EyedropperState {
    pub fn default(_canvas: &Canvas) -> Self {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> Self {
        EyedropperState {
        }
    }
}

impl super::MouseModeState for EyedropperState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let (x, y) = canvas.cursor_pos_pix_u();
        if let Some(pix) = canvas.image().try_pix_at(y as i32, x as i32) {
            if mod_keys.contains(ModifierType::CONTROL_MASK) {
                let _ = toolbar.add_color_to_palette(pix.to_rgba_struct());
            } else {
                toolbar.set_primary_color(pix.to_rgba_struct());
            }
        }
    }

    fn handle_right_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let (x, y) = canvas.cursor_pos_pix_u();
        if let Some(pix) = canvas.image().try_pix_at(y as i32, x as i32) {
           toolbar.set_secondary_color(pix.to_rgba_struct());
        }
    }

    fn handle_motion(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        canvas.update();
    }

    fn draw(&self, canvas: &Canvas, cr: &Context, toolbar: &mut Toolbar) {
        // draw one-pixel "brush" to highlight the target pixel
        let cursor_pos_f = canvas.cursor_pos_pix_f();
        let cursor_pos = (cursor_pos_f.0.floor(), cursor_pos_f.1.floor());

        let brush = toolbar.get_eyedropper_brush_mut();
        let x_offset = (brush.image.width() as i32 - 1) / 2;
        let y_offset = (brush.image.height() as i32 - 1) / 2;
        let path = brush.outline_path(cr);
        let _ = cr.save();
        {
            cr.translate(cursor_pos.0 - x_offset as f64, cursor_pos.1 - y_offset as f64);
            cr.new_path();
            cr.append_path(path);
            cr.set_source_rgb(0.0, 1.0, 0.0);
            let _ = cr.stroke();

        }
        let _ = cr.restore();

        // color box
        let _ = cr.save();
        {
            cr.translate(cursor_pos_f.0 - x_offset as f64, cursor_pos_f.1 - y_offset as f64);
            cr.scale(1.0 / canvas.zoom(), 1.0 / canvas.zoom());
            let (x, y) = canvas.cursor_pos_pix_u();
            if let Some(pix) = canvas.image_image_ref().try_pix_at(y, x) {
                let rgba = pix.to_rgba_struct();
                cr.rectangle(20.0, 30.0, 30.0, 30.0);

                // transparent rect
                const TRANSPARENT_CHECKER_SZ: f64 = 10.0;
                let trans_scale = TRANSPARENT_CHECKER_SZ;
                cr.scale(trans_scale, trans_scale);
                cr.set_source(canvas.transparent_checkerboard_pattern()).unwrap();
                let _ = cr.fill_preserve();
                cr.scale(1.0 / trans_scale, 1.0 / trans_scale);

                cr.set_source_rgba(
                    rgba.red().into(),
                    rgba.green().into(),
                    rgba.blue().into(),
                    rgba.alpha().into(),
                );
                let _ = cr.fill_preserve();
                cr.set_line_width(2.5);
                cr.set_source_rgb(0.0, 0.0, 0.0);
                let _ = cr.stroke();
            }
        }
        let _ = cr.restore();

    }
}
