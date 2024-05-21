use super::{Image, Pixel};

use gtk::gdk::RGBA;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Copy, PartialEq)]
pub enum BrushType {
    Square(u8),
    Round(u8),
    Dither(u8),
    Pen(u8), // faded round
    Crayon(u8), // faded dither
}

#[derive(PartialEq)]
struct BrushProperties {
    brush_type: BrushType,
    color: RGBA,
}

pub struct Brush {
    props: BrushProperties,
    pub image: Image,
}

const TRANS: Pixel = Pixel::from_rgba(0, 0, 0, 0);

fn mk_square_brush_image(n: u8, color: RGBA) -> Image {
    let p = Pixel::from_rgba_struct(color);
    Image::from_pixels(vec![vec![p; n as usize]; n as usize])
}

fn mk_round_brush_image(n: u8, fade: bool, dither:bool, color: RGBA) -> Image {
    let p = Pixel::from_rgba_struct(color);
    let n = n as usize;
    let mut pix = vec![vec![TRANS; n]; n];

    const CIRC_THRESH: f64 = 0.3;

    for i in 0..n {
        for j in 0..n {
            let x = (n as f64 / 2.0) - (j as f64);
            let y = (n as f64 / 2.0) - (i as f64);
            let dist = (x * x + y * y).sqrt();
            let opacity = 1.0 - (dist / (n as f64 / 2.0));

            if opacity > CIRC_THRESH  && (!dither || i % 2 == j % 2) {
                if fade {
                    pix[i][j] = p.scale_alpha(opacity);
                } else {
                    pix[i][j] = p.clone();
                }
            }
        }
    }

    Image::from_pixels(pix)
}

impl Brush {
    fn image_from_props(props: &BrushProperties) -> Image {
        match props.brush_type {
            BrushType::Square(n) => mk_square_brush_image(n, props.color),
            BrushType::Round(n) => mk_round_brush_image(n, false, false, props.color),
            BrushType::Dither(n) => mk_round_brush_image(n, false, true, props.color),
            BrushType::Pen(n) => mk_round_brush_image(n, true, false, props.color),
            BrushType::Crayon(n) => mk_round_brush_image(n, true, true, props.color),
        }
    }

    pub fn modify(&mut self, color: RGBA, brush_type: BrushType) {
        let new_props = BrushProperties {
            color,
            brush_type,
        };

        if self.props == new_props {
            // no changes necessary
        } else {
            *self = Brush::from_props(new_props)
        }
    }

    pub fn new(color: RGBA, brush_type: BrushType) -> Self {
        let props = BrushProperties {
            color,
            brush_type,
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
}

pub struct BrushToolbar {
    brush: Brush,
    brush_type: BrushType,
}

impl BrushToolbar {
    pub fn new(color: RGBA, brush_type: BrushType) -> Self {
        let props = BrushProperties {
            color,
            brush_type,
        };

        BrushToolbar {
            brush: Brush::from_props(props),
            brush_type,
        }
    }

    pub fn brush_type(&self) -> &BrushType {
        &self.brush_type
    }

    pub fn get_brush(&mut self, color: RGBA) -> &Brush {
        self.brush.modify(color, self.brush_type);
        &self.brush
    }
}
