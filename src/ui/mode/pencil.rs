use super::Canvas;
use super::super::super::image::{mk_test_brush};

use std::collections::HashSet;

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
    let max_x = x0.max(x1);
    let min_x = x0.min(x1);
    let max_y = y0.max(y1);
    let min_y = y0.min(y1);

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
    fn handle_drag_start(&mut self, canvas: &mut Canvas) {
        self.last_cursor_pos_pix = canvas.cursor_pos_pix();
    }

    fn handle_drag_update(&mut self, canvas: &mut Canvas) {
        let line_pt0= self.last_cursor_pos_pix;
        let line_pt1 = canvas.cursor_pos_pix();
        self.last_cursor_pos_pix = line_pt1;

        let target_pixels = pixels_on_segment(line_pt0, line_pt1);
        let brush = mk_test_brush();

        target_pixels.iter().for_each(|&(x, y)| {
            canvas.image().sample(&brush, x as i32 - 3, y as i32 - 3);
        });

        canvas.update();
    }

    fn handle_drag_end(&mut self, canvas: &mut Canvas) {
        self.handle_drag_update(canvas)
    }
}
