use super::{Canvas, Toolbar};
use gtk::{prelude::*, cairo, gdk};
use crate::geometry::*;
use crate::image::undo::action::ActionName;
use crate::transformable::Transformable;

/// Canvas-held data associated with a free-transform
pub struct TransformationSelection {
    pub transformable: Box<dyn Transformable>,
    pub matrix: cairo::Matrix,
    pub culprit: ActionName,
    /// (height/width); the "lossless" ratio, which is
    /// calculated upon instantiation and not modified
    /// when the aspect ratio is locked. This will likely
    /// not exactly match the aspect ratio of the matrix.
    true_aspect_ratio: f64,
}

impl TransformationSelection {
    pub fn new(transformable: Box<dyn Transformable>, matrix: cairo::Matrix, culprit: ActionName) -> Self {
        let (width, height) = matrix_width_height(&matrix);
        let true_aspect_ratio = height / width;

        TransformationSelection {
            transformable,
            matrix,
            culprit,
            true_aspect_ratio,
        }
    }
}

#[derive(Clone, Copy)]
enum TransformationType {
    /// Do nothing
    Sterile,
    Translate,
    ScaleUpLeft,
    ScaleUpRight,
    ScaleDownLeft,
    ScaleDownRight,
    ScaleUp,
    ScaleDown,
    ScaleLeft,
    ScaleRight,
    Rotate,
}

/// Returns the stub length w/r/t the height of the unit square
fn rotation_stub_length(width: f64, height: f64) -> f64 {
    0.05 * width.max(height) / height
}

impl TransformationType {
    fn from_matrix_and_point(matrix: &cairo::Matrix, (x, y): (f64, f64), zoom: f64) -> Self {
        let outer_margin = 10.0 / zoom;
        let inner_margin = 10.0 / zoom;

        let (width, height) = matrix_width_height(matrix);

        // how many pixels from the border
        // the mouse must be to switch to expansion
        let outer_border_radius_x = outer_margin / width;
        let outer_border_radius_y = outer_margin / height;
        let inner_border_radius_x = inner_margin / width;
        let inner_border_radius_y = inner_margin / height;

        // if there's no inverse, return garbage value - it's bad, but better than crashing
        let inverse = matrix.try_invert().unwrap_or(matrix.clone());
        let pt@(x, y) = inverse.transform_point(x, y);

        let (width, height) = matrix_width_height(matrix);
        let rotation_stub_length = rotation_stub_length(width, height);

        let rotation_nub_pt = (0.5, -rotation_stub_length);
        let dist_from_rotation_nub = point_tuple_dist(rotation_nub_pt, pt);

        let (x0, y0) = (0.0, 0.0);
        let (x1, y1) = (1.0, 1.0);
        let (x0i, y0i) = (x0 + inner_border_radius_x, y0 + inner_border_radius_y);
        let (x1i, y1i) = (x1 - inner_border_radius_x, y1 - inner_border_radius_y);
        let (x0o, y0o) = (x0 - outer_border_radius_x, y0 - outer_border_radius_y);
        let (x1o, y1o) = (x1 + outer_border_radius_x, y1 + outer_border_radius_y);

        // give a little extra margin for corners
        const EXTRA_MARGIN: f64 = 1.5;
        let (x0oo, y0oo) = (x0 - outer_border_radius_x * EXTRA_MARGIN, y0 - outer_border_radius_y * EXTRA_MARGIN);
        let (x1oo, y1oo) = (x1 + outer_border_radius_x * EXTRA_MARGIN, y1 + outer_border_radius_y * EXTRA_MARGIN);

        let close_to_left = x < x0i && x >= x0o;
        let close_to_right = x > x1i && x <= x1o;
        let close_to_top = y < y0i && y >= y0o;
        let close_to_bot = y > y1i && y <= y1o;
        let close_ish_to_left = x < x0i && x >= x0oo;
        let close_ish_to_right = x > x1i && x <= x1oo;
        let close_ish_to_top = y < y0i && y >= y0oo;
        let close_ish_to_bot = y > y1i && y <= y1oo;
        let in_vert_bounds = y >= y0 && y <= y1;
        let in_horz_bounds = x >= x0 && x <= x1;

        if dist_from_rotation_nub < (outer_border_radius_x + outer_border_radius_y) / 2.0 {
            TransformationType::Rotate
        } else if x >= x0i && x <= x1i && y >= y0i && y <= y1i {
            TransformationType::Translate
        } else if close_ish_to_top && close_ish_to_left {
            TransformationType::ScaleUpLeft
        } else if close_ish_to_top && close_ish_to_right {
            TransformationType::ScaleUpRight
        } else if close_ish_to_bot && close_ish_to_left {
            TransformationType::ScaleDownLeft
        } else if close_ish_to_bot && close_ish_to_right {
            TransformationType::ScaleDownRight
        } else if close_to_left && in_vert_bounds {
            TransformationType::ScaleLeft
        } else if close_to_right && in_vert_bounds {
            TransformationType::ScaleRight
        } else if close_to_top && in_horz_bounds {
            TransformationType::ScaleUp
        } else if close_to_bot && in_horz_bounds {
            TransformationType::ScaleDown
        } else {
            TransformationType::Sterile
        }
    }

    // very sloppy, but it works
    fn get_cursor_name(&self, matrix: &cairo::Matrix) -> &str {
        let inverse = matrix.try_invert().unwrap();

        let p00 = inverse.transform_point(0.0, 0.0);
        let p10 = inverse.transform_point(1.0, 0.0);
        let p01 = inverse.transform_point(0.0, 1.0);
        let p11 = inverse.transform_point(1.0, 1.0);

        let pts = match self {
            TransformationType::Sterile => return "default",
            TransformationType::Translate => return "move",
            TransformationType::Rotate => return "grab", // this one isn't great...
            TransformationType::ScaleUpLeft => (p11, p00),
            TransformationType::ScaleUpRight => (p10, p01),
            TransformationType::ScaleDownLeft => (p10, p01),
            TransformationType::ScaleDownRight => (p00, p11),
            TransformationType::ScaleUp => (p10, p00),
            TransformationType::ScaleDown => (p10, p00),
            TransformationType::ScaleLeft => (p01, p00),
            TransformationType::ScaleRight => (p01, p00),
        };

        let directional_cursors = vec![
            // (vector, cursor-name)
            // replicate each for the opposite arrow-direction
            (((1.0, -1.0)), "nwse-resize"),
            (((-1.0, 1.0)), "nwse-resize"),
            (((1.0, 1.0)), "nesw-resize"),
            (((-1.0, -1.0)), "nesw-resize"),
            (((0.0, 1.0)), "ns-resize"),
            (((0.0, -1.0)), "ns-resize"),
            (((1.0, 0.0)), "ew-resize"),
            (((-1.0, 0.0)), "ew-resize"),
        ];

        // the vector formed by the two points
        let vec = (pts.1.0 - pts.0.0, pts.1.1 - pts.0.1);

        fn vecs_to_sin(v0: (f64, f64), v1: (f64, f64)) -> f64 {
            cross_product(v0, v1) / vec_magnitude(v0) / vec_magnitude(v1)
        }

        directional_cursors.iter()
            .map(|(test_vec, name)| {
                (vecs_to_sin(*test_vec, vec), name)
            })
            .max_by(|(a, _name_a), (b, _name_b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .1
    }

    /// The gdk::Cursor associated with this transformation
    fn cursor(&self, matrix: &cairo::Matrix) -> Option<gdk::Cursor> {
        gdk::Cursor::from_name(
            self.get_cursor_name(matrix),
            gdk::Cursor::from_name("default", None).as_ref(),
        )
    }

    fn is_scale(&self) -> bool {
        match self {
            Self::ScaleUpLeft |
            Self::ScaleUp |
            Self::ScaleUpRight |
            Self::ScaleLeft |
            Self::ScaleRight |
            Self::ScaleDownLeft |
            Self::ScaleDown |
            Self::ScaleDownRight => true,
            _ => false,
        }
    }

    fn is_translate(&self) -> bool {
        match self {
            Self::Translate => true,
            _ => false,
        }
    }

    fn is_rotate(&self) -> bool {
        match self {
            Self::Rotate => true,
            _ => false,
        }
    }

    fn should_clamp(&self, clamp_translate: bool, clamp_scale: bool, clamp_rotate: bool) -> bool {
        self.is_translate() && clamp_translate ||
        self.is_scale() && clamp_scale ||
        self.is_rotate() && clamp_rotate
    }

    fn do_scale(
        matrix: &mut cairo::Matrix,
        maintain_aspect_ratio: bool,
        true_aspect_ratio: f64,
        should_clamp: bool,
        width: f64,
        height: f64,
        dx: f64,
        dy: f64,
        up: f64, // how much up? (1.0, 0.0, or -1.0)
        left: f64, // how much left?
    ) -> (f64, f64) {
        // let (dx, dy) = (dx * -left, dy * -up);

        // handle aspect-ratio-locking
        let (dx, dy) = if maintain_aspect_ratio {
            // trig that's really hard to explain concisely in comments...
            let ar_vec = (1.0 * -left, true_aspect_ratio * -up);
            let dist_to_scale = dot_product((dx, dy), ar_vec) / vec_magnitude(ar_vec);
            let d = dist_to_scale / (up.abs() + left.abs()).sqrt();

            // ideally we scale by (d, d), but the aspect ratio probably isn't equal to the
            // true aspect ratio (but we want it to be), so adjust accordingly

            let dy_over_dx = true_aspect_ratio * (width / height);
            (d, dy_over_dx * d)
        } else {
            (dx * -left, dy * -up)
        };

        // before-clamp scale
        let (sx, sy) = (
            1.0 + dx,
            1.0 + dy
        );

        // after-clamp scale
        let (sx, sy, rx, ry) = if should_clamp {
            let (rsx, rsy) = (
                (sx * width).floor().max(1.0 / width) / width,
                (sy * height).floor().max(1.0 / height) / height,
            );
            (rsx, rsy, sx - rsx, sy - rsy)
        } else {
            // prevent cross-over and zero-size here too
            // (zero size for matrix inversion problems;
            // cross-over for rotation bugs when crossed)
            (sx.max(0.1 / width), sy.max(0.1 / height), 0.0, 0.0)
        };

        let (tx, ty) = (
            (left + 1.0) * (1.0 - sx) / 2.0,
            (up + 1.0) * (1.0 - sy) / 2.0,
        );

        matrix.translate(tx, ty);
        matrix.scale(sx, sy);

        // remainder
        matrix.transform_distance(
            rx * -left,
            ry * -up,
        )
    }

    fn update_matrix_with_point_diff(
        &self,
        matrix: &mut cairo::Matrix,
        true_aspect_ratio: f64,
        (x0, y0): (f64, f64),
        (x1, y1): (f64, f64),
        maintain_aspect_ratio: bool,
        clamp_translate: bool,
        clamp_scale: bool,
        clamp_rotate: bool,
    ) -> (f64, f64) {
        let (width, height) = matrix_width_height(matrix);

        let inverse = matrix.try_invert().unwrap();
        let (dx, dy) = inverse.transform_distance(x1 - x0, y1 - y0);

        let should_clamp = self.should_clamp(clamp_translate, clamp_scale, clamp_rotate);

        match self {
            Self::Sterile => (0.0, 0.0),
            Self::Translate => {
                if should_clamp {
                    let (rdx, rdy) = ((dx * width).floor() / width, (dy * height).floor() / height);
                    let (rx, ry) = (dx - rdx, dy - rdy);
                    matrix.translate(rdx, rdy);
                    matrix.transform_distance(rx, ry)
                } else {
                    matrix.translate(dx, dy);
                    (0.0, 0.0)
                }
            },
            Self::ScaleUpLeft => {
                Self::do_scale(
                    matrix,
                    maintain_aspect_ratio, true_aspect_ratio, should_clamp,
                    width, height, dx, dy,
                    1.0, 1.0,
                )
            },
            Self::ScaleUpRight => {
                Self::do_scale(
                    matrix,
                    maintain_aspect_ratio, true_aspect_ratio, should_clamp,
                    width, height, dx, dy,
                    1.0, -1.0,
                )
            }
            Self::ScaleDownLeft => {
               Self::do_scale(
                    matrix,
                    maintain_aspect_ratio, true_aspect_ratio, should_clamp,
                    width, height, dx, dy,
                    -1.0, 1.0,
                )
            }
            Self::ScaleDownRight => {
                Self::do_scale(
                    matrix,
                    maintain_aspect_ratio, true_aspect_ratio, should_clamp,
                    width, height, dx, dy,
                    -1.0, -1.0,
                )
            },
            Self::ScaleUp => {
                Self::do_scale(
                    matrix,
                    maintain_aspect_ratio, true_aspect_ratio, should_clamp,
                    width, height, dx, dy,
                    1.0, 0.0,
                )
            }
            Self::ScaleDown => {
                Self::do_scale(
                    matrix,
                    maintain_aspect_ratio, true_aspect_ratio, should_clamp,
                    width, height, dx, dy,
                    -1.0, 0.0,
                )
            }
            Self::ScaleLeft => {
                Self::do_scale(
                    matrix,
                    maintain_aspect_ratio, true_aspect_ratio, should_clamp,
                    width, height, dx, dy,
                    0.0, 1.0,
                )
            }
            Self::ScaleRight => {
                Self::do_scale(
                    matrix,
                    maintain_aspect_ratio, true_aspect_ratio, should_clamp,
                    width, height, dx, dy,
                    0.0, -1.0,
                )
            }
            Self::Rotate => {
                let (x0, y0) = matrix.transform_point(0.5, 0.0);
                let (x2, y2) = matrix.transform_point(0.5, 0.5);

                // target angle (`a`) is angle between p0@(0.5, 0.0) (the rotation-nub-area),
                // p2@(0.5, 0.5) (the center of the image), and p1 (the current cursor position)

                /*     | target angle (`a`)
                       |
                    p1 v p0
                      \--|
                    v1 \ | v0
                        \|
                         p2
                 */

                let v0 = (x0 - x2, y0 - y2);
                let v1 = (x1 - x2, y1 - y2);

                let dp = dot_product(v0, v1);
                let cp = cross_product(v0, v1);

                // magnitude
                let m0 = vec_magnitude(v0);
                let m1 = vec_magnitude(v1);

                let a = (dp / (m0 * m1)).acos();

                // invert the direction, if necessary
                let a = if cp < 0.0 { -a } else { a };

                matrix.translate(0.5, 0.5);
                matrix.scale(1.0, width / height);
                matrix.rotate(a);
                matrix.scale(1.0, height / width);
                matrix.translate(-0.5, -0.5);
                (0.0, 0.0) // TODO
            }
        }
    }
}

#[derive(Clone, Copy)]
enum FreeTransformMouseState {
    Up,
    Down(f64, f64, TransformationType, f64, f64), // x, y, type, extra_dx, extra_dy
}

#[derive(Clone, Copy)]
pub struct FreeTransformState {
    mouse_state: FreeTransformMouseState,
}

impl FreeTransformState {
    pub fn from_coords(x: f64, y: f64) -> FreeTransformState {
        FreeTransformState {
            mouse_state: FreeTransformMouseState::Down(x, y, TransformationType::ScaleDownRight, 0.0, 0.0),
        }
    }

    pub fn default(canvas: &mut Canvas) -> FreeTransformState {
        if canvas.transformation_selection().borrow().is_some() {
            // we've got a selection - no state to retrive,
            // because it's fetched on-demand from the `Canvas` anyway
        } else {
            let _ = canvas.try_consume_selection_to_transformable();
            // if this fails, we continue anyway, but the mode
            // will be useless (but harmless)
        }

        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> FreeTransformState {
        FreeTransformState {
            mouse_state: FreeTransformMouseState::Up,
        }
    }
}

impl FreeTransformState {
    /// Draw the transform tool selection visual (border + corners, etc)
    /// in the unit square
    fn draw_transform_overlay(cr: &cairo::Context, zoom: f64, width: f64, height: f64) {
        const BORDER_WIDTH: f64 = 3.0;
        const POINT_RADIUS: f64 = 5.0;

        // height in terms of width
        let aspect_ratio = height / width;

        fn corner(cr: &cairo::Context, x: f64, y: f64, radius: f64) {
            cr.rectangle(x - radius, y - radius, 2.0 * radius, 2.0 * radius);
            let _ = cr.fill();
        }

        cr.set_source_rgb(0.0, 0.0, 1.0);
        let _ = cr.save();
        {
            cr.scale(1.0, 1.0 / aspect_ratio);

            let x0 = 0.0;
            let y0 = 0.0;
            let x1 = 1.0;
            let y1 = aspect_ratio;

            let mult = 1.0 / width / zoom;

            cr.set_line_width(BORDER_WIDTH * mult);
            cr.rectangle(x0, y0, x1 - x0, y1 - x0);
            let _ = cr.stroke();

            corner(cr, x0, y0, POINT_RADIUS * mult);
            corner(cr, x0, y1, POINT_RADIUS * mult);
            corner(cr, x1, y0, POINT_RADIUS * mult);
            corner(cr, x1, y1, POINT_RADIUS * mult);

            let rotation_stub_length = rotation_stub_length(width, height);

            cr.move_to(0.5, y0);
            cr.line_to(0.5, y0 - rotation_stub_length * aspect_ratio);
            let _ = cr.stroke();

            corner(cr, 0.5, -rotation_stub_length * aspect_ratio, POINT_RADIUS * mult)
        }
        let _ = cr.restore();
    }
}

impl super::MouseModeState for FreeTransformState {
    fn handle_motion(&mut self, _mod_keys: &gdk::ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let Some(selection) = canvas.transformation_selection().borrow_mut().as_mut() {
                // cursor
                let cursor_pos = canvas.cursor_pos_pix_f();
                canvas.drawing_area().set_cursor(
                    TransformationType::from_matrix_and_point(&selection.matrix, cursor_pos, *canvas.zoom())
                        .cursor(&selection.matrix).as_ref()
                );
        }
    }

    fn draw(&self, canvas: &Canvas, cr: &cairo::Context, _toolbar: &mut Toolbar) {
        if let Some(selection) = canvas.transformation_selection().borrow_mut().as_mut() {
            let _ = cr.save();
            {
                cr.set_matrix(cairo::Matrix::multiply(&selection.matrix, &cr.matrix()));
                let (w, h) = matrix_width_height(&selection.matrix);
                selection.transformable.draw(cr, w, h);

                let (width, height) = matrix_width_height(&selection.matrix);
                Self::draw_transform_overlay(cr, *canvas.zoom(), width, height);
            }
            let _ = cr.restore();
        }
    }

    fn handle_drag_start(&mut self, _mod_keys: &gtk::gdk::ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let Some(selection) = canvas.transformation_selection().borrow_mut().as_mut() {
            let (x, y) = canvas.cursor_pos_pix_f();
            self.mouse_state = FreeTransformMouseState::Down(x, y,
                TransformationType::from_matrix_and_point(&selection.matrix, (x, y), *canvas.zoom()),
                0.0, 0.0,
            )
        }
    }

    fn handle_drag_update(&mut self, mod_keys: &gtk::gdk::ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let should_update = if let Some(selection) = canvas.transformation_selection().borrow_mut().as_mut() {
            if let FreeTransformMouseState::Down(x0, y0, transform_type, extra_x, extra_y) = self.mouse_state {
                let (x, y) = canvas.cursor_pos_pix_f();

                let maintain_aspect_ratio = mod_keys.intersects(gtk::gdk::ModifierType::SHIFT_MASK);

                let (extra_x, extra_y) = transform_type.update_matrix_with_point_diff(
                    &mut selection.matrix,
                    selection.true_aspect_ratio,
                    (x0, y0),
                    (x + extra_x, y + extra_y),
                    maintain_aspect_ratio,
                    toolbar.get_clamp_translate(),
                    toolbar.get_clamp_scale(),
                    toolbar.get_clamp_rotate(),
                );

                selection.true_aspect_ratio = if maintain_aspect_ratio {
                    selection.true_aspect_ratio
                } else {
                    let (w, h) = matrix_width_height(&selection.matrix);
                    h / w
                };

                self.mouse_state = FreeTransformMouseState::Down(x, y, transform_type, extra_x, extra_y);
                true
            } else {
                false
            }
        } else {
            false
        };

        if should_update {
            canvas.update();
        }
    }

    fn handle_drag_end(&mut self, _mod_keys: &gtk::gdk::ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        self.mouse_state = FreeTransformMouseState::Up;
    }

    fn handle_close(&self, canvas: &mut Canvas, _toolbar: &Toolbar) {
        canvas.drawing_area().set_cursor(gdk::Cursor::from_name("default", None).as_ref());
    }
}
