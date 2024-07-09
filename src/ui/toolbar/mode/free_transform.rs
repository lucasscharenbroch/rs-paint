use super::{Canvas, Toolbar};
use gtk::cairo;

#[derive(Clone, Copy)]
pub enum TransformMode {
    /// Nothing selected, don't do anything
    NotTransforming,
    Transforming(cairo::Matrix),
}

#[derive(Clone, Copy)]
pub struct FreeTransformState {
    transform_mode: TransformMode,
}

impl FreeTransformState {
    pub fn default(canvas: &mut Canvas) -> FreeTransformState {
        canvas.try_consume_selection_to_transformable()
            .map(|transform_mode| {
                FreeTransformState {
                    transform_mode,
                }
            })
            .unwrap_or(Self::default_no_canvas())
    }

    pub const fn default_no_canvas() -> FreeTransformState {
        FreeTransformState {
            transform_mode: TransformMode::NotTransforming
        }
    }
}

impl super::MouseModeState for FreeTransformState {
    fn draw(&self, canvas: &Canvas, cr: &cairo::Context, _toolbar: &mut Toolbar) {
        if let TransformMode::Transforming(matrix) = &self.transform_mode {
            let _ = cr.save();
            {
                cr.set_matrix(cairo::Matrix::multiply(matrix, &cr.matrix()));
                cr.set_source_rgb(0.0, 0.0, 1.0);
                cr.rectangle(0.0, 0.0, 1.0, 1.0);
                // let _ = cr.stroke();
                let _ = cr.fill();
            }
            let _ = cr.restore();
        }
    }
}
