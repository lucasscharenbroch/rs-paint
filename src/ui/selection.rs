use crate::image::bitmask::ImageBitmask;
use super::canvas::Canvas;

use gtk::cairo;
use itertools::Itertools;

pub enum Selection {
    Rectangle(usize, usize, usize, usize), // x, y, w, h
    Bitmask(ImageBitmask),
    NoSelection
}

macro_rules! set_selection_fill_color {
    ($cr:expr) => {
        $cr.set_source_rgba(0.3, 0.6, 1.0, 0.5);
    };
}

macro_rules! set_selection_outline_color {
    ($cr:expr) => {
        $cr.set_source_rgb(0.2, 0.2, 7.0);
    };
}

fn draw_rect_sel(zoom: f64, x: usize, y: usize, w: usize, h: usize, cr: &cairo::Context) {
    const BORDER_WIDTH: f64 = 3.0;

    cr.set_line_width(BORDER_WIDTH / zoom);

    cr.rectangle(x as f64, y as f64, w as f64, h as f64);

    set_selection_fill_color!(cr);
    let _ = cr.fill_preserve();

    set_selection_outline_color!(cr);
    let _ = cr.stroke();
}

fn draw_sel_mask(zoom: f64, image_height: usize, image_width: usize, selection_mask: &mut ImageBitmask, cr: &cairo::Context) {
    assert!(image_width == selection_mask.width());
    assert!(image_height == selection_mask.height());

    let path = selection_mask.edge_path(cr);
    cr.new_path();
    cr.append_path(path);

    cr.set_fill_rule(gtk::cairo::FillRule::EvenOdd);
    set_selection_fill_color!(cr);
    let _ = cr.fill_preserve();

    const BORDER_WIDTH: f64 = 3.0;

    cr.set_line_width(BORDER_WIDTH / zoom);
    set_selection_outline_color!(cr);
    let _ = cr.stroke();
}

impl Canvas {
    pub fn draw_selection_outline(&mut self, cr: &cairo::Context) {
        let image_height = self.image_height() as usize;
        let image_width = self.image_width() as usize;
        let zoom = *self.zoom();

        match self.selection_mut() {
            Selection::Rectangle(x, y, w, h) => draw_rect_sel(zoom, *x, *y, *w, *h, cr),
            Selection::Bitmask(ref mut selection_mask) => draw_sel_mask(
                zoom,
                image_height,
                image_width,
                selection_mask,
                cr
            ),
            Selection::NoSelection => (),
        }
    }
}

impl Selection {
    pub fn iter(&self) -> Box<dyn Iterator<Item = (usize, usize)> + '_> {
        match self {
            Self::Rectangle(x, y, w, h) => {
                let xs = *x..(x + w);
                let ys = *y..(y + h);
                return Box::new(ys.cartesian_product(xs));
            },
            Self::Bitmask(selection_mask) => {
                return selection_mask.coords_of_active_bits();
            },
            Self::NoSelection => {
                return Box::new(std::iter::empty());
            },
        }
    }
}
