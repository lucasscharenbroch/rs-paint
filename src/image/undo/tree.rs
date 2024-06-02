use super::{ImageStateDiff, ImageDiff};

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use glib_macros::clone;
use gtk::{pango, prelude::*, Align, Box as GBox, Label, Orientation, ScrolledWindow, Widget};

struct UndoNode {
    parent: Option<Weak<UndoNode>>,
    children: RefCell<Vec<Rc<UndoNode>>>,
    value: Rc<ImageStateDiff>,
    widget: GBox,
    label: Label,
    container: Rc<GBox>, // possibly inherited from parent
}

impl UndoNode {
    fn new_container() -> Rc<GBox> {
        let gbox = GBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_start(25)
            .margin_bottom(10)
            .halign(Align::Start)
            .build();

        Rc::new(gbox)
    }

    fn new_widget(label: &Label) -> GBox {
        let widget = GBox::builder()
            .halign(Align::Start)
            .spacing(0)
            .build();

        widget.append(label);

        widget
    }

    fn new(parent_p: &Rc<UndoNode>, diff: ImageStateDiff) -> Self {
        let parent = Some(Rc::downgrade(parent_p));

        let label = Label::new(Some(format!("{:?}", diff.culprit).as_str()));
        let widget = Self::new_widget(&label);

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
            label,
            container,
        }
    }

    fn set_active(&self, is_active: bool) {
        let attributes = pango::AttrList::new();

        if is_active {
            let bold = pango::AttrInt::new_weight(pango::Weight::Bold);
            attributes.insert(bold);
        }

        self.label.set_attributes(Some(&attributes));
    }
}

pub struct UndoTree {
    root: Rc<UndoNode>,
    current: Rc<UndoNode>,
    widget: ScrolledWindow,
}

impl UndoTree {
    pub fn new() -> Self {
        const NULL_DIFF: ImageStateDiff = ImageStateDiff {
            image_diff: ImageDiff::Null,
            old_id: 0,
            new_id: 0,
            culprit: crate::image::undo::action::ActionName::Anonymous,
        };

        let label = Label::new(Some("(Root)"));
        let widget = UndoNode::new_widget(&label);

        let container = Rc::new(GBox::builder()
            .orientation(Orientation::Vertical)
            .halign(Align::Start)
            .spacing(4)
            .build());

        container.append(&widget);

        let root = UndoNode {
            parent: None,
            children: RefCell::new(vec![]),
            value: Rc::new(NULL_DIFF),
            widget,
            label,
            container,
        };

        root.set_active(true);

        let root_p = Rc::new(root);

        let widget = ScrolledWindow::builder()
            .child(&*root_p.container)
            .min_content_height(400)
            .min_content_width(250)
            .overlay_scrolling(true)
            // tweak expansion/ spacing?
            // .propagate_natural_height(true)
            // .propagate_natural_width(true)
            // .hexpand(true)
            // .vexpand(true)
            .build();

        UndoTree {
            root: Rc::clone(&root_p),
            current: root_p,
            widget,
        }
    }

    fn update_current(&mut self, new_current: Rc<UndoNode>) {
        self.set_node_is_active(&*self.current, false);
        self.current = new_current;
        self.set_node_is_active(&*self.current, true);
    }

    pub fn commit(&mut self, diff: ImageStateDiff) {
        let new_current = Rc::new(UndoNode::new(&self.current, diff));
        self.current.children.borrow_mut().push(Rc::clone(&new_current));
        self.update_current(new_current);
    }

    pub fn undo(&mut self) -> Option<Rc<ImageStateDiff>> {
        if let Some(ref parent_p) = self.current.parent {
            let ret = self.current.value.clone();
            self.update_current(parent_p.upgrade().unwrap());
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

        self.update_current(new_current);
        Some(self.current.value.clone())
    }

    pub fn widget(&self) -> &impl IsA<Widget> {
        &self.widget
    }

    fn set_node_is_active(&self, node: &UndoNode, is_active: bool) {
        node.set_active(is_active);

        // undo menu is up           && we're setting this node to active
        // vvvvvvvvvvvvvvvvvvvvvvvvv    vvvvvvvvv
        if node.widget.is_realized() && !is_active {

            let node_widget = &node.widget;
            let root_container = &*self.root.container;
            let window = &self.widget;

            // Hack: if we call node.widget.compute_point directly,
            // it will always return (0.0, 0.0), because the size isn't calculated yet.
            // I don't know of any better way to wait for a layout-update,
            // so we just call spawn_future_local, and hope that it's executed
            // after the resize.
            gtk::glib::spawn_future_local(clone!(@strong node_widget, @strong root_container, @strong window => async move {
                let focus_pt = node_widget.compute_point(&root_container, &gtk::graphene::Point::new(0.0, 0.0)).unwrap();
                let v_adjustment = window.vadjustment();
                let value = v_adjustment.value();
                let page_size = v_adjustment.page_size();
                let y = focus_pt.y() as f64;

                if y < value {
                    v_adjustment.set_value(y);
                } else if y > value + page_size {
                    v_adjustment.set_value(y + page_size);
                }
            }));
        }
    }
}
