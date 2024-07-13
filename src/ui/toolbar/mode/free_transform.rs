use super::{Canvas, Toolbar};
use gtk::{prelude::*, cairo, gdk};

#[derive(Clone, Copy)]
pub enum TransformMode {
    /// Nothing selected, don't do anything
    NotTransforming,
    Transforming(cairo::Matrix),
}

#[derive(Clone, Copy)]
enum TransformationType {
    /// Do nothing
    Sterile,
    Translate,
    ExpandUpLeft,
    ExpandUpRight,
    ExpandDownLeft,
    ExpandDownRight,
    ExpandUp,
    ExpandDown,
    ExpandLeft,
    ExpandRight,
    Rotate,
}

/// Returns the stub length w/r/t the height of the unit square
fn rotation_stub_length(width: f64, height: f64) -> f64 {
    0.05 * width.max(height) / height
}

fn point_tuple_dist((x0, y0): (f64, f64), (x1, y1): (f64, f64)) -> f64 {
    ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
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
            TransformationType::ExpandUpLeft
        } else if close_ish_to_top && close_ish_to_right {
            TransformationType::ExpandUpRight
        } else if close_ish_to_bot && close_ish_to_left {
            TransformationType::ExpandDownLeft
        } else if close_ish_to_bot && close_ish_to_right {
            TransformationType::ExpandDownRight
        } else if close_to_left && in_vert_bounds {
            TransformationType::ExpandLeft
        } else if close_to_right && in_vert_bounds {
            TransformationType::ExpandRight
        } else if close_to_top && in_horz_bounds {
            TransformationType::ExpandUp
        } else if close_to_bot && in_horz_bounds {
            TransformationType::ExpandDown
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
            TransformationType::ExpandUpLeft => (p11, p00),
            TransformationType::ExpandUpRight => (p10, p01),
            TransformationType::ExpandDownLeft => (p10, p01),
            TransformationType::ExpandDownRight => (p00, p11),
            TransformationType::ExpandUp => (p10, p00),
            TransformationType::ExpandDown => (p10, p00),
            TransformationType::ExpandLeft => (p01, p00),
            TransformationType::ExpandRight => (p01, p00),
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

        fn vec_magnitude((x, y): (f64, f64)) -> f64 {
            (x.powi(2) + y.powi(2)).sqrt()
        }

        fn cross_product((dx0, dy0): (f64, f64), (dx1, dy1): (f64, f64)) -> f64 {
            dx0 * dy1 - dx1 * dy0
        }

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

    fn update_matrix_with_point_diff(
        &self, matrix: &mut cairo::Matrix,
        (x0, y0): (f64, f64),
        (x1, y1): (f64, f64)
    ) {
        let (width, height) = matrix_width_height(matrix);

        // if there's no inverse, return garbage value - it's bad, but better than crashing
        let inverse = matrix.try_invert().unwrap_or(matrix.clone());
        let (dx, dy) = inverse.transform_distance(x1 - x0, y1 - y0);

        match self {
            Self::Sterile => (),
            Self::Translate => {
                matrix.translate(dx, dy);
            },
            Self::ExpandUpLeft => {
                let (sx, sy) = (1.0 - dx, 1.0 - dy);
                matrix.translate(1.0 - sx, 1.0 - sy);
                matrix.scale(sx, sy);
            },
            Self::ExpandUpRight => {
                let (sx, sy) = (1.0 + dx, 1.0 - dy);
                matrix.translate(0.0, 1.0 - sy);
                matrix.scale(sx, sy);
            }
            Self::ExpandDownLeft => {
                let (sx, sy) = (1.0 - dx, 1.0 + dy);
                matrix.translate(1.0 - sx, 0.0);
                matrix.scale(sx, sy);
            }
            Self::ExpandDownRight => {
                matrix.scale(1.0 + dx, 1.0 + dy);
            },
            Self::ExpandUp => {
                let sy = 1.0 - dy;
                matrix.translate(0.0, 1.0 - sy);
                matrix.scale(1.0, sy);
            }
            Self::ExpandDown => {
                matrix.scale(1.0, 1.0 + dy);
            }
            Self::ExpandLeft => {
                let sx = 1.0 - dx;
                matrix.translate(1.0 - sx, 0.0);
                matrix.scale(sx, 1.0);
            }
            Self::ExpandRight => {
                matrix.scale(1.0 + dx, 1.0);
            }
            Self::Rotate => {
                let (x0, y0) = (0.5, 0.0);
                let (x1, y1) = inverse.transform_point(x1, y1);
                let (x2, y2) = (0.5, 0.5);
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

                // dot product
                let dp = v0.0 * v1.0 + v0.1 * v1.1;

                // magnitude
                let m0 = (v0.0.powi(2) + v0.1.powi(2)).sqrt();
                let m1 = (v1.0.powi(2) + v1.1.powi(2)).sqrt();

                let a = (dp / (m0 * m1)).acos();

                // invert the direction, if necessary
                let a = if x0 >= x1 { -a } else { a };

                matrix.translate(0.5, 0.5);
                matrix.scale(1.0, width / height);
                matrix.rotate(a);
                matrix.scale(1.0, height / width);
                matrix.translate(-0.5, -0.5);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum FreeTransformMouseState {
    Up,
    Down(f64, f64, TransformationType),
}

#[derive(Clone, Copy)]
pub struct FreeTransformState {
    transform_mode: TransformMode,
    mouse_state: FreeTransformMouseState,
}

impl FreeTransformState {
    pub fn default(canvas: &mut Canvas) -> FreeTransformState {
        canvas.try_consume_selection_to_transformable()
            .map(|transform_mode| {
                FreeTransformState {
                    transform_mode,
                    mouse_state: FreeTransformMouseState::Up,
                }
            })
            .unwrap_or(Self::default_no_canvas())
    }

    pub const fn default_no_canvas() -> FreeTransformState {
        FreeTransformState {
            transform_mode: TransformMode::NotTransforming,
            mouse_state: FreeTransformMouseState::Up,
        }
    }
}

/// The effective width and height of a matrix's
/// unit square
fn matrix_width_height(matrix: &cairo::Matrix) -> (f64, f64) {
    // actual coordinates of the unit square's corners
    let p00 = matrix.transform_point(0.0, 0.0);
    let p10 = matrix.transform_point(1.0, 0.0);
    let p01 = matrix.transform_point(0.0, 1.0);

    (
        point_tuple_dist(p00, p10),
        point_tuple_dist(p00, p01),
    )
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
        if let TransformMode::Transforming(matrix) = &self.transform_mode {
            if let Some(transformable) = canvas.transformable().borrow_mut().as_mut() {
                    // cursor
                    let cursor_pos = canvas.cursor_pos_pix_f();
                    canvas.drawing_area().set_cursor(
                        TransformationType::from_matrix_and_point(matrix, cursor_pos, *canvas.zoom())
                            .cursor(matrix).as_ref()
                    );
            }
        }
    }

    fn draw(&self, canvas: &Canvas, cr: &cairo::Context, _toolbar: &mut Toolbar) {
        if let TransformMode::Transforming(matrix) = &self.transform_mode {
            if let Some(transformable) = canvas.transformable().borrow_mut().as_mut() {
                let _ = cr.save();
                {
                    cr.set_matrix(cairo::Matrix::multiply(matrix, &cr.matrix()));
                    transformable.draw(cr);

                    let (width, height) = matrix_width_height(matrix);
                    Self::draw_transform_overlay(cr, *canvas.zoom(), width, height);
                }
                let _ = cr.restore();
            } else {
                // `canvas.transformable` is gone - this shouldn't happen,
                // but this state isn't destructive/unrecoverable,
                // so just do nothing
            }
        }
    }

    fn handle_drag_start(&mut self, _mod_keys: &gtk::gdk::ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let TransformMode::Transforming(matrix) = self.transform_mode {
            let (x, y) = canvas.cursor_pos_pix_f();

            self.mouse_state = FreeTransformMouseState::Down(x, y,
                TransformationType::from_matrix_and_point(&matrix, (x, y), *canvas.zoom())
            )
        }
    }

    fn handle_drag_update(&mut self, _mod_keys: &gtk::gdk::ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let TransformMode::Transforming(ref mut matrix) = &mut self.transform_mode {
            if let FreeTransformMouseState::Down(x0, y0, transform_type) = self.mouse_state {
                let (x, y) = canvas.cursor_pos_pix_f();
                let dx = x - x0;
                let dy = y - y0;

                transform_type.update_matrix_with_point_diff(matrix, (x0, y0), (x, y));
                self.mouse_state = FreeTransformMouseState::Down(x, y, transform_type);
                canvas.update();
            }
        }
    }

    fn handle_drag_end(&mut self, _mod_keys: &gtk::gdk::ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        self.mouse_state = FreeTransformMouseState::Up;
    }
}
