use super::{Image, Pixel};

use gtk::gdk::RGBA;

pub type BrushProperties = RGBA;

pub struct Brush {
    pub props: BrushProperties,
    pub image: Image,
}

const TRANS: Pixel = Pixel::from_rgba(0, 0, 0, 0);
const BLACK: Pixel = Pixel::from_rgb(0, 0, 0);

pub fn default_brush() -> Brush {
    let props = RGBA::new(0.0, 0.0, 0.0, 1.0);
    let image = Image::from_pixels(vec![
            vec![TRANS, BLACK, BLACK, BLACK, TRANS],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![TRANS, BLACK, BLACK, BLACK, TRANS],
        ]);

    Brush {
        props,
        image
    }
}