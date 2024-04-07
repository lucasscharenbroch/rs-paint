use super::image::{Image, UnifiedImage, Pixel};

use std::{collections::HashMap};

enum PixelVecDiff {
    Diff(Vec<(usize, Pixel, Pixel)>), // [(pos, old_pix, new_pix)]
    FullCopy(Vec<Pixel>, Vec<Pixel>), // (old_pix_vec, new_pix_vec)
}

struct ImageDiff {
    old_dimensions: (usize, usize),
    new_dimensions: (usize, usize),
    changed_pix: PixelVecDiff,
}

impl ImageDiff {
    pub fn new(
        from: &UnifiedImage,
        to: &UnifiedImage,
        (mod_pix, save_image): (HashMap<usize, (Pixel, Pixel)>, Option<Image>)
    ) -> ImageDiff {
        let old_dimensions = (from.width() as usize, from.height() as usize);
        let new_dimensions = (to.width() as usize, to.height() as usize);

        // it's probably faster to not bother with the hash set if enough pixels have been modified
        const EXHAUSTIVE_CHECK_THRESHOLD: f64 = 0.25;
        let hash_set_too_big_to_bother = (mod_pix.len() as f64 / (from.width() * from.height()) as f64) > EXHAUSTIVE_CHECK_THRESHOLD;

        let changed_pix =
            if let Some(save_image) = save_image {
                PixelVecDiff::FullCopy(save_image.into_pixels(), to.image().pixels().clone())
            } else if hash_set_too_big_to_bother || old_dimensions != new_dimensions {
                PixelVecDiff::FullCopy(from.image().pixels().clone(), to.image().pixels().clone())
            } else { // just consider pixel coordinates in the hash map
                let width = from.width();
                let from_pix = from.image().pixels();
                let to_pix = to.image().pixels();

                let diff_vec = mod_pix.into_iter()
                    .map(|(i, (b, a))| (i, b, a))
                    .collect::<Vec<_>>();

                PixelVecDiff::Diff(diff_vec)
            };

        ImageDiff {
            old_dimensions,
            new_dimensions,
            changed_pix,
        }
    }

    pub fn apply_to(&self, image: &mut UnifiedImage) {
        match self.changed_pix {
            PixelVecDiff::Diff(ref pixs) => {
                assert!(self.old_dimensions == self.new_dimensions);
            },
            PixelVecDiff::FullCopy(ref before, ref after) => {

            },
        }
    }

    pub fn unapply_to(&self, image: &mut UnifiedImage) {
        // image.set_image(&self.old, true);
    }
}

pub struct ImageHistory {
    now: UnifiedImage,
    last_save: UnifiedImage,
    undo_stack: Vec<ImageDiff>,
    redo_stack: Vec<ImageDiff>,
}

impl ImageHistory {
    pub fn new(initial_image: UnifiedImage) -> ImageHistory {
        ImageHistory {
            now: initial_image.clone(),
            last_save: initial_image,
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
        self.undo_stack.push(ImageDiff::new(&self.last_save, &self.now, mod_pix_info));
        self.redo_stack = vec![];
        self.last_save = self.now.clone();
    }

    pub fn undo(&mut self) {
        if let Some(d) = self.undo_stack.pop() {
            d.unapply_to(&mut self.now);
            self.last_save = self.now.clone();
            self.redo_stack.push(d);
        }
    }

    // TODO mult-level-undo (choose which branch to take)
    pub fn redo(&mut self) {
        if let Some(d) = self.redo_stack.pop() {
            d.apply_to(&mut self.now);
            self.last_save = self.now.clone();
            self.undo_stack.push(d);
        }
    }
}
