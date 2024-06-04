use crate::image::UnifiedImage;

use super::{ImageStateDiff, ImageDiff, ImageState};

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use glib_macros::clone;
use gtk::{pango, prelude::*, Align, Box as GBox, Button, Orientation, ScrolledWindow, Widget, Label};
use gtk::{glib, graphene};
use core::time::Duration;
use std::collections::{HashMap, VecDeque, HashSet};

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

    fn new_widget(button: &Button) -> GBox {
        let widget = GBox::builder()
            .halign(Align::Start)
            .spacing(0)
            .build();

        widget.append(button);

        widget
    }

    fn new(parent_p: &Rc<UndoNode>, diff: ImageStateDiff) -> Self {
        let parent = Some(Rc::downgrade(parent_p));

        let label = Label::new(Some(format!("{:?}", diff.culprit).as_str()));

        let button = Button::builder()
            .child(&label)
            .build();

        let widget = Self::new_widget(&button);

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

    fn id(&self) -> usize {
        // while nodes represent commits, we'll associate
        // them with the state *after* the commit (like with git)
        self.value.new_id
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

        let button = Button::builder()
            .child(&label)
            .build();

        let widget = UndoNode::new_widget(&button);

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

    fn win_scroll_to_widget(window: &ScrolledWindow, widget: &impl IsA<Widget>) {
        let reference_pt = widget.compute_point(window, &graphene::Point::new(0.0, 0.0)).unwrap();

        let v_adjustment = window.vadjustment();
        let v_value = v_adjustment.value();
        let v_page_size = v_adjustment.page_size();
        let y0 = reference_pt.y() as f64;
        let widget_height = widget.height() as f64;
        let y1 = y0 + widget_height;
        let v_overshoot = 0.25 * v_page_size;
        const V_MARGIN: f64 = 20.0;

        if y0 < (0.0 + V_MARGIN) {
            v_adjustment.set_value(v_value + y0 - v_overshoot);
        } else if y1 > (v_page_size - V_MARGIN) {
            v_adjustment.set_value(v_value + y1 - v_page_size + v_overshoot);
        }

        let h_adjustment = window.hadjustment();
        let h_value = h_adjustment.value();
        let h_page_size = h_adjustment.page_size();
        let x0 = reference_pt.x() as f64;
        let widget_width = widget.width() as f64;
        let x1 = x0 + widget_width;
        let h_overshoot = 0.25 * h_page_size;
        const H_MARGIN: f64 = 20.0;

        if x0 < (0.0 + H_MARGIN) {
            h_adjustment.set_value(h_value + x0 - h_overshoot);
        } else if x1 > (h_page_size - H_MARGIN) {
            h_adjustment.set_value(h_value + x1 - h_page_size + h_overshoot);
        }
    }

    fn win_scroll_to_widget_after_resize(window: &ScrolledWindow, widget: &impl IsA<Widget>) {
            // Hack: if we call node.widget.compute_point directly,
            // it will always return (0.0, 0.0), because the size isn't calculated yet.
            // I don't know of any better way to wait for a layout-update,
            // so we just wait 50 milliseconds, and hope that it's executed
            // after the resize.
            glib::timeout_add_local_once(Duration::from_millis(50), clone!(@strong window, @strong widget => move || {
                Self::win_scroll_to_widget(&window, &widget);
            }));
    }

    pub fn scroll_to_active_node_after_resize(&self) {
        Self::win_scroll_to_widget_after_resize(&self.widget, &self.current.widget);
    }

    fn set_node_is_active(&self, node: &UndoNode, is_active: bool) {
        node.set_active(is_active);

        // undo menu is up           && we're setting this node to active
        // vvvvvvvvvvvvvvvvvvvvvvvvv    vvvvvvvvv
        if node.widget.is_realized() && is_active {
            Self::win_scroll_to_widget_after_resize(&self.widget, &node.widget);
        }
    }

    // Finds the path from self.current to the
    // node with the given id, returning the diff
    // functions necessary to convert the image to the target,
    // setting current to the target.
    pub fn traverse_to(&self, target_id: usize) -> Vec<Box<dyn Fn(&mut ImageState)>> {
        let mut q = VecDeque::new();
        // parent map (also serves as visited map)
        let mut pi: HashMap<usize, Option<Rc<UndoNode>>> = HashMap::new();

        q.push_back(self.current.clone());
        pi.insert(self.current.id(), None);

        while let Some(curr) = q.pop_front() {
            let mut neighbors = curr.children.borrow().iter()
                .cloned()
                .collect::<Vec<_>>();

            // add parent (if present)
            neighbors.extend(curr.parent.iter().map(|p| p.upgrade().unwrap()));

            for neigh in neighbors.iter() {
                if pi.contains_key(&neigh.id()) {
                    continue; // already visited neigh
                }

                pi.insert(neigh.id(), Some(curr.clone()));
                q.push_back(neigh.clone());

                if neigh.id() == target_id {
                    // found target: now form diff chain, walking backwards from target to self.current
                    let mut curr = neigh;
                    let mut diff_chain: Vec<Box<dyn Fn(&mut ImageState)>> = vec![
                        // the last thing we do is apply the target commit's diff
                        Box::new(clone!(@strong curr => move |img| curr.value.apply_to(img)))
                    ];

                    while let Some(pred) = &pi[&curr.id()] {
                        if let Some(parent) = &curr.parent {
                            if parent.upgrade().unwrap().id() == pred.id() {
                                // pred is parent: curr needs to be unapplied
                                diff_chain.push(Box::new(
                                    clone!(@strong curr => move |img| curr.value.unapply_to(img))
                                ));
                            }
                        } else {
                            // pred is child: curr needs to be applied
                            diff_chain.push(Box::new(
                                clone!(@strong curr => move |img| curr.value.apply_to(img))
                            ));

                        }

                        curr = pred;
                    }

                    diff_chain.reverse();
                    return diff_chain;
                }
            }
        }

        panic!("Couldn't reach node with id {target_id}");
    }
}
