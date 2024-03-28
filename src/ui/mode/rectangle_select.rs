use super::Canvas;

use gtk::gdk::ModifierType;
use gtk::cairo::{Context};


#[derive(Clone, Copy)]
pub enum RectangleSelectState {
    Unselected,
    Selecting(f64, f64),
    Selected(f64, f64, f64, f64),
}

impl RectangleSelectState {
    pub const fn default() -> RectangleSelectState {
        Self::Unselected
    }

    fn calc_xywh(ax: f64, ay: f64, canvas: &Canvas) -> (f64, f64, f64, f64) {
        let (cx, cy) = canvas.cursor_pos_pix();

        let x = if cx > ax { ax.floor() } else { ax.ceil() };
        let y = if cy > ay { ay.floor() } else { ay.ceil() };

        let w = if cx > x { cx.ceil() - x } else { cx.floor() - x };
        let h = if cy > y { cy.ceil() - y } else { cy.floor() - y };

        (x, y, w, h)
    }

    fn visual_box_around(x: f64, y: f64, w: f64, h: f64, zoom: f64) -> Box<dyn Fn(&Context)> {
        Box::new(move |cr| {
            const LINE_WIDTH: f64 = 6.0;
            const LINE_BORDER_FACTOR: f64 = 0.4;

            cr.set_line_width(LINE_WIDTH / zoom);

            cr.rectangle(x, y, w, h);
            cr.set_source_rgba(0.25, 0.25, 0.25, 0.75);
            cr.stroke();

            cr.rectangle(x, y, w, h);
            cr.set_line_width(LINE_WIDTH / zoom * LINE_BORDER_FACTOR);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.75);
            cr.stroke();
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
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas) {
        let (ax, ay) = canvas.cursor_pos_pix();
        *self = Self::Selecting(ax, ay);
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas) {
        canvas.update_with(self.visual_cue_fn(canvas));
    }

    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        canvas.update_with(self.visual_cue_fn(canvas));
    }

    fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
    }

    fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
    }
}
