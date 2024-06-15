use super::{Image, ImageLike, Pixel};
use super::blend::BlendingMode;

use gtk::gdk::RGBA;
use gtk::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BrushType {
    Square,
    Round,
    Dither,
    Pen, // faded round
    Crayon, // faded dither
}

#[derive(PartialEq)]
struct BrushProperties {
    brush_type: BrushType,
    radius: u8,
    color: RGBA,
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
    pub image: BrushImage,
}

fn mk_square_brush_image(n: u8, color: RGBA) -> BrushImage {
    let p = Pixel::from_rgba_struct(color);
    BrushImage::from_pixels_options(vec![vec![Some(p); n as usize]; n as usize])
}

fn mk_round_brush_image(n: u8, fade: bool, dither: bool, color: RGBA) -> BrushImage {
    let p = Pixel::from_rgba_struct(color);
    let n = n as usize;
    let mut pix = vec![vec![None; n]; n];

    const CIRC_THRESH: f64 = 0.15;

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
            }
        }
    }

    BrushImage::from_pixels_options(pix)
}

impl Brush {
    fn image_from_props(props: &BrushProperties) -> BrushImage {
        let r = props.radius;
        match props.brush_type {
            BrushType::Square => mk_square_brush_image(r, props.color),
            BrushType::Round => mk_round_brush_image(r, false, false, props.color),
            BrushType::Dither => mk_round_brush_image(r, false, true, props.color),
            BrushType::Pen => mk_round_brush_image(r, true, false, props.color),
            BrushType::Crayon => mk_round_brush_image(r, true, true, props.color),
        }
    }

    pub fn modify(&mut self, color: RGBA, brush_type: BrushType, radius: u8) {
        let new_props = BrushProperties {
            color,
            brush_type,
            radius,
        };

        if self.props == new_props {
            // no changes necessary
        } else {
            *self = Brush::from_props(new_props)
        }
    }

    pub fn new(color: RGBA, brush_type: BrushType, radius: u8) -> Self {
        let props = BrushProperties {
            color,
            brush_type,
            radius,
        };

        Self::from_props(props)
    }

    fn from_props(props: BrushProperties) -> Self {
        let image = Self::image_from_props(&props);

        Brush {
            props,
            image,
        }
    }

    pub fn radius(&self) -> usize {
        self.props.radius as usize
    }
}
