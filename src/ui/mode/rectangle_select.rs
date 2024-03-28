use super::Canvas;

use gtk::gdk::ModifierType;
use gtk::cairo::{Context};

#[derive(Clone, Copy)]
pub struct RectangleSelectState {
    anchor_pos: (f64, f64),
}

impl RectangleSelectState {
    pub const fn default() -> RectangleSelectState {
        RectangleSelectState {
            anchor_pos: (0.0, 0.0),
        }
    }

    fn visual_cue_fn(&self, canvas: &Canvas) -> Box<dyn Fn(&Context)> {
        let zoom = *canvas.zoom();
        let (ax, ay) = self.anchor_pos;
        let (cx, cy) = canvas.cursor_pos_pix();

        Box::new(move |cr| {
            const LINE_WIDTH: f64 = 6.0;
            const LINE_BORDER_FACTOR: f64 = 0.4;

            let line_width = LINE_WIDTH / zoom;
            let line_widthp = line_width / 2.0;

            // anchor
            let ax = if cx > ax { ax.floor() } else { ax.ceil() };
            let ay = if cy > ay { ay.floor() } else { ay.ceil() };

            // target
            let tx = if cx > ax { cx.ceil() - ax } else { cx.floor() - ax };
            let ty = if cy > ay { cy.ceil() - ay } else { cy.floor() - ay };

            let (x, w) = if tx < ax {
                (ax + line_widthp, tx - line_widthp)
            } else {
                (ax - line_widthp, tx + line_widthp)
            };

            let (y, h) = if ty < ay {
                (ay + line_widthp, ty - line_widthp)
            } else {
                (ay - line_widthp, ty + line_widthp)
            };

            cr.set_line_width(line_width);

            cr.rectangle(x, y, w, h);
            cr.set_source_rgba(0.25, 0.25, 0.25, 0.75);
            cr.stroke();

            cr.rectangle(x, y, w, h);
            cr.set_line_width(LINE_WIDTH / zoom * LINE_BORDER_FACTOR);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.75);
            cr.stroke();
        })
    }
}

impl super::MouseModeState for RectangleSelectState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas) {
        self.anchor_pos = canvas.cursor_pos_pix();
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas) {
        canvas.update_with(self.visual_cue_fn(canvas));
    }

    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        // TODO
    }

    fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
    }

    fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
    }
}
