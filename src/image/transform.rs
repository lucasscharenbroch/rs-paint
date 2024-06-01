use super::undo::action::{UndoableAction, StaticUndoableAction, ActionName};
use super::Image;

#[derive(Clone)]
pub enum Flip {
    Horizontal,
    Vertical,
}

impl UndoableAction for Flip {
    fn name(&self) -> ActionName {
        ActionName::Flip
    }

    fn exec(&self, image: &mut Image) {
        let height = image.height;
        let width = image.width;

        match self {
            Self::Vertical => {
                for i in 0..(height / 2) {
                    for j in 0..width {
                        image.swap_pixels((i, j), (height - i - 1, j));
                    }
                }
            },
            Self::Horizontal => {
                for i in 0..height {
                    for j in 0..(width / 2) {
                        image.swap_pixels((i, j), (i, width - j - 1));
                    }
                }
            }
        }
    }

    fn undo(&self, image: &mut Image) {
        // flips are their own inverse
        self.exec(image)
    }
}

impl StaticUndoableAction for Flip {
    fn dyn_clone(&self) -> Box<dyn UndoableAction> {
        Box::new(self.clone())
    }
}
