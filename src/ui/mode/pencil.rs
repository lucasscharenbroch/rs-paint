use super::Canvas;
use super::super::super::image::{mk_test_brush};

use std::collections::HashSet;
use gtk::gdk::{ModifierType};
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
}

impl PencilState {
    pub const fn default() -> PencilState {
        PencilState {
            last_cursor_pos_pix: (0.0, 0.0),
            mode: PencilMode::TraceCursor,
        }
    }

    fn draw_line_between(&self, line_pt0: (f64, f64), line_pt1: (f64, f64), canvas: &mut Canvas) {
        let target_pixels = pixels_on_segment(line_pt0, line_pt1);
        let brush = mk_test_brush();

        target_pixels.iter().for_each(|&(x, y)| {
            canvas.image().sample(&brush, x as i32 - 3, y as i32 - 3);
        });
    }

    fn draw_to_cursor(&mut self, canvas: &mut Canvas) {
        let line_pt0 = self.last_cursor_pos_pix;
        let line_pt1 = canvas.cursor_pos_pix();
        self.last_cursor_pos_pix = line_pt1;

        self.draw_line_between(line_pt0, line_pt1, canvas);

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
            cr.stroke();

            cr.set_line_width(LINE_WIDTH / zoom * LINE_BORDER_FACTOR);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.75);
            cr.move_to(x0, y0);
            cr.line_to(x1, y1);
            cr.stroke();
        })
    }
}

fn dfs_pix_where(
    x0: usize,
    y0: usize,
    pred: &dyn Fn(usize, usize) -> bool,
    vis: &mut HashSet<(usize, usize)>
) {
    vis.insert((x0, y0));

    let x0 = x0 as i32;
    let y0 = y0 as i32;

    let surrounding = vec![
        (x0 + 1, y0),
        (x0, y0 + 1),
        (x0 - 1, y0),
        (x0, y0 - 1),
    ];

    surrounding.iter().for_each(|&(x, y)| {
        if x < 0 || y < 0 {
            return;
        }

        let x = x as usize;
        let y = y as usize;

        if vis.contains(&(x, y)) || !pred(x, y) {
            return;
        }

        dfs_pix_where(x, y, pred, vis);
    });
}

// given a continuous line segment, return the set of
// discrete pixels that intersect it
fn pixels_on_segment((x0, y0): (f64, f64), (x1, y1): (f64, f64)) -> HashSet<(usize, usize)> {
    let max_x = x0.max(x1).floor();
    let min_x = x0.min(x1).floor();
    let max_y = y0.max(y1).floor();
    let min_y = y0.min(y1).floor();

    let pt_direction = move |(px, py): (f64, f64)| -> bool {
        // dot-product of normal-vector of segment <y1 - y0, -(x1 - x0)>
        // and vector between the segment's first point and the given point <px - x0, py - y0>
        // sign flips when point crosses the line
        (px - x0) * (y1 - y0) - (py - y0) * (x1 - x0) >= 0.0
    };

    let pix_intersects_line = move |px: usize, py: usize| -> bool {
        let px = px as f64;
        let py = py as f64;

        let corners = vec![
            (px, py), // top left
            (px, py + 1.0),
            (px + 1.0, py),
            (px + 1.0, py + 1.0),
        ];

        let top_left_direction = pt_direction((px, py));

        // one should be within the bounding-box of the segment
        corners.iter().any(|&(x, y)| min_x <= x && x <= max_x && min_y <= y && y <= max_y)
        &&
        // can't all be on the same side of the segment
        !corners.iter().skip(1).all(|&pt| pt_direction(pt) == top_left_direction)
    };

    let mut vis = HashSet::new();
    dfs_pix_where(x0 as usize, y0 as usize, &pix_intersects_line, &mut vis);
    vis
}

impl super::MouseModeState for PencilState {
    fn handle_drag_start(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        if mod_keys.intersects(ModifierType::SHIFT_MASK) {
            self.draw_to_cursor(canvas);
            self.mode = PencilMode::DrawLineCooldown;
        } else {
            self.mode = PencilMode::TraceCursor;
        }

        self.last_cursor_pos_pix = canvas.cursor_pos_pix();
    }

    fn handle_drag_update(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas) {
        match self.mode {
            PencilMode::DrawLineCooldown => (), // line already drawn
            PencilMode::TraceCursor => self.draw_to_cursor(canvas),
        }
    }

    fn handle_drag_end(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        self.handle_drag_update(mod_keys, canvas);
        canvas.save_state_for_undo();
    }

    fn handle_motion(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        if mod_keys.intersects(ModifierType::SHIFT_MASK) {
            canvas.update_with(self.straight_line_visual_cue_fn(canvas));
        } else {
            canvas.update();
        }
    }

    fn handle_mod_key_update(&mut self, mod_keys: &ModifierType, canvas: &mut Canvas) {
        // mod_keys.intersects(SHIFT_MASK) is true when shift was just released,
        // and false when shift was just pressed.
        // v
        if !mod_keys.intersects(ModifierType::SHIFT_MASK) {
            canvas.update_with(self.straight_line_visual_cue_fn(canvas));
        } else {
            canvas.update();
        }
    }
}
