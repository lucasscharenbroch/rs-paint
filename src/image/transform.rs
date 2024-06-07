use super::undo::action::{UndoableAction, StaticUndoableAction, ActionName};
use super::{Image, ImageLike, Pixel};

use itertools::{Itertools, Either};

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

    fn exec(&mut self, image: &mut Image) {
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

    fn undo(&mut self, image: &mut Image) {
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

    fn exec(&mut self, image: &mut Image) {
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

    fn undo(&mut self, image: &mut Image) {
        self.invert().exec(image)
    }
}

impl StaticUndoableAction for Rotate {
    fn dyn_clone(&self) -> Box<dyn UndoableAction> {
        Box::new(self.clone())
    }
}

struct CropUndoInfo {
    old_w: usize,
    old_h: usize,
    scrapped_pixels: Vec<Pixel>,
}

pub struct Crop {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    undo_info: Option<CropUndoInfo>,
}

impl Crop {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Crop {
            x, y, w, h,
            undo_info: None,
        }
    }

    // Whether an index of the flat pixel array should be removed in the crop
    #[inline]
    fn should_keep_pix_at_idx(&self, old_w: usize, idx: usize) -> bool {
        let (i, j) = (idx / old_w, idx % old_w);
        j >= self.x && j < self.x + self.w && i >= self.y && i < self.y + self.h
    }
}

impl UndoableAction for Crop {
    fn name(&self) -> ActionName {
        ActionName::Crop
    }

    fn exec(&mut self, image: &mut Image) {
        let kept_pixels = if let None = self.undo_info {
            // only record undo_info on the first execution

            let old_h = image.height;
            let old_w = image.width;
            let (scrapped_pixels, kept_pixels): (Vec<_>, Vec<_>) = image.pixels.iter()
                .enumerate()
                .partition_map(|(idx, pix)| {
                    if self.should_keep_pix_at_idx(old_w, idx) {
                        Either::Right(pix.clone())
                    } else {
                        Either::Left(pix.clone())
                    }
                });

            self.undo_info = Some(CropUndoInfo {
                old_h,
                old_w,
                scrapped_pixels,
            });

            kept_pixels
        } else {
            let old_w = self.undo_info.as_ref().unwrap().old_w;

            image.pixels.iter()
                .enumerate()
                .filter_map(|(idx, pix)| {
                    if self.should_keep_pix_at_idx(old_w, idx) {
                        Some(pix.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        image.width = self.w;
        image.height = self.h;
        image.pixels = kept_pixels;
    }

    fn undo(&mut self, image: &mut Image) {
        let undo_info = self.undo_info.as_ref().unwrap();
        let old_sz = undo_info.old_h * undo_info.old_w;

        let mut old_pix = Vec::with_capacity(old_sz);

        let mut scrapped_idx = 0;
        let mut kept_idx = 0;

        for idx in 0..old_sz {
            if self.should_keep_pix_at_idx(undo_info.old_w, idx) {
                old_pix.push(image.pixels[kept_idx].clone());
                kept_idx += 1;
            } else {
                old_pix.push(undo_info.scrapped_pixels[scrapped_idx].clone());
                scrapped_idx += 1;
            };
        }

        image.width = undo_info.old_w;
        image.height = undo_info.old_h;
        image.pixels = old_pix;
    }
}
