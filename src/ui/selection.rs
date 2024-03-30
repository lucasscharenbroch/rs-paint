use super::canvas::Canvas;

use gtk::cairo::Context;

pub enum Selection {
    Rectangle(usize, usize, usize, usize), // x, y, w, h
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
    cr.stroke();

    cr.set_dash(&[DASH_LENGTH / zoom, DASH_LENGTH / zoom], DASH_LENGTH / zoom);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.rectangle(x as f64, y as f64, w as f64, h as f64);
    cr.stroke();

    cr.set_dash(&[], 0.0);
}

impl Selection {
    pub fn draw_outline(&self, canvas: &Canvas, cr: &Context) {
        match self {
            Self::Rectangle(x, y, w, h) => draw_rect_sel(canvas, x, y, w, h, cr),
            Self::NoSelection => (),
        }
    }
}
