use super::bitmask::ImageBitmask;
use super::{ImageLike, Pixel};

use gtk::gdk::RGBA;
use gtk::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BrushType {
    Square,
    Round,
    Dither,
    Caligraphy,
}

#[derive(PartialEq)]
struct BrushProperties {
    brush_type: BrushType,
    radius: u8,
    primary_color: RGBA,
    secondary_color: RGBA,
}

pub struct BrushImage {
    pixel_options: Vec<Option<Pixel>>,
    width: usize,
    height: usize,
}

impl BrushImage {
    pub fn from_pixels_options(pixel_options: Vec<Vec<Option<Pixel>>>) -> Self {
        let width = pixel_options.len();
        let height = pixel_options[0].len();
        let pixel_options = pixel_options.into_iter().flatten().collect::<Vec<_>>();

        Self {
            width,
            height,
            pixel_options,
        }
    }
}

impl ImageLike for BrushImage {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    #[inline]
    fn try_pix_at(&self, r: usize, c: usize) -> Option<&Pixel> {
        self.pixel_options[r * self.width + c].as_ref()
    }
}

pub struct Brush {
    props: BrushProperties,
    pub brush_image: BrushImage,
    bitmask: ImageBitmask,
}

fn mk_square_brush_image(n: u8, primary_color: RGBA, _secondary_color: RGBA) -> BrushImage {
    let p = Pixel::from_rgba_struct(primary_color);
    BrushImage::from_pixels_options(vec![vec![Some(p); n as usize]; n as usize])
}

fn mk_round_brush_image(n: u8, fade: bool, dither: bool, primary_color: RGBA, secondary_color: RGBA) -> BrushImage {
    let p = Pixel::from_rgba_struct(primary_color);
    let p2 = Pixel::from_rgba_struct(secondary_color);
    let n = n as usize;
    let mut pix = vec![vec![None; n]; n];

    const CIRC_THRESH: f64 = 0.15;

    // include points closer than CIRC_THRESH to the center

    for i in 0..n {
        for j in 0..n {
            let x = (n as f64 / 2.0) - (j as f64 + 0.5);
            let y = (n as f64 / 2.0) - (i as f64 + 0.5);
            let dist = (x * x + y * y).sqrt();
            let opacity = 1.0 - (dist / (n as f64 / 2.0));

            if opacity > CIRC_THRESH  && (!dither || i % 2 == j % 2) {
                if fade {
                    pix[i][j] = Some(p.scale_alpha(opacity));
                } else {
                    pix[i][j] = Some(p.clone());
                }
            } else if opacity > CIRC_THRESH && dither {
                if fade {
                    pix[i][j] = Some(p2.scale_alpha(opacity));
                } else {
                    pix[i][j] = Some(p2.clone());
                }
            }
        }
    }

    BrushImage::from_pixels_options(pix)
}

fn mk_caligraphy_brush_image(n: u8, primary_color: RGBA, _secondary_color: RGBA) -> BrushImage {
    let p = Pixel::from_rgba_struct(primary_color);
    let n = n as usize;
    let mut pix = vec![vec![None; n]; n];

    // scuffed, but easier to change than some random formula
    let dist_thresh: f64 = match n {
        0..=2 => 1.0,
        3..=5 => 0.35,
        6 => 0.25,
        7.. => 0.15,
    };

    // include points closer than dist_thresh to the line (y=x)

    for i in 0..n {
        for j in 0..n {
            let diff = (i as f64 - j as f64) / n as f64;

            if diff.abs() < dist_thresh  {
                pix[i][j] = Some(p.clone());
            }
        }
    }

    BrushImage::from_pixels_options(pix)
}

impl Brush {
    fn brush_image_from_props(props: &BrushProperties) -> BrushImage {
        let r = props.radius;
        match props.brush_type {
            BrushType::Square => mk_square_brush_image(r, props.primary_color, props.secondary_color),
            BrushType::Round => mk_round_brush_image(r, false, false, props.primary_color, props.secondary_color),
            BrushType::Dither => mk_round_brush_image(r, false, true, props.primary_color, props.secondary_color),
            BrushType::Caligraphy => mk_caligraphy_brush_image(r, props.primary_color, props.secondary_color),
        }
    }

    pub fn modify(&mut self, primary_color: RGBA, secondary_color: RGBA, brush_type: BrushType, radius: u8) {
        let new_props = BrushProperties {
            primary_color,
            secondary_color,
            brush_type,
            radius,
        };

        if self.props == new_props {
            // no changes necessary
        } else {
            *self = Brush::from_props(new_props)
        }
    }

    pub fn new(primary_color: RGBA, secondary_color: RGBA, brush_type: BrushType, radius: u8) -> Self {
        let props = BrushProperties {
            primary_color,
            secondary_color,
            brush_type,
            radius,
        };

        Self::from_props(props)
    }

    fn from_props(props: BrushProperties) -> Self {
        let brush_image = Self::brush_image_from_props(&props);
        let bitmask = ImageBitmask::from_flat_bits(
            props.radius as usize,
            props.radius as usize,
            brush_image.pixel_options.iter()
                .map(|opt| opt.is_some())
                .collect::<Vec<_>>()
        );

        Brush {
            props,
            brush_image,
            bitmask,
        }
    }

    pub fn radius(&self) -> usize {
        self.props.radius as usize
    }

    pub fn outline_path(&mut self, cr: &gtk::cairo::Context) -> &gtk::cairo::Path {
        self.bitmask.edge_path(cr)
    }
}
