use crate::image::{Image, ImageLikeUncheckedMut, LayerIndex, TrackedLayeredImage, LayeredImage};
use super::{ImageDiff, ImageHistory, ImageStateDiff};

use std::any::Any;

// The algorithm that causes an undo commit:
// solely used for display
pub enum ActionName {
    Anonymous, // Caused by non-commited image writes, probably
               // due to an internal error, or gtk invariant issue
    Pencil,
    Fill,
    Delete,
    Rotate,
    Flip,
    Scale,
    LevelShift,
    Crop,
    Expand,
    AppendLayer,
    RemoveLayer,
    RearrangeLayers,
    MergeLayers,
}

impl ActionName {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Anonymous => "Anonyous",
            Self::Pencil => "Pencil",
            Self::Fill => "Fill",
            Self::Delete => "Delete",
            Self::Rotate => "Rotate",
            Self::Flip => "Flip",
            Self::Scale => "Scale",
            Self::LevelShift => "Level Shift",
            Self::Crop => "Crop",
            Self::Expand => "Expand",
            Self::AppendLayer => "Append Layer",
            Self::RemoveLayer => "Remove Layer",
            Self::RearrangeLayers => "Rearrange Layers",
            Self::MergeLayers => "Merge Layers",
        }
    }
}

/// An action that uses the automatic undo/redo in `LayeredImage`
/// (exposed through the `TrackedLayeredImage` interface)
pub trait DoableAction {
    fn name(&self) -> ActionName;
    fn exec(self, image: &mut impl TrackedLayeredImage);
    // undo is imlpicit: it will be done by diffing the image
}

/// An action with a manual undo that modifies the image
/// through the `ImageLikeUncheckedMut` interface. The action
/// will only be used on a single layer (and is free to store
/// mutable data tied to that layer).
pub trait UndoableAction<I>
    where I: ImageLikeUncheckedMut
{
    fn name(&self) -> ActionName;
    fn exec(&mut self, image: &mut I);
    fn undo(&mut self, image: &mut I); // explicit undo provided
}

pub trait StaticUndoableAction<I>: UndoableAction<I>
    where I: ImageLikeUncheckedMut
{
    fn dyn_clone(&self) -> Box<dyn UndoableAction<I>>;
}

/// An action with a manual undo that is given full access
/// to the `Image` (including resizing). The action is
/// executed/undone to each layer individually.
/// It's assumed that both `undo` and `exec` are effectively
/// pure in terms of the size of the input and output image (so
/// layers sizes will not become mismatched).
pub trait MultiUndoableAction {
    /// Layer-Specific undo data provided mutably to both
    /// `exec` and `undo`
    type LayerData;
    fn new_layer_data(&self, image: &mut Image) -> Self::LayerData;

    fn name(&self) -> ActionName;
    fn exec(&mut self, layer_data: &mut Self::LayerData, image: &mut Image);
    fn undo(&mut self, layer_data: &mut Self::LayerData, image: &mut Image);
}

pub trait StaticMultiUndoableAction<D>: MultiUndoableAction + {
    fn dyn_clone(&self) -> Box<dyn MultiUndoableAction<LayerData = D>>;
}

impl<D, T> StaticMultiUndoableAction<D> for T
    where T: MultiUndoableAction<LayerData = D> + Clone + 'static
{
    fn dyn_clone(&self) -> Box<dyn MultiUndoableAction<LayerData = D>> {
        Box::new(self.clone())
    }
}

/// Wrapper trait to prevent propigation of `LayerData`
/// (using Box<dyn Any> instead)
trait MultiUndoableActionWrapperTrait {
    fn new_layer_data(&self, image: &mut Image) -> Box<dyn Any>;

    fn name(&self) -> ActionName;
    fn exec(&mut self, layer_data: &mut Box<dyn Any>, image: &mut Image);
    fn undo(&mut self, layer_data: &mut Box<dyn Any>, image: &mut Image);
}

impl<D: 'static> MultiUndoableActionWrapperTrait for Box<dyn MultiUndoableAction<LayerData = D>> {
    fn new_layer_data(&self, image: &mut Image) -> Box<dyn Any> {
        Box::new(MultiUndoableAction::new_layer_data(self.as_ref(), image))
    }

    fn name(&self) -> ActionName {
        MultiUndoableAction::name(self.as_ref())
    }

    fn exec(&mut self, layer_data: &mut Box<dyn Any>, image: &mut Image) {
        MultiUndoableAction::exec(self.as_mut(), layer_data.as_mut().downcast_mut().unwrap(), image)
    }

    fn undo(&mut self, layer_data: &mut Box<dyn Any>, image: &mut Image) {
        MultiUndoableAction::undo(self.as_mut(), layer_data.as_mut().downcast_mut().unwrap(), image)
    }
}

/// Wrapper struct for handling the vector of
/// `layer_data`s
pub struct MultiUndoableActionWrapper {
    action: Box<dyn MultiUndoableActionWrapperTrait>,
    layer_datas: Option<Vec<Box<dyn Any>>>,
}

impl MultiUndoableActionWrapper {
    fn from_action<D: 'static>(action: Box<dyn MultiUndoableAction<LayerData = D>>) -> Self {
        Self {
            action: Box::new(action),
            layer_datas: None,
        }
    }

    // Idempotent.
    fn init_layer_datas(&mut self, image: &mut LayeredImage) {
        if let None = self.layer_datas {
            let mut layer_datas = Vec::new();

            for idx in image.layer_indices() {
                layer_datas.push(self.action.new_layer_data(image.image_at_layer_mut(idx)))
            }

            self.layer_datas = Some(layer_datas);
        }
    }

    pub fn exec(&mut self, image: &mut LayeredImage) {
        self.init_layer_datas(image);
        let layer_datas = self.layer_datas.as_mut().unwrap();

        for (i, ld) in layer_datas.iter_mut().enumerate() {
            self.action.exec(ld, image.image_at_layer_mut(LayerIndex::from_usize(i)));
        }

        image.update_drawable_sizes();
    }

    pub fn undo(&mut self, image: &mut LayeredImage) {
        self.init_layer_datas(image);
        let layer_datas = self.layer_datas.as_mut().unwrap();

        for (i, ld) in layer_datas.iter_mut().enumerate() {
            self.action.undo(ld, image.image_at_layer_mut(LayerIndex::from_usize(i)));
        }

        image.update_drawable_sizes();
    }
}

impl ImageHistory {
    pub fn exec_doable_action<A>(&mut self, action: A)
    where
        A: DoableAction,
    {
        let name = action.name();
        action.exec(self.now_mut());
        self.push_current_state(name);
    }

    pub fn exec_undoable_action(&mut self, mut action: Box<dyn UndoableAction<Image>>) {
        let (mod_pix, layer) = self.now.img.get_and_reset_modified();
        if !mod_pix.is_empty() {
            // if self is modified in any way, push the sate with Anon
            self.push_current_state(ActionName::Anonymous);
        }

        self.now_mut().apply_action(&mut action, layer);
        self.push_undo_action(action, layer);
    }

    fn push_undo_action(&mut self, action: Box<dyn UndoableAction<Image>>, layer: LayerIndex) {
        // assume the current state is already pushed (this is done in `exec_undoable_action`)
        // otherwise an anonymous undo step might get lost

        let culprit = action.name();
        let image_diff = ImageDiff::SingleManualUndo(action, layer);
        let image_state_diff = ImageStateDiff::new(image_diff, self.now.id, self.id_counter, culprit);

        self.push_state_diff(image_state_diff)
    }

    pub fn exec_multi_undoable_action<D: 'static>(&mut self, action: Box<dyn MultiUndoableAction<LayerData = D>>) {
        let culprit = action.name();
        let wrapper_struct = MultiUndoableActionWrapper::from_action(action);
        let diff = ImageDiff::MultiManualUndo(wrapper_struct);

        self.apply_and_push_diff(diff, culprit);
    }
}

impl LayeredImage {
    pub fn apply_action(&mut self, action: &mut Box<dyn UndoableAction<Image>>, layer: LayerIndex) {
        action.exec(self.image_at_layer_mut(layer));
        self.re_compute_active_drawables();
    }

    pub fn unapply_action(&mut self, action: &mut Box<dyn UndoableAction<Image>>, layer: LayerIndex) {
        action.undo(self.image_at_layer_mut(layer));
        self.re_compute_active_drawables();
    }
}
