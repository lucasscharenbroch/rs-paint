pub mod action;
mod tree;

use self::action::UndoableAction;
use super::{ImageLayer, LayerIndex, LayeredImage, Pixel};
use tree::UndoTree;
use action::ActionName;

use std::collections::HashMap;
use std::rc::Rc;
use gtk::{prelude::*, Widget};

enum ImageDiff {
    Diff(Vec<(usize, Pixel, Pixel)>, LayerIndex), // [(pos, old_pix, new_pix)], layer#
    // FullCopy(Image, Image), // (before, after)
    ManualUndo(Box<dyn UndoableAction>, LayerIndex),
    AppendLayer(gtk::gdk::RGBA, LayerIndex),
    RemoveLayer(ImageLayer, LayerIndex),
    SwapLayers(LayerIndex, LayerIndex),
    MergeLayers(ImageLayer, LayerIndex, ImageLayer, LayerIndex), /// (save_top_image, top_index, save_bottom_image, bottom_index)
    Null,
}

impl ImageDiff {
    pub fn new(
        to: &LayeredImage,
        (mod_pix, layer): (HashMap<usize, (Pixel, Pixel)>, LayerIndex)
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
                    image.update_drawable_and_layer_at(*i, *layer);
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
            ImageDiff::AppendLayer(color, idx) => {
                image.append_new_layer(*color, *idx);
            },
            ImageDiff::RemoveLayer(_deleted_image, idx) => {
                image.remove_layer(*idx);
            },
            ImageDiff::SwapLayers(i1, i2) => {
                image.swap_layers(*i1, *i2);
            }
            ImageDiff::MergeLayers(_save_top, top_idx, _save_bot, bot_idx) => {
                image.merge_layers(*top_idx, *bot_idx);
            },
            ImageDiff::Null => (),
        }
    }

    pub fn unapply_to(&mut self, image: &mut LayeredImage) {
        match self {
            ImageDiff::Diff(ref pixs, layer) => {
                for (i, before, _after) in pixs.iter() {
                    image.image_at_layer_mut(*layer).pixels[*i] = before.clone();
                    image.update_drawable_and_layer_at(*i, *layer);
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
            ImageDiff::AppendLayer(_color, idx) => {
                image.remove_layer(*idx);
            },
            ImageDiff::RemoveLayer(removed_layer_image, idx) => {
                image.append_layer_with_image(removed_layer_image.clone(), *idx);
            },
            ImageDiff::SwapLayers(i1, i2) => {
                image.swap_layers(*i1, *i2);
            }
            ImageDiff::MergeLayers(save_top, top_index, save_bot, bot_index) => {
                image.append_layer_with_image(save_top.clone(), *top_index);
                *image.image_at_layer_mut(*bot_index) = save_bot.image.clone();
                image.fused_image_at_layer_mut(*bot_index).info = save_bot.info.clone();
                image.re_compute_drawables();
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
    id_counter: usize,
}

impl ImageHistory {
    pub fn new(initial_image: LayeredImage) -> ImageHistory {
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

    fn apply_and_push_diff(&mut self, mut diff: ImageDiff, culprit: ActionName) {
        diff.apply_to(self.now_mut());

        let image_state_diff = ImageStateDiff::new(
            diff,
            self.now.id,
            self.id_counter,
            culprit,
        );

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

    pub fn append_layer(&mut self, fill_color: gtk::gdk::RGBA, idx: LayerIndex) {
        let image_diff = ImageDiff::AppendLayer(fill_color, idx);
        self.apply_and_push_diff(image_diff, ActionName::AppendLayer);
    }

    fn commit_any_changes_on_active_layer(&mut self) {
        let (mod_pix, _layer) = self.now_mut().get_and_reset_modified();
        if !mod_pix.is_empty() {
            // if self is modified in any way, push the sate with Anon
            self.push_current_state(ActionName::Anonymous);
        }
    }

    pub fn focus_layer(&mut self, idx: LayerIndex) {
        self.commit_any_changes_on_active_layer();
        self.now_mut().active_layer_index = idx;
    }

    pub fn remove_layer(&mut self, idx: LayerIndex) {
        if let LayerIndex::BaseLayer = self.now().active_layer_index() {
            self.commit_any_changes_on_active_layer();
        }

        let image_diff = ImageDiff::RemoveLayer(
            self.now().fused_image_at_layer(idx).unfuse(),
            idx,
        );

        self.apply_and_push_diff(image_diff, ActionName::RemoveLayer)
    }

    pub fn swap_layers(&mut self, i1: LayerIndex, i2: LayerIndex) {
        if [i1, i2].contains(self.now().active_layer_index()) {
            self.commit_any_changes_on_active_layer();
        }

        let image_diff = ImageDiff::SwapLayers(i1, i2);

        self.apply_and_push_diff(image_diff, ActionName::RearrangeLayers);
    }

    pub fn merge_layers(&mut self, top_index: LayerIndex, bottom_index: LayerIndex) {
        if [top_index, bottom_index].contains(self.now().active_layer_index()) {
            self.commit_any_changes_on_active_layer();
        }

        let image_diff = ImageDiff::MergeLayers(
            self.now().fused_image_at_layer(top_index).unfuse(),
            top_index,
            self.now().fused_image_at_layer(bottom_index).unfuse(),
            bottom_index,
        );

        self.apply_and_push_diff(image_diff, ActionName::MergeLayers);
    }
}
