use crate::image::Pixel;

use super::{Image};
use gtk::gdk::{RGBA};

pub struct NewImageProps {
    pub width: usize,
    pub height: usize,
    pub color: RGBA,
}

pub fn generate(props: NewImageProps) -> Image {
    let p = Pixel::from_rgba_struct(props.color);
    let pixels = vec![vec![p; props.width]; props.height];
    Image::from_pixels(pixels)
}
