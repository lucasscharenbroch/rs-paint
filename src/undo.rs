use super::image::{Image, UnifiedImage};

struct ImageDiff { // TODO store diff, not full image
    old: Image,
    new: Image,
}

impl ImageDiff {
    pub fn new(from: &UnifiedImage, to: &UnifiedImage) -> ImageDiff {
        ImageDiff {
            old: from.image().clone(),
            new: to.image().clone(),
        }
    }

    pub fn apply_to(&self, image: &mut UnifiedImage) {
        image.set_image(&self.new);
    }

    pub fn unapply_to(&self, image: &mut UnifiedImage) {
        image.set_image(&self.old);
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
        self.undo_stack.push(ImageDiff::new(&self.last_save, &self.now));
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
