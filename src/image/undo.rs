use super::{Image, UnifiedImage, Pixel};

use std::{collections::HashMap};

enum ImageDiff {
    Diff(Vec<(usize, Pixel, Pixel)>), // [(pos, old_pix, new_pix)]
    FullCopy(Image, Image), // (before, after)
}

impl ImageDiff {
    pub fn new(
        to: &UnifiedImage,
        (mod_pix, save_image): (HashMap<usize, (Pixel, Pixel)>, Option<Image>)
    ) -> ImageDiff {
        let new_dimensions = (to.width() as usize, to.height() as usize);

        // it's probably faster to not bother with the hash set if enough pixels have been modified
        const EXHAUSTIVE_CHECK_THRESHOLD: f64 = 0.25;
        let hash_set_too_big_to_bother = (mod_pix.len() as f64 / (to.width() * to.height()) as f64) > EXHAUSTIVE_CHECK_THRESHOLD;

        if let Some(save_image) = save_image {
            ImageDiff::FullCopy(save_image, to.image().clone())
        } else { // just consider pixel coordinates in the hash map
            let diff_vec = mod_pix.into_iter()
                .map(|(i, (b, a))| (i, b, a))
                .collect::<Vec<_>>();

            ImageDiff::Diff(diff_vec)
        }
    }

    pub fn apply_to(&self, image: &mut UnifiedImage) {
        match self {
            ImageDiff::Diff(ref pixs) => {
                for (i, _before, after) in pixs.iter() {
                    image.image.pixels[*i] = after.clone();
                }
            },
            ImageDiff::FullCopy(ref _before, ref after) => {
                image.image.pixels = after.pixels.clone();
                (image.image.width, image.image.height) = (after.width, after.height);
            },
        }
    }

    pub fn unapply_to(&self, image: &mut UnifiedImage) {
        match self {
            &ImageDiff::Diff(ref pixs) => {
                for (i, before, _after) in pixs.iter() {
                    image.image.pixels[*i] = before.clone();
                }
            },
            &ImageDiff::FullCopy(ref before, ref _after) => {
                image.image.pixels = before.pixels.clone();
                (image.image.width, image.image.height) = (before.width, before.height);
            },
        }
    }
}

pub struct ImageHistory {
    now: UnifiedImage,
    undo_stack: Vec<ImageDiff>,
    redo_stack: Vec<ImageDiff>,
}

impl ImageHistory {
    pub fn new(initial_image: UnifiedImage) -> ImageHistory {
        ImageHistory {
            now: initial_image,
            undo_stack: vec![],
            redo_stack: vec![],
        }
    }

    pub fn now(&self) -> &UnifiedImage {
        &self.now
    }

    pub fn now_mut(&mut self) -> &mut UnifiedImage {
        &mut self.now
    }

    pub fn push_state(&mut self) {
        let mod_pix_info = self.now.get_and_reset_modified();
        self.undo_stack.push(ImageDiff::new(&self.now, mod_pix_info));
        self.redo_stack = vec![];
    }

    pub fn undo(&mut self) {
        if let Some(d) = self.undo_stack.pop() {
            d.unapply_to(&mut self.now);
            self.redo_stack.push(d);
        }
    }

    // TODO mult-level-undo (choose which branch to take)
    pub fn redo(&mut self) {
        if let Some(d) = self.redo_stack.pop() {
            d.apply_to(&mut self.now);
            self.undo_stack.push(d);
        }
    }
}
