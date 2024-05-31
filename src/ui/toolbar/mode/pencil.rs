use super::{Canvas, Toolbar};
use crate::image::ImageLike;

use std::collections::HashSet;
use gtk::gdk::ModifierType;
use gtk::cairo::{Context, LineCap};

#[derive(Clone, Copy)]
enum PencilMode {
    TraceCursor,
    DrawLineCooldown,
}

#[derive(Clone, Copy)]
pub struct PencilState {
    last_cursor_pos_pix: (f64, f64),
    mode: PencilMode,
    // Pencil strokes are formed by drawing `BrushImage`s
    // periodically on points along the line segments the mouse follows.
    // When segments get really small, we don't want to sample at every segment
    // (else the line gets too dark), so `dist_till_resample` is used to maintain
    // a uniform sampling-to-distance, independent of the number of segments.
    dist_till_resample: f64,
}

impl PencilState {
    pub fn default(_canvas: &Canvas) -> PencilState {
        Self::default_no_canvas()
    }

    pub fn default_no_canvas() -> PencilState {
        PencilState {
            last_cursor_pos_pix: (0.0, 0.0),
            mode: PencilMode::TraceCursor,
            dist_till_resample: 0.0,
        }
    }

    fn draw_line_between(&mut self, line_pt0: (f64, f64), line_pt1: (f64, f64), canvas: &mut Canvas, toolbar: &mut Toolbar) {
        // half the distance, for now
        let dx = line_pt0.0 - line_pt1.0;
        let dy = line_pt0.1 - line_pt1.1;
        let d = (dx.powi(2) + dy.powi(2)).sqrt();

        self.dist_till_resample -= d;
        if self.dist_till_resample > 0.0 {
            return; // no points to draw
        }

        let blending_mode = toolbar.get_blending_mode();
        let brush = toolbar.get_brush();

        const SAMPLE_DIST_FACTOR: f64 = 0.1;
        // distance (in pixels) between two samples
        let sample_distance = (brush.radius() as f64).powf(1.05) * SAMPLE_DIST_FACTOR;
        let num_points = (-self.dist_till_resample / sample_distance).floor() as usize + 1;
        self.dist_till_resample = self.dist_till_resample % sample_distance + sample_distance;

        let target_pixels = pixels_along_segment(line_pt0, line_pt1, num_points);

        target_pixels.iter().for_each(|&(x, y)| {
            let x_offset = (brush.image.width() as i32 - 1) / 2;
            let y_offset = (brush.image.height() as i32 - 1) / 2;
            canvas.image().sample(&brush.image, &blending_mode, x as i32 - x_offset, y as i32 - y_offset);
        });
    }

    fn draw_to_cursor(&mut self, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let line_pt0 = self.last_cursor_pos_pix;
        let line_pt1 = canvas.cursor_pos_pix();
        self.last_cursor_pos_pix = line_pt1;

        self.draw_line_between(line_pt0, line_pt1, canvas, toolbar);

        canvas.update();
    }

    fn straight_line_visual_cue_fn(&mut self, canvas: &Canvas) -> Box<dyn Fn(&Context)> {
        let zoom = *canvas.zoom();

        let (x0, y0) = self.last_cursor_pos_pix;
        let (x1, y1) = canvas.cursor_pos_pix();

        Box::new(move |cr| {
            const LINE_WIDTH: f64 = 3.0;
            const LINE_BORDER_FACTOR: f64 = 0.4;

            cr.set_line_cap(LineCap::Round);
            cr.set_line_width(LINE_WIDTH / zoom);

            cr.set_source_rgba(0.25, 0.25, 0.25, 0.75);
            cr.move_to(x0, y0);
            cr.line_to(x1, y1);
            let _ = cr.stroke();

            cr.set_line_width(LINE_WIDTH / zoom * LINE_BORDER_FACTOR);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.75);
            cr.move_to(x0, y0);
            cr.line_to(x1, y1);
            let _ = cr.stroke();
        })
    }
}

// given a continuous line segment, return the given number
// of discrete points (pixels) that "intersect" it
fn pixels_along_segment(
    (x0, y0): (f64, f64),
    (x1, y1): (f64, f64),
    num_pix: usize,
) -> Vec<(usize, usize)> {
    let total_dx = x1 - x0;
    let total_dy = y1 - y0;

    let dx = total_dx / (num_pix as f64);
    let dy = total_dy / (num_pix as f64);

    (0..num_pix).map(|i| {
        let i = i as f64;
        let x = x0 + dx * i;
        let y = y0 + dy * i;
        (x as usize, y as usize)
    }).collect::<Vec<_>>()
}

impl super::MouseModeState for PencilState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.dist_till_resample = 0.0;
        if mod_keys.intersects(ModifierType::SHIFT_MASK) {
            self.draw_to_cursor(canvas, toolbar);
            self.mode = PencilMode::DrawLineCooldown;
        } else {
            self.mode = PencilMode::TraceCursor;
        }

        self.last_cursor_pos_pix = canvas.cursor_pos_pix();
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        match self.mode {
            PencilMode::DrawLineCooldown => (), // line already drawn
            PencilMode::TraceCursor => self.draw_to_cursor(canvas, toolbar),
        }
    }

    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.handle_drag_update(mod_keys, canvas, toolbar);
        canvas.save_state_for_undo();
    }

    fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        if mod_keys.intersects(ModifierType::SHIFT_MASK) {
            canvas.update_with(self.straight_line_visual_cue_fn(canvas));
        } else {
            canvas.update();
        }
    }

    fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.handle_motion(mod_keys, canvas, toolbar)
    }
}
