use crate::image::selection::ImageBitmask;
use super::canvas::Canvas;

use gtk::cairo::Context;

pub enum Selection {
    Rectangle(usize, usize, usize, usize), // x, y, w, h
    Bitmask(ImageBitmask),
    NoSelection
}

fn draw_rect_sel(canvas: &Canvas, &x: &usize, &y: &usize, &w: &usize, &h: &usize, cr: &Context) {
    const DASH_LENGTH: f64 = 10.0;
    const BORDER_WIDTH: f64 = 3.0;
    let zoom = canvas.zoom();

    cr.set_line_width(BORDER_WIDTH / zoom);

    cr.set_dash(&[DASH_LENGTH / zoom, DASH_LENGTH / zoom], 0.0);
    cr.set_source_rgb(1.0, 1.0, 0.0);
    cr.rectangle(x as f64, y as f64, w as f64, h as f64);
    let _ = cr.stroke();

    cr.set_dash(&[DASH_LENGTH / zoom, DASH_LENGTH / zoom], DASH_LENGTH / zoom);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.rectangle(x as f64, y as f64, w as f64, h as f64);
    let _ = cr.stroke();

    cr.set_dash(&[], 0.0);
}

fn draw_sel_mask(canvas: &Canvas, selection_mask: &ImageBitmask, cr: &Context) {
    let w = canvas.image_width() as usize;
    let h = canvas.image_height() as usize;
    assert!(w == selection_mask.width());
    assert!(h == selection_mask.height());

    // TODO change this
    for (r, c) in selection_mask.coords_of_active_bits().iter() {
        if r % 5 != 0 || c % 5 != 0 {
            continue;
        }
        cr.set_source_rgb(0.0, 0.0, 1.0);
        cr.rectangle(*c as f64, *r as f64, 1.0, 1.0);
        let _ = cr.fill();
    }
}

impl Selection {
    pub fn draw_outline(&self, canvas: &Canvas, cr: &Context) {
        match self {
            Self::Rectangle(x, y, w, h) => draw_rect_sel(canvas, x, y, w, h, cr),
            Self::Bitmask(selection_mask) => draw_sel_mask(canvas, selection_mask, cr),
            Self::NoSelection => (),
        }
    }
}
