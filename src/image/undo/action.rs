use crate::image::{DrawableImage, Image, LayerIndex, LayeredImage};
use super::{ImageDiff, ImageHistory, ImageStateDiff};

#[derive(Debug)] // TODO remove/change
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
}

pub trait DoableAction {
    fn name(&self) -> ActionName;
    fn exec(self, image: &mut LayeredImage);
    // undo is imlpicit: it will be done by diffing the image
}

pub trait StaticDoableAction: DoableAction {
    fn dyn_clone(&self) -> Box<dyn DoableAction>;
}

pub trait UndoableAction {
    fn name(&self) -> ActionName;
    fn exec(&mut self, image: &mut Image);
    fn undo(&mut self, image: &mut Image); // explicit undo provided
}

pub trait StaticUndoableAction: UndoableAction {
    fn dyn_clone(&self) -> Box<dyn UndoableAction>;
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

    pub fn exec_undoable_action(&mut self, mut action: Box<dyn UndoableAction>) {
        let (mod_pix, layer) = self.now.img.get_and_reset_modified();
        if !mod_pix.is_empty() {
            // if self is modified in any way, push the sate with Anon
            self.push_current_state(ActionName::Anonymous);
        }

        self.now_mut().apply_action(&mut action, layer);
        self.push_undo_action(action, layer);
    }

    fn push_undo_action(&mut self, action: Box<dyn UndoableAction>, layer: LayerIndex) {
        // assume the current state is already pushed (this is done in `exec_undoable_action`)
        // otherwise an anonymous undo step might get lost

        let culprit = action.name();
        let image_diff = ImageDiff::ManualUndo(action, layer);
        let image_state_diff = ImageStateDiff::new(image_diff, self.now.id, self.id_counter, culprit);

        self.push_state_diff(image_state_diff)
    }
}

impl LayeredImage {
    pub fn apply_action(&mut self, action: &mut Box<dyn UndoableAction>, layer: LayerIndex) {
        action.exec(self.image_at_layer_mut(layer));
        self.re_compute_drawable();
    }

    pub fn unapply_action(&mut self, action: &mut Box<dyn UndoableAction>, layer: LayerIndex) {
        action.undo(self.image_at_layer_mut(layer));
        self.re_compute_drawable();
    }
}
