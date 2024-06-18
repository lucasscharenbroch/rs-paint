use crate::ui::selection::Selection;

use super::{Canvas, Toolbar, MouseModeVariant};
use crate::ui::form::Form;

use gtk::gdk::ModifierType;
use gtk::cairo::Context;

#[derive(Clone, Copy)]
pub enum RectangleSelectState {
    Unselected,
    Selecting(f64, f64),
    Selected(f64, f64, f64, f64),
}

impl RectangleSelectState {
    pub fn default(canvas: &Canvas) -> RectangleSelectState {
        match canvas.selection() {
            Selection::Rectangle(x, y, w, h) => Self::Selected(*x as f64, *y as f64, *w as f64, *h as f64),
            _ => Self::Unselected
        }
    }

    pub fn default_no_canvas() -> RectangleSelectState {
        Self::Unselected
    }

    fn calc_xywh(ax: f64, ay: f64, canvas: &Canvas) -> (f64, f64, f64, f64) {
        let (cx, cy) = canvas.cursor_pos_pix();

        // round boundaries to nearest pixel
        let x = if cx > ax { ax.floor() } else { ax.ceil() };
        let y = if cy > ay { ay.floor() } else { ay.ceil() };

        let w = if cx > x { cx.ceil() - x } else { cx.floor() - x };
        let h = if cy > y { cy.ceil() - y } else { cy.floor() - y };

        // normalize negative values
        let (x, w) = if w < 0.0 { (x + w, -w) } else { (x, w) };
        let (y, h) = if h < 0.0 { (y + h, -h) } else { (y, h) };

        // pull coordinates into image bounds
        let max_x = canvas.image_width() as f64;
        let max_y = canvas.image_height() as f64;
        let (x, w) = if x < 0.0 || x > max_x {
            let xp = x.max(0.0).min(max_x);
            (xp, (w - (x - xp).abs()).max(0.0))
        } else { (x, w) };
        let (y, h) = if y < 0.0 || y > max_y {
            let yp = y.max(0.0).min(max_y);
            (yp, (h - (y - yp).abs()).max(0.0))
        } else { (y, h) };
        let w = w.min(max_x - x);
        let h = h.min(max_y - y);

        (x, y, w, h)
    }

    fn visual_box_around(x: f64, y: f64, w: f64, h: f64, zoom: f64) -> Box<dyn Fn(&Context)> {
        Box::new(move |cr| {
            const LINE_WIDTH: f64 = 6.0;
            const LINE_BORDER_FACTOR: f64 = 0.4;

            cr.set_line_width(LINE_WIDTH / zoom);
            cr.set_source_rgba(0.25, 0.25, 0.25, 0.75);
            cr.rectangle(x, y, w, h);
            let _ = cr.stroke();

            cr.set_line_width(LINE_WIDTH / zoom * LINE_BORDER_FACTOR);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.75);
            cr.rectangle(x, y, w, h);
            let _ = cr.stroke();
        })
    }

    fn visual_cue_fn(&self, canvas: &Canvas) -> Box<dyn Fn(&Context)> {
        let zoom = *canvas.zoom();

        match self {
            Self::Unselected => Box::new(|_| ()),
            Self::Selected(x, y, w, h) => Self::visual_box_around(*x, *y, *w, *h, zoom),
            Self::Selecting(ax, ay) => {
                let (x, y, w, h) = Self::calc_xywh(*ax, *ay, canvas);
                Self::visual_box_around(x, y, w, h, zoom)
            }
        }
    }
}

impl super::MouseModeState for RectangleSelectState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if mod_keys.contains(ModifierType::CONTROL_MASK) {
            if let Self::Selected(x, y, w, h) = self {
                canvas.crop_to(*x as usize, *y as usize, *w as usize, *h as usize);
                *self = Self::Unselected;
                canvas.set_selection(Selection::NoSelection);
                return;
            }
        }

        let (ax, ay) = canvas.cursor_pos_pix();
        *self = Self::Selecting(ax, ay);
        canvas.set_selection(Selection::NoSelection);
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        canvas.update_with(self.visual_cue_fn(canvas));
    }

    fn handle_drag_end(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let Self::Selecting(ax, ay) = self {
            let (x, y, w, h) = Self::calc_xywh(*ax, *ay, canvas);
            *self = Self::Selected(x, y, w, h);
            canvas.set_selection(Selection::Rectangle(x as usize, y as usize, w as usize, h as usize));
        }
        canvas.update();
    }

    fn draw(&self, canvas: &Canvas, cr: &Context, _toolbar: &mut Toolbar) {
        if let Self::Selected(x, y, w, h) = self {
            Self::visual_box_around(*x, *y, *w, *h, *canvas.zoom())(cr);
        }
    }
}
