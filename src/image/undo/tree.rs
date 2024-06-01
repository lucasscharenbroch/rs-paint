use super::{ImageStateDiff, ImageDiff};

use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct UndoNode {
    parent: Option<Weak<UndoNode>>,
    children: RefCell<Vec<Rc<UndoNode>>>,
    value: Rc<ImageStateDiff>,
}

impl UndoNode {
    fn new(parent_p: &Rc<UndoNode>, diff: ImageStateDiff) -> Self {
        let parent = Some(Rc::downgrade(parent_p));

        UndoNode {
            parent,
            value: Rc::new(diff),
            children: RefCell::new(vec![]),
        }
    }
}

pub struct UndoTree {
    head: Rc<UndoNode>,
    current: Rc<UndoNode>,
}

impl UndoTree {
    pub fn new() -> Self {
        const NULL_DIFF: ImageStateDiff = ImageStateDiff {
            image_diff: ImageDiff::Null,
            old_id: 0,
            new_id: 0,
            culprit: crate::image::undo::action::ActionName::Anonymous,
        };

        let root = UndoNode {
            parent: None,
            children: RefCell::new(vec![]),
            value: Rc::new(NULL_DIFF),
        };

        let root_p = Rc::new(root);

        UndoTree {
            head: Rc::clone(&root_p),
            current: root_p,
        }
    }

    pub fn commit(&mut self, diff: ImageStateDiff) {
        let new_current = Rc::new(UndoNode::new(&self.current, diff));
        self.current.children.borrow_mut().push(Rc::clone(&new_current));
        self.current = new_current;
    }

    pub fn undo(&mut self) -> Option<Rc<ImageStateDiff>> {
        if let Some(ref parent_p) = self.current.parent {
            let ret = self.current.value.clone();
            self.current = parent_p.upgrade().unwrap();
            Some(ret)
        } else {
            None
        }
    }

    // just return the first child for this one
    // (no primitive binding for multi-level undo)
    pub fn redo(&mut self) -> Option<Rc<ImageStateDiff>> {
        let new_current = if let Some(new_current) = self.current.children.borrow().get(0) {
            new_current.clone()
        } else {
            return None;
        };

        self.current = new_current;
        Some(self.current.value.clone())
    }
}
