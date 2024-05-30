use super::{Image};
use gtk::gdk::{RGBA};

pub struct NewImageProps {
    width: usize,
    height: usize,
    color: RGBA,
}

pub fn generate(props: NewImageProps) -> Image {
    todo!()
}
