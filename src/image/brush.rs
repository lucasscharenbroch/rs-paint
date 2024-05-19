use super::{Image, Pixel};

use gtk::gdk::RGBA;

#[derive(PartialEq)]
pub enum BrushType {
    Square(u8),
    Round(u8),
    Diamond(u8)
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

/*
    Image::from_pixels(vec![
            vec![TRANS, BLACK, BLACK, BLACK, TRANS],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![TRANS, BLACK, BLACK, BLACK, TRANS],
        ]) // TODO
*/

fn mk_square_brush_image(n: u8, color: RGBA) -> Image {
    let p = Pixel::from_rgba_struct(color);
    Image::from_pixels(vec![vec![p; n as usize]; n as usize])
}

fn mk_round_brush_image(n: u8, color: RGBA) -> Image {
    todo!()
}

fn mk_diamond_brush_image(n: u8, color: RGBA) -> Image {
    todo!()
}

impl Brush {
    fn image_from_props(props: &BrushProperties) -> Image {
        match props.brush_type {
            BrushType::Square(n) => mk_square_brush_image(n, props.color),
            BrushType::Round(n) => mk_round_brush_image(n, props.color),
            BrushType::Diamond(n) => mk_round_brush_image(n, props.color),
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