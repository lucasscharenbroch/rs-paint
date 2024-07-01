pub mod action;
mod tree;

use crate::ui::layers::LayersUi;
use crate::{image::DrawableImage, ui::UiState};
use self::action::UndoableAction;
use super::{FusedImage, Image, LayerSpecifier, LayeredImage, Pixel};
use tree::UndoTree;
use action::{ActionName};

use std::{cell::RefCell, collections::HashMap};
use std::rc::Rc;
use gtk::{prelude::*, Widget};

enum ImageDiff {
    Diff(Vec<(usize, Pixel, Pixel)>, LayerSpecifier), // [(pos, old_pix, new_pix)], layer#
    // FullCopy(Image, Image), // (before, after)
    ManualUndo(Box<dyn UndoableAction>, LayerSpecifier),
    Null,
}

impl ImageDiff {
    pub fn new(
        to: &LayeredImage,
        (mod_pix, layer): (HashMap<usize, (Pixel, Pixel)>, LayerSpecifier)
    ) -> ImageDiff {
        /* TODO
        if let Some(save_image) = save_image {
            ImageDiff::FullCopy(save_image, to.image().clone())
        } else { // just consider pixel coordinates in the hash map
        */
            let diff_vec = mod_pix.into_iter()
                .map(|(i, (b, a))| (i, b, a))
                .collect::<Vec<_>>();

            ImageDiff::Diff(diff_vec, layer)
        // }
    }

    pub fn apply_to(&mut self, image: &mut LayeredImage) {
        match self {
            ImageDiff::Diff(ref pixs, layer) => {
                for (i, _before, after) in pixs.iter() {
                    image.image_at_layer_mut(*layer).pixels[*i] = after.clone();
                    image.update_drawable_at(*i);
                }
            },
            /* TODO
            ImageDiff::FullCopy(ref _before, ref after) => {
                image.image.pixels = after.pixels.clone();
                (image.image.width, image.image.height) = (after.width, after.height);
                image.drawable = DrawableImage::from_image(&image.image);
            },
            */
            ImageDiff::ManualUndo(action, layer) => {
                image.apply_action(action, *layer);
            },
            ImageDiff::Null => (),
        }
    }

    pub fn unapply_to(&mut self, image: &mut LayeredImage) {
        match self {
            ImageDiff::Diff(ref pixs, layer) => {
                for (i, before, _after) in pixs.iter() {
                    image.image_at_layer_mut(*layer).pixels[*i] = before.clone();
                    image.update_drawable_at(*i);
                }
            },
            /* TODO
            ImageDiff::FullCopy(ref before, ref _after) => {
                image.image.pixels = before.pixels.clone();
                (image.image.width, image.image.height) = (before.width, before.height);
                image.drawable = DrawableImage::from_image(&image.image);
            },
            */
            ImageDiff::ManualUndo(action, layer) => {
                image.unapply_action(action, *layer);
            },
            ImageDiff::Null => (),
        }
    }
}

pub struct ImageState {
    img: LayeredImage,
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

    fn apply_to(&mut self, image_state: &mut ImageState) {
        self.image_diff.apply_to(&mut image_state.img);
        image_state.id = self.new_id;
    }

    fn unapply_to(&mut self, image_state: &mut ImageState) {
        self.image_diff.unapply_to(&mut image_state.img);
        image_state.id = self.old_id;
    }
}

pub struct ImageHistory {
    now: ImageState,
    undo_tree: UndoTree,
    layers_ui: LayersUi,
    id_counter: usize,
}

impl ImageHistory {
    pub fn new(initial_image: LayeredImage) -> ImageHistory {
        let layers_ui = LayersUi::from_layered_image(&initial_image);

        let initial_state = ImageState {
            img: initial_image,
            id: 0,
        };

        ImageHistory {
            now: initial_state,
            undo_tree: UndoTree::new(),
            layers_ui,
            id_counter: 1,
        }
    }

    pub fn now(&self) -> &LayeredImage {
        &self.now.img
    }

    pub fn now_id(&self) -> usize {
        self.now.id
    }

    pub fn now_mut(&mut self) -> &mut LayeredImage {
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
            d.borrow_mut().unapply_to(&mut self.now);
        }
    }

    pub fn redo(&mut self) {
        if let Some(d) = self.undo_tree.redo() {
            d.borrow_mut().apply_to(&mut self.now);
        }
    }

    pub fn widget_scrolled_to_active_commit(&self) -> &impl IsA<Widget> {
        // this "waits for redraw", so it's fine to call it here
        self.undo_tree.scroll_to_active_node_after_resize();

        self.undo_tree.widget()
    }

    pub fn layers_widget(&self) -> &impl IsA<Widget> {
        self.layers_ui.widget()
    }

    fn id_exists(&self, id: usize) -> bool {
        self.id_counter > id
    }

    // locate the given commit-id in the tree, then
    // apply the diffs along the path to that commit
    fn migrate_to_commit(&mut self, target_id: usize) {
        assert!(self.id_exists(target_id), "can't migrate to a non-existant commit");
        let diffs = self.undo_tree.traverse_to(target_id);

        for diff in diffs {
            diff(&mut self.now)
        }
    }

    pub fn set_hooks(
        &mut self,
        mod_self: Rc<dyn Fn(Box<dyn Fn(&mut Self)>)>,
        update_canvas: Rc<dyn Fn()>,
    ) {
        self.undo_tree.set_hooks(mod_self, update_canvas);
    }
}
