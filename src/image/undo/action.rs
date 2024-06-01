use crate::image::{DrawableImage, Image, UnifiedImage};
use super::{ImageDiff, ImageHistory, ImageStateDiff};

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
}

pub trait DoableAction {
    fn name(&self) -> ActionName;
    fn exec(&self, image: &mut UnifiedImage);
    // undo is imlpicit: it will be done by diffing the image
}

pub trait UndoableAction {
    fn name(&self) -> ActionName;
    fn exec(&self, image: &mut Image);
    fn undo(&self, image: &mut Image); // explicit undo provided
}

pub trait StaticUndoableAction: UndoableAction {
    fn dyn_clone(&self) -> Box<dyn UndoableAction>;
}

impl ImageHistory {
    pub fn exec_doable_action(&mut self, action: &impl DoableAction) {
        action.exec(self.now_mut());
        self.push_current_state(action.name());
    }

    pub fn exec_undoable_action(&mut self, action: Box<dyn UndoableAction>) {
        let (mod_pix, maybe_new_image) = self.now.img.get_and_reset_modified();
        if !mod_pix.is_empty() || maybe_new_image.is_some() {
            // if self is modified in any way, push the sate with Anon
            self.push_current_state(ActionName::Anonymous);
        }

        self.now_mut().apply_action(&action);
        self.push_undo_action(action);
    }

    fn push_undo_action(&mut self, action: Box<dyn UndoableAction>) {
        // assume the current state is already pushed (this is done in `exec_undoable_action`)
        // otherwise an anonymous undo step might get lost

        let culprit = action.name();
        let image_diff = ImageDiff::ManualUndo(action);
        let image_state_diff = ImageStateDiff::new(image_diff, self.now.id, self.id_counter, culprit);

        self.push_state_diff(image_state_diff)
    }
}

impl UnifiedImage {
    pub fn apply_action(&mut self, action: &Box<dyn UndoableAction>) {
        action.exec(&mut self.image);
        self.drawable = DrawableImage::from_image(&self.image);
    }

    pub fn unapply_action(&mut self, action: &Box<dyn UndoableAction>) {
        action.undo(&mut self.image);
        self.drawable = DrawableImage::from_image(&self.image);
    }
}
