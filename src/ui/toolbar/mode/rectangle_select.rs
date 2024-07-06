use crate::ui::selection::Selection;
use super::{Canvas, Toolbar};
use crate::image::TrackedLayeredImage;

use gtk::gdk::ModifierType;
use gtk::cairo::Context;

#[derive(Clone, Copy)]
pub enum RectangleSelectMode {
    Unselected,
    Selecting(f64, f64),
    Selected(f64, f64, f64, f64),
}

#[derive(Clone, Copy)]
pub struct RectangleSelectState {
    pub mode: RectangleSelectMode,
    /// Gray out the non-selected region?
    crop_visual_enabled: bool,
}

impl RectangleSelectState {
    pub fn default(canvas: &Canvas) -> RectangleSelectState {
        let mode = match canvas.selection() {
            Selection::Rectangle(x, y, w, h) => RectangleSelectMode::Selected(*x as f64, *y as f64, *w as f64, *h as f64),
            _ => RectangleSelectMode::Unselected,
        };

        Self {
            mode,
            crop_visual_enabled: false,
        }
    }

    pub fn default_no_canvas() -> RectangleSelectState {
        Self {
            mode: RectangleSelectMode::Unselected,
            crop_visual_enabled: false,
        }
    }

    fn calc_xywh(ax: f64, ay: f64, canvas: &Canvas) -> (f64, f64, f64, f64) {
        let (cx, cy) = canvas.cursor_pos_pix_f();

        // round boundaries to nearest pixel
        let x = if cx > ax { ax.floor() } else { ax.ceil() };
        let y = if cy > ay { ay.floor() } else { ay.ceil() };

        let w = if cx > x { cx.ceil() - x } else { cx.floor() - x };
        let h = if cy > y { cy.ceil() - y } else { cy.floor() - y };

        // normalize negative values
        let (x, w) = if w < 0.0 { (x + w, -w) } else { (x, w) };
        let (y, h) = if h < 0.0 { (y + h, -h) } else { (y, h) };

        // pull coordinates into image bounds
        let max_x = canvas.image_width() as f64 - 1.0;
        let max_y = canvas.image_height() as f64 - 1.0;
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

        match self.mode {
            RectangleSelectMode::Unselected => Box::new(|_| ()),
            RectangleSelectMode::Selected(x, y, w, h) => Self::visual_box_around(x, y, w, h, zoom),
            RectangleSelectMode::Selecting(ax, ay) => {
                let (x, y, w, h) = Self::calc_xywh(ax, ay, canvas);
                Self::visual_box_around(x, y, w, h, zoom)
            }
        }
    }
}

fn crop_visual(x: f64, y: f64, w: f64, h: f64, img_w: i32, img_h: i32, cr: &Context) {
    // gray outline around everything
    cr.set_source_rgba(0.1, 0.1, 0.1, 0.5);
    cr.rectangle(0.0, 0.0, img_w as f64, img_h as f64);

    // inner rectangle not to fill
    cr.rectangle(x, y, w, h);

    cr.set_fill_rule(gtk::cairo::FillRule::EvenOdd);
    let _ = cr.fill();
}

impl super::MouseModeState for RectangleSelectState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if mod_keys.contains(ModifierType::CONTROL_MASK) {
            if let RectangleSelectMode::Selected(x, y, w, h) = self.mode {
                canvas.crop_to(x as usize, y as usize, w as usize, h as usize);
                self.mode = RectangleSelectMode::Unselected;
                canvas.set_selection(Selection::NoSelection);
                return;
            }
        }

        let (ax, ay) = canvas.cursor_pos_pix_f();
        self.mode = RectangleSelectMode::Selecting(ax, ay);
        canvas.set_selection(Selection::NoSelection);
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        canvas.update_with(self.visual_cue_fn(canvas));
    }

    fn handle_drag_end(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let RectangleSelectMode::Selecting(ax, ay) = self.mode {
            let (x, y, w, h) = Self::calc_xywh(ax, ay, canvas);
            self.mode = RectangleSelectMode::Selected(x, y, w, h);
            canvas.set_selection(Selection::Rectangle(x as usize, y as usize, w as usize, h as usize));
        }
        canvas.update();
    }

    fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        self.crop_visual_enabled = mod_keys.contains(ModifierType::CONTROL_MASK);
        canvas.update();
    }

    fn draw(&self, canvas: &Canvas, cr: &Context, _toolbar: &mut Toolbar) {
        if let RectangleSelectMode::Selected(x, y, w, h) = self.mode {
            if self.crop_visual_enabled {
                let image = canvas.image_ref();
                crop_visual(x, y, w, h, image.width(), image.height(), cr);
            }

            Self::visual_box_around(x, y, w, h, *canvas.zoom())(cr);
        }
    }

    fn handle_right_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        // deselect
        self.mode = RectangleSelectMode::Unselected;
        canvas.set_selection(Selection::NoSelection);
        canvas.update();
    }
}
