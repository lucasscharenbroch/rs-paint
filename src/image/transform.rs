use super::undo::action::{MultiLayerAction, ActionName};
use super::Image;

#[derive(Clone)]
pub enum Flip {
    Horizontal,
    Vertical,
    Transpose,
}

impl MultiLayerAction for Flip {
    type LayerData = ();

    fn new_layer_data(&self, _image: &mut Image) -> Self::LayerData {
        ()
    }

    fn name(&self) -> ActionName {
        ActionName::Flip
    }

    fn exec(&mut self, _layer_data: &mut (), image: &mut Image) {
        let height = image.height;
        let width = image.width;

        match self {
            Self::Vertical => {
                for i in 0..(height / 2) {
                    for j in 0..width {
                        image.swap_pixels((i, j), (height - i - 1, j));
                    }
                }
            },
            Self::Horizontal => {
                for i in 0..height {
                    for j in 0..(width / 2) {
                        image.swap_pixels((i, j), (i, width - j - 1));
                    }
                }
            },
            Self::Transpose => {
                // there's probably some way to do this in place, but
                // the locality and speed is questionable
                let mut new_pixels = Vec::with_capacity(image.width * image.height);
                unsafe {
                    new_pixels.set_len(image.width * image.height);
                }

                for i in 0..height {
                    for j in 0..width {
                        new_pixels[j * height + i] = image.pixels[i * width + j].clone();
                    }
                }

                std::mem::swap(&mut image.height, &mut image.width);
                image.pixels = new_pixels;
            }
        }
    }

    fn undo(&mut self, _layer_data: &mut (), image: &mut Image) {
        // flips are their own inverse
        self.exec(&mut(), image);
    }
}

#[derive(Clone)]
pub enum Rotate {
    OneEighty,
    // 90 deg...
    Clockwise,
    CounterClockwise,
}

impl Rotate {
    fn invert(&self) -> Self {
        match self {
            Self::OneEighty => Self::OneEighty,
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

impl MultiLayerAction for Rotate {
    type LayerData = ();

    fn new_layer_data(&self, _image: &mut Image) -> Self::LayerData {
        ()
    }

    fn name(&self) -> ActionName {
        ActionName::Rotate
    }

    fn exec(&mut self, _layer_data: &mut (), image: &mut Image) {
        match self {
            Self::OneEighty => {
                // dimensions remain the same, flat pixel vector is reversed
                image.pixels.reverse()
            }
            Self::Clockwise => {
                Flip::Transpose.exec(&mut (), image);
                Flip::Horizontal.exec(&mut (), image);
            },
            Self::CounterClockwise => {
                Flip::Transpose.exec(&mut(), image);
                Flip::Vertical.exec(&mut(), image);
            },
        }
    }

    fn undo(&mut self, _layer_data: &mut (), image: &mut Image) {
        self.invert().exec(&mut(), image)
    }
}
