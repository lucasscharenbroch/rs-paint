mod bezier;

use super::{cursor, Canvas, MouseModeVariant, Toolbar};
use crate::image::{brush::Brush, ImageLike};
use crate::image::undo::action::ActionName;
use crate::ui::form::Form;
use bezier::{IncrementalBezierSnapshot, BezierSegment};

use std::collections::HashMap;
use gtk::gdk::{ModifierType, RGBA};
use gtk::cairo::{Context, LineCap};

#[derive(Clone, Copy)]
enum PencilMode {
    PencilUp,
    PencilDown,
}

#[derive(Clone, Copy)]
pub struct PencilState {
    /// The (real-number) pixel coordinates of the
    /// last-drawn-to position
    last_cursor_pos_pix: (f64, f64),
    mode: PencilMode,
    // Pencil strokes are formed by drawing `BrushImage`s
    // periodically on points along the line segments the mouse follows.
    // When segments get really small, we don't want to sample at every segment
    // (else the line gets too dark), so `dist_till_resample` is used to maintain
    // a uniform sampling-to-distance, independent of the number of segments.
    dist_till_resample: f64,
    /// Serves as a queue of control points before the bezier
    /// segment can be eagerly drawn
    bezier_snapshot: IncrementalBezierSnapshot,
}

impl PencilState {
    pub fn default(canvas: &Canvas) -> PencilState {
        let last_cursor_pos_pix = canvas.last_cursor_pos_pix();
        PencilState {
            last_cursor_pos_pix,
            mode: PencilMode::PencilUp,
            dist_till_resample: 0.0,
            bezier_snapshot: IncrementalBezierSnapshot::NoPoints,
        }
    }

    pub fn default_no_canvas() -> PencilState {
        PencilState {
            last_cursor_pos_pix: (0.0, 0.0),
            mode: PencilMode::PencilUp,
            dist_till_resample: 0.0,
            bezier_snapshot: IncrementalBezierSnapshot::NoPoints,
        }
    }

    /// Claims `distance` of the draw-length
    /// (adjusts `self.dist_till_resample`), returning
    /// the number of sample points that lie along that distance
    fn get_and_claim_num_points_to_sample(&mut self, distance: f64, brush: &Brush) -> usize {
        self.dist_till_resample -= distance;
        if self.dist_till_resample > 0.0 {
            return 0; // no points to draw
        }

        const SAMPLE_DIST_FACTOR: f64 = 0.1;
        // distance (in pixels) between two samples
        let sample_distance = (brush.radius() as f64).powf(1.05) * SAMPLE_DIST_FACTOR;
        let res = (-self.dist_till_resample / sample_distance).floor() as usize + 1;
        self.dist_till_resample = self.dist_till_resample % sample_distance + sample_distance;

        return res;
    }

    fn draw_line_between(
        &mut self,
        line_pt0: (f64, f64),
        line_pt1: (f64, f64),
        canvas: &mut Canvas,
        toolbar: &mut Toolbar
    ) {
        let dx = line_pt0.0 - line_pt1.0;
        let dy = line_pt0.1 - line_pt1.1;
        let d = (dx.powi(2) + dy.powi(2)).sqrt();

        let blending_mode = toolbar.get_blending_mode();
        let brush = toolbar.get_brush();

        let num_points = self.get_and_claim_num_points_to_sample(d, brush);
        let target_pixels = pixels_along_segment(line_pt0, line_pt1, num_points);

        target_pixels.iter().for_each(|&(x, y)| {
            let x_offset = (brush.image.width() as i32 - 1) / 2;
            let y_offset = (brush.image.height() as i32 - 1) / 2;
            canvas.image().sample(&brush.image, &blending_mode, x as i32 - x_offset, y as i32 - y_offset);
        });

        // TODO remove
        let brush = crate::image::brush::Brush::new(
            RGBA::new(1.0, 0.0, 0.0, 1.0),
            crate::image::brush::BrushType::Round,
            5,
        );
        target_pixels.get(0).map(|(x, y)| {
            let x_offset = (brush.image.width() as i32 - 1) / 2;
            let y_offset = (brush.image.height() as i32 - 1) / 2;
            canvas.image().sample(&brush.image, &blending_mode, *x as i32 - x_offset, *y as i32 - y_offset);
        });
    }

    fn draw_straight_line_to_cursor(&mut self, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let line_pt0 = self.last_cursor_pos_pix;
        let line_pt1 = canvas.cursor_pos_pix_f();
        self.last_cursor_pos_pix = line_pt1;

        self.draw_line_between(line_pt0, line_pt1, canvas, toolbar);

        canvas.update();
    }

    fn draw_to_cursor(&mut self, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let new_point = canvas.cursor_pos_pix_u();

        // We're starting a new stroke: draw a single brush sample
        // at the cursor to not leave the user hanging
        if let IncrementalBezierSnapshot::NoPoints = self.bezier_snapshot {
            let target_pixels = vec![
                canvas.cursor_pos_pix_f()
                ].into_iter()
                // filter out the negatives, else they'll be converted to 0
                // (and stick to the side of the image)
                .filter(|(x, y)| *x > 0.0 && *y > 0.0)
                .map(|(x, y)| (x as usize, y as usize))
                .collect::<Vec<_>>();

            let blending_mode = toolbar.get_blending_mode();
            let brush = toolbar.get_brush();

            target_pixels.iter().for_each(|&(x, y)| {
                let x_offset = (brush.image.width() as i32 - 1) / 2;
                let y_offset = (brush.image.height() as i32 - 1) / 2;
                canvas.image().sample(&brush.image, &blending_mode, x as i32 - x_offset, y as i32 - y_offset);
            });
        }

        if let Some(segment) = self.bezier_snapshot.append_point(new_point) {
            self.last_cursor_pos_pix = (segment.x2 as f64, segment.y2 as f64);
            let d = segment.rough_length();

            let blending_mode = toolbar.get_blending_mode();
            let brush = toolbar.get_brush();

            let num_points = self.get_and_claim_num_points_to_sample(d, brush);
            let target_pixels = segment.sample_n_pixels(num_points);

            target_pixels.iter().for_each(|&(x, y)| {
                let x_offset = (brush.image.width() as i32 - 1) / 2;
                let y_offset = (brush.image.height() as i32 - 1) / 2;
                canvas.image().sample(&brush.image, &blending_mode, x as i32 - x_offset, y as i32 - y_offset);
            });

            // TODO remove
            let brush = crate::image::brush::Brush::new(
                RGBA::new(0.0, 0.0, 1.0, 1.0),
                crate::image::brush::BrushType::Round,
                5,
            );
            target_pixels.get(0).map(|(x, y)| {
                let x_offset = (brush.image.width() as i32 - 1) / 2;
                let y_offset = (brush.image.height() as i32 - 1) / 2;
                canvas.image().sample(&brush.image, &blending_mode, *x as i32 - x_offset, *y as i32 - y_offset);
            });
        }
    }

    fn complete_curve(&mut self, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        match self.bezier_snapshot {
            IncrementalBezierSnapshot::NoPoints => (),
            IncrementalBezierSnapshot::One(pt) => {
                self.last_cursor_pos_pix = (pt.0 as f64, pt.1 as f64);
                self.draw_straight_line_to_cursor(canvas, toolbar)
            },
            IncrementalBezierSnapshot::Two(_, _) => self.draw_to_cursor(canvas, toolbar),
        }
        self.bezier_snapshot = IncrementalBezierSnapshot::NoPoints;
    }

    fn straight_line_visual_cue_fn(&mut self, canvas: &Canvas) -> Box<dyn Fn(&Context)> {
        let zoom = *canvas.zoom();

        let (x0, y0) = self.last_cursor_pos_pix;
        let (x1, y1) = canvas.cursor_pos_pix_f();

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

    pub fn set_last_cursor_pos_pix(&mut self, value: (f64, f64)) {
        self.last_cursor_pos_pix = value;
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
        (x, y)
    })
        // filter out the negatives, else they'll be converted to 0
        // (and stick to the side of the image)
        .filter(|(x, y)| *x > 0.0 && *y > 0.0)
        .map(|(x, y)| (x as usize, y as usize))
        .collect::<Vec<_>>()
}

impl super::MouseModeState for PencilState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        self.dist_till_resample = 0.0;
        if mod_keys.intersects(ModifierType::SHIFT_MASK) {
            self.draw_straight_line_to_cursor(canvas, toolbar);
            self.mode = PencilMode::PencilUp;
        } else {
            self.mode = PencilMode::PencilDown;
        }

        self.last_cursor_pos_pix = canvas.cursor_pos_pix_f();
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        match self.mode {
            PencilMode::PencilDown => self.draw_to_cursor(canvas, toolbar),
            PencilMode::PencilUp => (), // line already drawn
        }
    }

    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        match self.mode {
            PencilMode::PencilDown => {
                self.complete_curve(canvas, toolbar);
                canvas.save_state_for_undo(ActionName::Pencil);
                self.mode = PencilMode::PencilUp;
            },
            PencilMode::PencilUp => (),
        }
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

    fn draw(&self, canvas: &Canvas, cr: &Context, toolbar: &mut Toolbar) {
        let cursor_pos = canvas.cursor_pos_pix_f();
        let cursor_pos = (cursor_pos.0.floor(), cursor_pos.1.floor());

        let brush = toolbar.get_brush_mut();
        let x_offset = (brush.image.width() as i32 - 1) / 2;
        let y_offset = (brush.image.height() as i32 - 1) / 2;
        let path = brush.outline_path(cr);
        let _ = cr.save();
        {
            cr.translate(cursor_pos.0 - x_offset as f64, cursor_pos.1 - y_offset as f64);
            cr.new_path();
            cr.append_path(path);
            cr.set_source_rgb(0.0, 1.0, 0.0);
            let _ = cr.stroke();
        }
        let _ = cr.restore();
    }
}
