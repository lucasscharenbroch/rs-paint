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
    /// Pan the canvas
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

impl TransformationType {
    fn from_matrix_and_point(matrix: &cairo::Matrix, (x, y): (f64, f64)) -> Self {
        // TODO combine with zoom

        let (x0, y0) = matrix.transform_point(0.0, 0.0);
        let (x1, y1) = matrix.transform_point(1.0, 1.0);

        if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
            TransformationType::Translate
        } else {
            TransformationType::Sterile
        }
    }

    /// The gdk::Cursor associated with this transformation
    fn cursor(&self) -> Option<gdk::Cursor> {
        gdk::Cursor::from_name(
            match self {
                TransformationType::Sterile => "default",
                TransformationType::Translate => "move",
                TransformationType::ExpandUpLeft => "nwse-resize",
                TransformationType::ExpandUpRight => "nesw-resize",
                TransformationType::ExpandDownLeft => "nesw-resize",
                TransformationType::ExpandDownRight => "nwse-resize",
                TransformationType::ExpandUp => "ns-resize",
                TransformationType::ExpandDown => "ns-resize",
                TransformationType::ExpandLeft => "ew-resize",
                TransformationType::ExpandRight => "ew-resize",
                TransformationType::Rotate => "grab", // this one isn't great...
            },
            gdk::Cursor::from_name("default", None).as_ref(),
        )
    }

    fn update_matrix_with_diff(&self, matrix: &mut cairo::Matrix, dx: f64, dy: f64) {
        match self {
            Self::Sterile => (), // TODO
            Self::Translate => {
                let (width, height) = matrix.transform_distance(1.0, 1.0);
                matrix.translate(dx / width, dy / height);
            },
            _ => todo!(),
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
                    canvas.drawing_area().set_cursor(TransformationType::from_matrix_and_point(matrix, cursor_pos).cursor().as_ref());
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

                    let (width, height) = matrix.transform_distance(1.0, 1.0);

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
                TransformationType::from_matrix_and_point(&matrix, (x, y))
            )
        }
    }

    fn handle_drag_update(&mut self, _mod_keys: &gtk::gdk::ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        if let TransformMode::Transforming(ref mut matrix) = &mut self.transform_mode {
            if let FreeTransformMouseState::Down(x0, y0, transform_type) = self.mouse_state {
                let (x, y) = canvas.cursor_pos_pix_f();
                let dx = x - x0;
                let dy = y - y0;

                transform_type.update_matrix_with_diff(matrix, dx, dy);
                self.mouse_state = FreeTransformMouseState::Down(x, y, transform_type);
                canvas.update();
            }
        }
    }

    fn handle_drag_end(&mut self, _mod_keys: &gtk::gdk::ModifierType, _canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        self.mouse_state = FreeTransformMouseState::Up;
    }
}
