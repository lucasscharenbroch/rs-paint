use crate::image::bitmask::ImageBitmask;
use super::canvas::Canvas;
use crate::util::Iterable;

use gtk::cairo::{Context, Rectangle};
use itertools::Itertools;

pub enum Selection {
    Rectangle(usize, usize, usize, usize), // x, y, w, h
    Bitmask(ImageBitmask),
    NoSelection
}

fn draw_rect_sel(zoom: f64, x: usize, y: usize, w: usize, h: usize, cr: &Context) {
    const DASH_LENGTH: f64 = 10.0;
    const BORDER_WIDTH: f64 = 3.0;

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

fn draw_sel_mask(image_height: usize, image_width: usize, selection_mask: &mut ImageBitmask, cr: &Context) {
    assert!(image_width == selection_mask.width());
    assert!(image_height == selection_mask.height());

    let path = selection_mask.edge_path(cr);
    cr.new_path();
    cr.append_path(path);

    cr.set_fill_rule(gtk::cairo::FillRule::EvenOdd);
    cr.set_source_rgb(0.0, 0.0, 1.0);
    let _ = cr.fill_preserve();

    cr.set_source_rgb(1.0, 0.0, 0.0);
    let _ = cr.stroke();
}

impl Canvas {
    pub fn draw_selection_outline(&mut self, cr: &Context) {
        let image_height = self.image_height() as usize;
        let image_width = self.image_width() as usize;

        match self.selection {
            Selection::Rectangle(x, y, w, h) => draw_rect_sel(*self.zoom(), x, y, w, h, cr),
            Selection::Bitmask(ref mut selection_mask) => draw_sel_mask(
                image_height,
                image_width,
                selection_mask,
                cr
            ),
            Selection::NoSelection => (),
        }
    }
}

impl Iterable for Selection {
    type Item = (usize, usize);

    fn iter(&self) -> Box<dyn Iterator<Item = (usize, usize)>> {
        match self {
            Self::Rectangle(x, y, w, h) => {
                let xs = *x..(x + w);
                let ys = *y..(y + h);
                return Box::new(ys.cartesian_product(xs));
            },
            Self::Bitmask(selection_mask) => {
                return Box::new(selection_mask.coords_of_active_bits().into_iter());
            },
            Self::NoSelection => {
                return Box::new(std::iter::empty());
            },
        }
    }
}
