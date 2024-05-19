use super::{Image, Pixel};

use gtk::gdk::RGBA;

#[derive(PartialEq)]
struct BrushProperties {
    color: RGBA,
}

pub struct Brush {
    props: BrushProperties,
    pub image: Image,
}

const TRANS: Pixel = Pixel::from_rgba(0, 0, 0, 0);
const BLACK: Pixel = Pixel::from_rgb(0, 0, 0);

impl Brush {
    fn image_from_props(props: &BrushProperties) -> Image {
        Image::from_pixels(vec![
                vec![TRANS, BLACK, BLACK, BLACK, TRANS],
                vec![BLACK, BLACK, BLACK, BLACK, BLACK],
                vec![BLACK, BLACK, BLACK, BLACK, BLACK],
                vec![BLACK, BLACK, BLACK, BLACK, BLACK],
                vec![TRANS, BLACK, BLACK, BLACK, TRANS],
            ]) // TODO
    }

    pub fn modify(mut self, color: RGBA) -> Self {
        let new_props = BrushProperties {
            color
        };

        if self.props == new_props {
            self
        } else {
            Brush::from_props(new_props)
        }
    }

    pub fn new(color: RGBA) -> Self {
        let props = BrushProperties {
            color
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