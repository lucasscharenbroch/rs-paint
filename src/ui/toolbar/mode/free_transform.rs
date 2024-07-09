use super::{Canvas, Toolbar};
use gtk::cairo;

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

    fn update_matrix_with_diff(&self, matrix: &mut cairo::Matrix, dx: f64, dy: f64) {
        match self {
            Self::Sterile => (), // TODO
            Self::Translate => {
                matrix.translate(dx, dy);
            },
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

impl super::MouseModeState for FreeTransformState {
    fn draw(&self, canvas: &Canvas, cr: &cairo::Context, _toolbar: &mut Toolbar) {
        if let TransformMode::Transforming(matrix) = &self.transform_mode {
            if let Some(transformable) = canvas.transformable().borrow_mut().as_mut() {
                let _ = cr.save();
                {
                    cr.set_matrix(cairo::Matrix::multiply(matrix, &cr.matrix()));
                    transformable.draw(cr);

                    // TODO method to draw pretty border,
                    // scaled to zoom
                    cr.set_line_width(0.01);
                    cr.set_source_rgb(0.0, 0.0, 1.0);
                    cr.rectangle(0.0, 0.0, 1.0, 1.0);
                    let _ = cr.stroke();
                }
                let _ = cr.restore();
            } else {
                // canvas.transformable is gone - this shouldn't happen,
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
                const SENSITIVITY: f64 = 0.01;

                // TODO scale by matrix zoom level
                let dx = (x - x0) * SENSITIVITY;
                let dy = (y - y0) * SENSITIVITY;

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
