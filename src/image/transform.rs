use super::undo::action::{UndoableAction, StaticUndoableAction, ActionName};
use super::Image;

#[derive(Clone)]
pub enum Flip {
    Horizontal,
    Vertical,
    Transpose,
}

impl StaticUndoableAction for Flip {
    fn dyn_clone(&self) -> Box<dyn UndoableAction> {
        Box::new(self.clone())
    }
}

impl UndoableAction for Flip {
    fn name(&self) -> ActionName {
        ActionName::Flip
    }

    fn exec(&self, image: &mut Image) {
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

    fn undo(&self, image: &mut Image) {
        // flips are their own inverse
        self.exec(image)
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

impl UndoableAction for Rotate {
    fn name(&self) -> ActionName {
        ActionName::Rotate
    }

    fn exec(&self, image: &mut Image) {
        match self {
            Self::OneEighty => {
                // dimensions remain the same, flat pixel vector is reversed
                image.pixels.reverse()
            }
            Self::Clockwise => {
                Flip::Transpose.exec(image);
                Flip::Horizontal.exec(image);
            },
            Self::CounterClockwise => {
                Flip::Transpose.exec(image);
                Flip::Vertical.exec(image);
            },
        }
    }

    fn undo(&self, image: &mut Image) {
        self.invert().exec(image)
    }
}

impl StaticUndoableAction for Rotate {
    fn dyn_clone(&self) -> Box<dyn UndoableAction> {
        Box::new(self.clone())
    }
}
