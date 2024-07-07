pub mod action;
mod tree;

use self::action::SingleLayerAction;
use super::{FusedLayeredImage, Image, Layer, LayerIndex, Pixel};
use tree::UndoTree;
use action::{ActionName, MultiLayerActionWrapper};

use std::collections::HashMap;
use std::rc::Rc;
use gtk::{prelude::*, Widget};
use std::collections::HashSet;

/// A unifiable value for which drawables/pixels of
/// a `FusedLayeredImage` should be re-computed. This
/// can be used to lazily accumulate changes before updating.
/// The inclusion of any pixel implicitly includes that
/// pixel on the "main-drawable" (`FusedLayerImage::drawable`).
struct DrawablesToUpdate {
    full_layers: HashSet<LayerIndex>,
    /// pixels_in_layers[i] <=> set of flat indices into LayerIndex::from_usize(i)
    pixels_in_layers: Vec<HashSet<usize>>,
    /// If this is `true`, the main drawable will always be updated.
    /// If not, it will be updated if any full layer is included;
    /// if none are, it'll be updated at any pixels in `pixels_in_layers`.
    definitely_main: bool,
}

impl DrawablesToUpdate {
    fn union(mut self, mut other: DrawablesToUpdate) -> Self {
        // ensure self.pixels_in_layers is the same length
        // as other.pixels_in_layers

        while self.pixels_in_layers.len() < other.pixels_in_layers.len() {
            self.pixels_in_layers.push(HashSet::new());
        }

        while self.pixels_in_layers.len() > other.pixels_in_layers.len() {
            other.pixels_in_layers.push(HashSet::new());
        }

        let self_pix = self.pixels_in_layers.into_iter();
        let other_pix = other.pixels_in_layers.into_iter();

        let pixels_in_layers = self_pix.zip(other_pix)
            .map(|(a, b)| a.union(&b).map(|x| *x).collect())
            .collect();

        self.full_layers.extend(&other.full_layers);

        DrawablesToUpdate {
            full_layers: self.full_layers,
            pixels_in_layers,
            definitely_main: self.definitely_main || other.definitely_main,
        }
    }

    fn just_this_layer(layer_idx: LayerIndex) -> Self {
        DrawablesToUpdate {
            full_layers: std::iter::once(layer_idx).collect(),
            pixels_in_layers: Vec::new(),
            definitely_main: false,
        }
    }

    fn just_these_pixels(pixels: HashSet<usize>, layer_idx: LayerIndex) -> Self {
        let mut pixels_in_layers = vec![HashSet::new(); layer_idx.to_usize()];
        pixels_in_layers.push(pixels);

        DrawablesToUpdate {
            full_layers: HashSet::new(),
            pixels_in_layers,
            definitely_main: false,
        }
    }

    fn just_the_main_drawable() -> Self {
        DrawablesToUpdate {
            full_layers: HashSet::new(),
            pixels_in_layers: Vec::new(),
            definitely_main: true,
        }
    }

    fn these_layers(layer_indices: impl Iterator<Item = LayerIndex>) -> Self {
        DrawablesToUpdate {
            full_layers: layer_indices.collect(),
            pixels_in_layers: Vec::new(),
            definitely_main: false,
        }
    }

    fn none() -> Self {
        DrawablesToUpdate {
            full_layers: HashSet::new(),
            pixels_in_layers: Vec::new(),
            definitely_main: false,
        }
    }

    fn do_update(self, image: &mut FusedLayeredImage) {
        let update_main = self.definitely_main || self.full_layers.len() != 0;

        if update_main {
            image.re_compute_main_drawable();
        }

        self.full_layers.into_iter()
            .for_each(|idx| image.re_compute_drawable_at_index(idx));

        for (idx_usize, pix) in self.pixels_in_layers.iter().enumerate() {
            let idx = LayerIndex::from_usize(idx_usize);
            for i in pix.iter() {
                image.re_compute_layer_drawable_pixel(*i, idx);
            }
        }

        if !update_main {
            let mut main_pix = HashSet::new();

            for pix in self.pixels_in_layers.iter() {
                main_pix.extend(pix.iter());
            }

            for i in main_pix.iter() {
                image.re_compute_main_drawable_pixel(*i);
            }
        }
    }
}

enum ImageDiff {
    Diff(Vec<(usize, Pixel, Pixel)>, LayerIndex), // [(pos, old_pix, new_pix)], layer#
    // FullCopy(Image, Image), // (before, after)
    SingleLayerManualUndo(Box<dyn SingleLayerAction<Image>>, LayerIndex),
    AppendLayer(gtk::gdk::RGBA, LayerIndex),
    RemoveLayer(Layer, LayerIndex),
    SwapLayers(LayerIndex, LayerIndex),
    MergeLayers(Layer, LayerIndex, Layer, LayerIndex), /// (save_top_image, top_index, save_bottom_image, bottom_index)
    MultiLayerManualUndo(MultiLayerActionWrapper),
    Null,
}

impl ImageDiff {
    pub fn new(
        _to: &FusedLayeredImage,
        (mod_pix, layer): (HashMap<usize, (Pixel, Pixel)>, LayerIndex)
    ) -> ImageDiff {
        let diff_vec = mod_pix.into_iter()
            .map(|(i, (b, a))| (i, b, a))
            .collect::<Vec<_>>();

        ImageDiff::Diff(diff_vec, layer)
    }

    pub fn apply_to(&mut self, image: &mut FusedLayeredImage) -> DrawablesToUpdate {
        match self {
            ImageDiff::Diff(ref pixs, layer) => {
                let mut pix_mod = HashSet::new();
                for (i, _before, after) in pixs.iter() {
                    image.image_at_layer_mut(*layer).pixels[*i] = after.clone();
                    pix_mod.insert(*i);
                }
                DrawablesToUpdate::just_these_pixels(pix_mod, *layer)
            },
            ImageDiff::SingleLayerManualUndo(action, layer) => {
                image.apply_action(action, *layer);
                DrawablesToUpdate::just_this_layer(*layer)
            },
            ImageDiff::AppendLayer(color, idx) => {
                image.append_new_layer(*color, *idx);
                DrawablesToUpdate::just_this_layer(*idx)
            },
            ImageDiff::RemoveLayer(_deleted_image, idx) => {
                image.remove_layer(*idx);
                DrawablesToUpdate::just_the_main_drawable()
            },
            ImageDiff::SwapLayers(i1, i2) => {
                image.swap_layers(*i1, *i2);
                DrawablesToUpdate::just_the_main_drawable()
            }
            ImageDiff::MergeLayers(_save_top, top_idx, _save_bot, bot_idx) => {
                image.merge_layers(*top_idx, *bot_idx);
                DrawablesToUpdate::just_this_layer(*bot_idx)
            },
            ImageDiff::MultiLayerManualUndo(action_struct) => {
                action_struct.exec(image);
                DrawablesToUpdate::these_layers(image.layer_indices())
            }
            ImageDiff::Null => DrawablesToUpdate::none(),
        }
    }

    pub fn unapply_to(&mut self, image: &mut FusedLayeredImage) -> DrawablesToUpdate {
        match self {
            ImageDiff::Diff(ref pixs, layer) => {
                let mut pix_mod = HashSet::new();
                for (i, before, _after) in pixs.iter() {
                    image.image_at_layer_mut(*layer).pixels[*i] = before.clone();
                    pix_mod.insert(*i);
                }
                DrawablesToUpdate::just_these_pixels(pix_mod, *layer)
            },
            ImageDiff::SingleLayerManualUndo(action, layer) => {
                image.unapply_action(action, *layer);
                DrawablesToUpdate::just_this_layer(*layer)
            },
            ImageDiff::AppendLayer(_color, idx) => {
                image.remove_layer(*idx);
                DrawablesToUpdate::just_the_main_drawable()
            },
            ImageDiff::RemoveLayer(removed_layer_image, idx) => {
                image.append_layer_with_image(removed_layer_image.clone(), *idx);
                DrawablesToUpdate::just_this_layer(*idx)
            },
            ImageDiff::SwapLayers(i1, i2) => {
                image.swap_layers(*i1, *i2);
                DrawablesToUpdate::just_the_main_drawable()
            }
            ImageDiff::MergeLayers(save_top, top_index, save_bot, bot_index) => {
                image.append_layer_with_image(save_top.clone(), *top_index);
                *image.image_at_layer_mut(*bot_index) = save_bot.image.clone();
                image.fused_image_at_layer_mut(*bot_index).info = save_bot.info.clone();
                DrawablesToUpdate::these_layers(vec![*top_index, *bot_index].into_iter())
            },
            ImageDiff::MultiLayerManualUndo(action_struct) => {
                action_struct.undo(image);
                DrawablesToUpdate::these_layers(image.layer_indices())
            }
            ImageDiff::Null => DrawablesToUpdate::none(),
        }
    }
}

pub struct ImageState {
    img: FusedLayeredImage,
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

    fn apply_to(&mut self, image_state: &mut ImageState) -> DrawablesToUpdate {
        let to_update = self.image_diff.apply_to(&mut image_state.img);
        image_state.id = self.new_id;
        to_update
    }

    fn unapply_to(&mut self, image_state: &mut ImageState) -> DrawablesToUpdate {
        let to_update = self.image_diff.unapply_to(&mut image_state.img);
        image_state.id = self.old_id;
        to_update
    }
}

pub struct ImageHistory {
    now: ImageState,
    undo_tree: UndoTree,
    id_counter: usize,
}

impl ImageHistory {
    pub fn new(initial_image: FusedLayeredImage) -> ImageHistory {
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

    pub fn now(&self) -> &FusedLayeredImage {
        &self.now.img
    }

    pub fn now_id(&self) -> usize {
        self.now.id
    }

    pub fn now_mut(&mut self) -> &mut FusedLayeredImage {
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
        let to_update = diff.apply_to(self.now_mut());
        to_update.do_update(self.now_mut());

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
            let to_update = d.borrow_mut().unapply_to(&mut self.now);
            to_update.do_update(self.now_mut());
        }
    }

    pub fn redo(&mut self) {
        if let Some(d) = self.undo_tree.redo() {
            let to_update = d.borrow_mut().apply_to(&mut self.now);
            to_update.do_update(self.now_mut());
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

        let mut drawables_to_update = DrawablesToUpdate::none();

        for diff in diffs {
            let to_update = diff(&mut self.now);
            drawables_to_update = drawables_to_update.union(to_update);
        }

        drawables_to_update.do_update(self.now_mut())
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
            self.now().fused_image_at_layer(idx).unfused(),
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
            self.now().fused_image_at_layer(top_index).unfused(),
            top_index,
            self.now().fused_image_at_layer(bottom_index).unfused(),
            bottom_index,
        );

        self.apply_and_push_diff(image_diff, ActionName::MergeLayers);
    }
}
