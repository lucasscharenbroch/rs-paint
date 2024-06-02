use super::{ImageStateDiff, ImageDiff};

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use gtk::{prelude::*, Box as GBox, Orientation, Widget};

struct UndoNode {
    parent: Option<Weak<UndoNode>>,
    children: RefCell<Vec<Rc<UndoNode>>>,
    value: Rc<ImageStateDiff>,
    widget: GBox,
    container: Rc<GBox>, // possibly inherited from parent
}

impl UndoNode {
    fn new_container() -> Rc<GBox> {
        let gbox = GBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_start(25)
            .build();

        Rc::new(gbox)
    }

    fn new(parent_p: &Rc<UndoNode>, diff: ImageStateDiff) -> Self {
        let parent = Some(Rc::downgrade(parent_p));

        let inner_widget = gtk::Label::new(Some(format!("{:?}", diff.culprit).as_str()));
        let widget = GBox::new(Orientation::Vertical, 0);
        widget.append(&inner_widget);

        let container = if parent_p.children.borrow().len() == 0 {
            // first child: use parent's container
            Rc::clone(&parent_p.container)
        } else {
            let container = Self::new_container();

            parent_p.container.insert_child_after(&*container, Some(&parent_p.widget));

            container
        };

        container.append(&widget);

        UndoNode {
            parent,
            value: Rc::new(diff),
            children: RefCell::new(vec![]),
            widget,
            container,
        }
    }
}

pub struct UndoTree {
    root: Rc<UndoNode>,
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

        let widget = GBox::new(Orientation::Vertical, 0);
        let container = Rc::new(GBox::new(Orientation::Vertical, 4));

        widget.append(&*container);

        let root = UndoNode {
            parent: None,
            children: RefCell::new(vec![]),
            value: Rc::new(NULL_DIFF),
            widget,
            container,
        };

        let root_p = Rc::new(root);

        UndoTree {
            root: Rc::clone(&root_p),
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

    pub fn widget(&self) -> &impl IsA<Widget> {
        &self.root.widget
    }
}
