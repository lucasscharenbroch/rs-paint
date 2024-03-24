use super::Canvas;
use super::super::super::image::{mk_test_brush};

#[derive(Clone, Copy)]
pub struct PencilState {
    last_cursor_pos_pix: (f64, f64),
}

impl PencilState {
    pub const fn default() -> PencilState {
        PencilState {
            last_cursor_pos_pix: (0.0, 0.0),
        }
    }
}

impl super::MouseModeState for PencilState {

    fn handle_drag_start(&mut self, canvas: &mut Canvas) {
        self.last_cursor_pos_pix = canvas.cursor_pos_pix();
    }

    fn handle_drag_update(&mut self, canvas: &mut Canvas) {
        // calculate line, sample for each pixel-center-point on line

        let (x, y) = self.last_cursor_pos_pix;

        let brush = mk_test_brush();
        canvas.image().sample(&brush, x as i32 - 3, y as i32 - 3);
        canvas.update();

        self.last_cursor_pos_pix = canvas.cursor_pos_pix();
    }

    fn handle_drag_end(&mut self, canvas: &mut Canvas) {
        self.handle_drag_update(canvas)
    }
}
