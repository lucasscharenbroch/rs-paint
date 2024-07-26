use crate::geometry::{dot_product, normalized_vec, vec_magnitude, vec_plus, vec_scale};
use crate::ui::selection::Selection;
use super::{Canvas, MouseMode, Toolbar};
use crate::image::TrackedLayeredImage;

use gtk::gdk::ModifierType;
use gtk::cairo::Context;
use gtk::prelude::*;
use gtk::gdk;

#[derive(Clone, Copy)]
pub enum TransformType {
    Pan(usize, usize), // anchor point (relative to the image)
    // resizes:
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl TransformType {
    fn cursor(&self) -> Option<gdk::Cursor> {
        let name = match self {
            Self::Pan(_ax, _ay) => "move",
            Self::Up |
            Self::Down => "ns-resize",
            Self::Left |
            Self::Right => "ew-resize",
            Self::UpLeft |
            Self::DownRight => "nwse-resize",
            Self::UpRight |
            Self::DownLeft  => "nesw-resize"
        };

        gdk::Cursor::from_name(
            name,
            gdk::Cursor::from_name("default", None).as_ref(),
        )
    }

    fn from_rect_and_pos(
        _rect@(x, y, w, h): (f64, f64, f64, f64),
        _pos@(cx, cy): (f64, f64),
        zoom: f64
    ) -> Option<Self> {
        const SIDE_THRESH_UNSCALED: f64 = 10.0;
        const CORNER_THRESH_UNSCALED: f64 = 15.0;

        let side_thresh = SIDE_THRESH_UNSCALED / zoom;
        let corner_thresh = CORNER_THRESH_UNSCALED / zoom;

        let d_left = (cx - x).abs();
        let d_right = (cx - (x + w)).abs();
        let d_top = (cy - y).abs();
        let d_bot = (cy - (y + h)).abs();

        let epsilon = 0.1;

        if d_left < corner_thresh && d_top < corner_thresh {
            Some(Self::UpLeft)
        } else if d_right < corner_thresh && d_top < corner_thresh {
            Some(Self::UpRight)
        } else if d_left < corner_thresh && d_bot < corner_thresh {
            Some(Self::DownLeft)
        } else if d_right < corner_thresh && d_bot < corner_thresh {
            Some(Self::DownRight)
        } else if d_left < side_thresh && (d_top + d_bot <= epsilon + h) {
            Some(Self::Left)
        } else if d_right < side_thresh && (d_top + d_bot <= epsilon + h) {
            Some(Self::Right)
        } else if d_top < side_thresh && (d_left + d_right <= epsilon + w) {
            Some(Self::Up)
        } else if d_bot < side_thresh && (d_left + d_right <= epsilon + w) {
            Some(Self::Down)
        } else if d_left + d_right <= epsilon + w && d_top + d_bot <= epsilon + h {
            Some(Self::Pan(cx as usize, cy as usize))
        } else {
            None
        }
    }

    fn compute_transformed_rect(&mut self,
        originial_rect@(ox, oy, ow, oh): (usize, usize, usize, usize),
        cursor_pos@(cx, cy): (usize, usize),
        max_x: usize,
        max_y: usize,
        lock_aspect_ratio: bool, /* TODO actually do this? */
    ) -> (usize, usize, usize, usize) {
        // assume all arguments are in-bounds

        // convert everything into i32, then back to usize at the end
        let (ox, oy, ow, oh) = (ox as i32, oy as i32, ow as i32, oh as i32);
        let (cx, cy) = (cx as i32, cy as i32);

        let (x, y, w, h) = match self {
            Self::Pan(ax, ay) => {
                let (dx, dy) = (cx - *ax as i32, cy - *ay as i32);
                *ax = cx as usize;
                *ay = cy as usize;

                (ox + dx, oy + dy, ow, oh)
            }
            Self::Up => {
                let d = (oy - cy).max(1 - oh);
                (ox, oy - d, ow, oh + d)
            },
            Self::Down => {
                let d = cy - (oy + oh);
                (ox, oy, ow, oh + d)
            },
            Self::Left => {
                let d = (ox - cx).max(1 - ow);
                (ox - d, oy, ow + d, oh)
            },
            Self::Right => {
                let d = cx - (ox + ow);
                (ox, oy, ow + d, oh)
            }
            Self::UpLeft => {
                let dx = (ox - cx).max(1 - ow);
                let dy = (oy - cy).max(1 - oh);
                (ox - dx, oy - dy, ow + dx, oh + dy)
            }
            Self::UpRight => {
                let dx = cx - (ox + ow);
                let dy = (oy - cy).max(1 - oh);
                (ox, oy - dy, ow + dx, oh + dy)
            },
            Self::DownLeft => {
                let dx = (ox - cx).max(1 - ow);
                let dy = cy - (oy + oh);
                (ox - dx, oy, ow + dx, oh + dy)
            },
            Self::DownRight => {
                let dx = cx - (ox + ow);
                let dy = cy - (oy + oh);
                (ox, oy, ow + dx, oh + dy)
            }
        };

        (
            x.max(0).min(1 + max_x as i32 - w) as usize,
            y.max(0).min(1 + max_y as i32 - h) as usize,
            w.max(1) as usize,
            h.max(1) as usize
        )
    }
}

#[derive(Clone, Copy)]
pub enum RectangleSelectMode {
    Unselected,
    Selecting(f64, f64),
    Selected(usize, usize, usize, usize),
    SelectedAndTransforming(usize, usize, usize, usize, TransformType),
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
            Selection::Rectangle(x, y, w, h) => RectangleSelectMode::Selected(*x, *y, *w, *h),
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

    /// Determine the the position of the selected rect
    /// where `(ax, ay)` is the "anchor point"
    fn calc_xywh(ax: f64, ay: f64, canvas: &Canvas, maintain_square_ratio: bool) -> (usize, usize, usize, usize) {
        let (cx, cy) = canvas.cursor_pos_pix_f();

        let (cx, cy) = if !maintain_square_ratio {
            (cx, cy)
        } else {
            // compute the diagonal of the square whose
            // corner's "tangent" intersects (cx, cy)

            fn sign(x: f64) -> f64 {
                if x < 0.0 { -1.0 } else { 1.0 }
            }

            // hypotenuse vector, anchor -> cursor
            let hv = (cx - ax, cy - ay);
            // diagonal vector: the (unit) vector along the direction
            // of the target side (the diagonal of the square)
            let dv = (sign(cx - ax), sign(cy - ay));

            // cosine of the angle between = diagonal_length / hypotenuse_length
            // (normalize both for consistency/sanity)
            let cos = dot_product(normalized_vec(hv), normalized_vec(dv));

            let diagonal_length = cos * vec_magnitude(hv);

            vec_plus((ax, ay), vec_scale(diagonal_length / std::f64::consts::SQRT_2, dv))
        };

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
        let w = w.min(max_x - x + 1.0);
        let h = h.min(max_y - y + 1.0);

        (x as usize, y as usize, w as usize, h as usize)
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

    fn visual_cue_fn(&self, canvas: &Canvas, maintain_square_ratio: bool) -> Box<dyn Fn(&Context)> {
        let zoom = *canvas.zoom();

        match self.mode {
            RectangleSelectMode::Unselected => Box::new(|_| ()),
            RectangleSelectMode::Selected(x, y, w, h) => Self::visual_box_around(x as f64, y as f64, w as f64, h as f64, zoom),
            RectangleSelectMode::SelectedAndTransforming(x, y, w, h, _scale_type) => Self::visual_box_around(x as f64, y as f64, w as f64, h as f64, zoom),
            RectangleSelectMode::Selecting(ax, ay) => {
                let (x, y, w, h) = Self::calc_xywh(ax, ay, canvas, maintain_square_ratio);
                Self::visual_box_around(x as f64, y as f64, w as f64, h as f64, zoom)
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

        if let RectangleSelectMode::Selected(x, y, w, h) = self.mode {
            let cursor_pos = canvas.cursor_pos_pix_f();
            let zoom = *canvas.zoom();
            if let Some(resize_type) = TransformType::from_rect_and_pos((x as f64, y as f64, w as f64, h as f64), cursor_pos, zoom) {
                self.mode = RectangleSelectMode::SelectedAndTransforming(x, y, w, h, resize_type);
                return;
            }
        }

        let (ax, ay) = canvas.cursor_pos_pix_f();
        self.mode = RectangleSelectMode::Selecting(ax, ay);
        canvas.set_selection(Selection::NoSelection);
    }

    fn handle_drag_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let RectangleSelectMode::SelectedAndTransforming(x, y, w, h, mut resize_type) = self.mode {
            let (x, y, w, h) = resize_type.compute_transformed_rect(
                (x as usize, y as usize, w as usize, h as usize),
                canvas.cursor_pos_pix_u_in_bounds(),
                (canvas.image_width() - 1) as usize,
                (canvas.image_height() - 1) as usize,
                mod_keys.intersects(gdk::ModifierType::SHIFT_MASK),
            );
            canvas.set_selection(Selection::Rectangle(x as usize, y as usize, w as usize, h as usize));
            self.mode = RectangleSelectMode::SelectedAndTransforming(x, y, w, h, resize_type);
        }

        canvas.update_with(self.visual_cue_fn(canvas, mod_keys.intersects(ModifierType::SHIFT_MASK)));
    }

    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let RectangleSelectMode::Selecting(ax, ay) = self.mode {
            let (x, y, w, h) = Self::calc_xywh(ax, ay, canvas, mod_keys.intersects(ModifierType::SHIFT_MASK));
            self.mode = RectangleSelectMode::Selected(x, y, w, h);
            canvas.set_selection(Selection::Rectangle(x as usize, y as usize, w as usize, h as usize));
        } else if let RectangleSelectMode::SelectedAndTransforming(x, y, w, h, mut resize_type) = self.mode {
            let (x, y, w, h) = resize_type.compute_transformed_rect(
                (x as usize, y as usize, w as usize, h as usize),
                canvas.cursor_pos_pix_u_in_bounds(),
                (canvas.image_width() - 1) as usize,
                (canvas.image_height() - 1) as usize,
                mod_keys.intersects(gdk::ModifierType::SHIFT_MASK),
            );
            canvas.set_selection(Selection::Rectangle(x as usize, y as usize, w as usize, h as usize));
            self.mode = RectangleSelectMode::Selected(x, y, w, h);
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
                let fused_layered_image = canvas.layered_image();
                crop_visual(
                    x as f64, y as f64, w as f64, h as f64,
                    fused_layered_image.width(),
                    fused_layered_image.height(),
                    cr
                );
            }

            Self::visual_box_around(x as f64, y as f64, w as f64, h as f64, *canvas.zoom())(cr);
        }
    }

    fn handle_right_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        // deselect
        self.mode = RectangleSelectMode::Unselected;
        canvas.set_selection(Selection::NoSelection);
        canvas.update();
    }

    fn handle_selection_deleted(&mut self) {
        self.mode = RectangleSelectMode::Unselected;
    }

    fn handle_close(&self, canvas: &mut Canvas, _toolbar: &Toolbar, _new_mode: &MouseMode) {
        canvas.drawing_area().set_cursor(gdk::Cursor::from_name("default", None).as_ref());
    }

    fn handle_motion(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        let default_cursor = gdk::Cursor::from_name("default", None);

        let cursor = match self.mode {
            RectangleSelectMode::SelectedAndTransforming(_x, _y, _w, _h, scale_mode) => {
                scale_mode.cursor()
            },
            RectangleSelectMode::Selected(x, y, w, h) => {
                let (cx, cy) = canvas.cursor_pos_pix_f();
                TransformType::from_rect_and_pos((x as f64, y as f64, w as f64, h as f64), (cx, cy), *canvas.zoom())
                    .map(|scale_type| scale_type.cursor())
                    .unwrap_or(default_cursor)
            },
            _ => default_cursor,
        };

        canvas.drawing_area().set_cursor(
            cursor.as_ref()
        );
    }
}
