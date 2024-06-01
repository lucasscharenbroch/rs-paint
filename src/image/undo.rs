pub mod action;
mod tree;

use crate::image::DrawableImage;
use self::action::UndoableAction;
use super::{Image, UnifiedImage, Pixel};
use action::{ActionName};
use tree::UndoTree;

use std::collections::HashMap;

enum ImageDiff {
    Diff(Vec<(usize, Pixel, Pixel)>), // [(pos, old_pix, new_pix)]
    FullCopy(Image, Image), // (before, after)
    ManualUndo(Box<dyn UndoableAction>),
    Null,
}

impl ImageDiff {
    pub fn new(
        to: &UnifiedImage,
        (mod_pix, save_image): (HashMap<usize, (Pixel, Pixel)>, Option<Image>)
    ) -> ImageDiff {
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
                    image.drawable.pixels[*i] = after.to_drawable();
                }
            },
            ImageDiff::FullCopy(ref _before, ref after) => {
                image.image.pixels = after.pixels.clone();
                (image.image.width, image.image.height) = (after.width, after.height);
                image.drawable = DrawableImage::from_image(&image.image);
            },
            ImageDiff::ManualUndo(ref action) => {
                image.apply_action(action);
            },
            ImageDiff::Null => (),
        }
    }

    pub fn unapply_to(&self, image: &mut UnifiedImage) {
        match self {
            &ImageDiff::Diff(ref pixs) => {
                for (i, before, _after) in pixs.iter() {
                    image.image.pixels[*i] = before.clone();
                    image.drawable.pixels[*i] = before.to_drawable();
                }
            },
            &ImageDiff::FullCopy(ref before, ref _after) => {
                image.image.pixels = before.pixels.clone();
                (image.image.width, image.image.height) = (before.width, before.height);
                image.drawable = DrawableImage::from_image(&image.image);
            },
            ImageDiff::ManualUndo(ref action) => {
                image.unapply_action(action);
            },
            ImageDiff::Null => (),
        }
    }
}

pub struct ImageState {
    img: UnifiedImage,
    id: usize,
}

pub struct ImageStateDiff {
    image_diff: ImageDiff,
    // ids are used as a quick "hash" of a commit
    // to determine if an image has been changed
    old_id: usize,
    new_id: usize,
    culprit: ActionName,
}

impl ImageStateDiff {
    fn new(image_diff: ImageDiff, old_id: usize, new_id: usize, culprit: ActionName) -> Self {
        ImageStateDiff {
            image_diff,
            old_id,
            new_id,
            culprit,
        }
    }

    fn apply_to(&self, image_state: &mut ImageState) {
        self.image_diff.apply_to(&mut image_state.img);
        image_state.id = self.new_id;
    }

    fn unapply_to(&self, image_state: &mut ImageState) {
        self.image_diff.unapply_to(&mut image_state.img);
        image_state.id = self.old_id;
    }
}

pub struct ImageHistory {
    now: ImageState,
    undo_tree: UndoTree,
    id_counter: usize,
}

impl ImageHistory {
    pub fn new(initial_image: UnifiedImage) -> ImageHistory {
        let initial_state = ImageState {
            img: initial_image,
            id: 0,
        };

        ImageHistory {
            now: initial_state,
            undo_tree: UndoTree::new(),
            id_counter: 1,
        }
    }

    pub fn now(&self) -> &UnifiedImage {
        &self.now.img
    }

    pub fn now_id(&self) -> usize {
        self.now.id
    }

    pub fn now_mut(&mut self) -> &mut UnifiedImage {
        &mut self.now.img
    }

    fn push_state_diff(&mut self, state_diff: ImageStateDiff) {
        self.now.id = self.id_counter;
        self.id_counter += 1;

        self.undo_tree.commit(state_diff);
    }

    pub fn push_current_state(&mut self, culprit: ActionName) {
        let mod_pix_info = self.now.img.get_and_reset_modified();
        let image_diff = ImageDiff::new(&self.now.img, mod_pix_info);
        let image_state_diff = ImageStateDiff::new(image_diff, self.now.id, self.id_counter, culprit);

        self.push_state_diff(image_state_diff);
    }

    pub fn undo(&mut self) {
        if let Some(d) = self.undo_tree.undo() {
            d.unapply_to(&mut self.now);
        }
    }

    pub fn redo(&mut self) {
        if let Some(d) = self.undo_tree.redo() {
            d.apply_to(&mut self.now);
        }
    }
}
