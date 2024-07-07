use super::{ImageStateDiff, ImageDiff, ImageState};
use super::action::ActionName;
use super::{ImageHistory, DrawablesToUpdate};

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
    recent_child_idx: RefCell<usize>,
    value: Rc<RefCell<ImageStateDiff>>,
    widget: GBox,
    label: Label,
    button: Button,
    container: Rc<GBox>, // possibly inherited from parent
}

impl PartialEq for UndoNode {
    fn eq(&self, other: &UndoNode) -> bool {
        self.value.borrow().new_id == other.value.borrow().new_id
    }
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

    fn new_button(label: &Label) -> Button {
        let button = Button::builder()
            .child(label)
            .css_classes(["undo-tree-node-button"])
            .build();

        button
    }

    fn new(parent_p: &Rc<UndoNode>, diff: ImageStateDiff) -> Self {
        let parent = Some(Rc::downgrade(parent_p));

        let label = Label::new(Some(diff.culprit.to_str()));
        let button = Self::new_button(&label);
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
            value: Rc::new(RefCell::new(diff)),
            children: RefCell::new(vec![]),
            recent_child_idx: RefCell::new(0),
            widget,
            label,
            button,
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
        self.value.borrow().new_id
    }

    fn connect_hooks(&self, tree: &UndoTree) {
        // hooks should be set before calling this
        let mod_hist = tree.mod_hist.as_ref().unwrap().clone();
        let update_canvas = tree.update_canvas.as_ref().unwrap().clone();
        let target_id = self.id();

        self.button.connect_clicked(clone!(@strong mod_hist, @strong update_canvas => move |_| {
            let f = Box::new(move |hist: &mut ImageHistory| {
                hist.migrate_to_commit(target_id);
            });

            mod_hist(f);
            update_canvas();
        }));
    }
}

pub struct UndoTree {
    root: Rc<UndoNode>,
    current: Rc<UndoNode>,
    widget: ScrolledWindow,
    // `mod_hist` is a gross hack to avoid explicitly wrapping `ImageHistory`
    // in a pointer (this is undesirable because `Canvas` uses lots of
    // returned references to its `image_hist` field, which is annoying to
    // do when it's wrapped in a pointer).
    // It's used to pass a self-pointer into widget handlers.
    mod_hist: Option<Rc<dyn Fn(Box<dyn Fn(&mut ImageHistory)>)>>,
    // Yet another attempt to keep some semblance of encapsulation.
    // It's probably ideal to refactor the ui out entirely.
    update_canvas: Option<Rc<dyn Fn()>>,
}

impl UndoTree {
    pub fn new() -> Self {
        const NULL_DIFF: ImageStateDiff = ImageStateDiff {
            image_diff: ImageDiff::Null,
            old_id: 0,
            new_id: 0,
            culprit: ActionName::Anonymous,
        };

        let label = Label::new(Some("(Root)"));
        let button = UndoNode::new_button(&label);
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
            recent_child_idx: RefCell::new(0),
            value: Rc::new(RefCell::new(NULL_DIFF)),
            widget,
            label,
            button,
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
            mod_hist: None,
            update_canvas: None,
        }
    }

    fn update_current(&mut self, new_current: Rc<UndoNode>) {
        self.set_node_is_active(&*self.current, false);
        self.current = new_current;
        self.set_node_is_active(&*self.current, true);
    }

    pub fn commit(&mut self, state_diff: ImageStateDiff) {
        let new_current = Rc::new(UndoNode::new(&self.current, state_diff));
        new_current.connect_hooks(&self);
        self.current.children.borrow_mut().push(Rc::clone(&new_current));
        self.update_current(new_current);
    }

    pub fn undo(&mut self) -> Option<Rc<RefCell<ImageStateDiff>>> {
        if let Some(ref parent_p) = self.current.parent {
            let ret = self.current.value.clone();
            let parent = parent_p.upgrade().unwrap();
            *parent.recent_child_idx.borrow_mut() = parent.children.borrow().iter().position(|c| **c == *self.current).unwrap();
            self.update_current(parent);
            Some(ret)
        } else {
            None
        }
    }

    // just return the first child for this one
    // (no primitive binding for multi-level undo)
    pub fn redo(&mut self) -> Option<Rc<RefCell<ImageStateDiff>>> {
        let new_current = if let Some(new_current) =
                self.current.children.borrow().get(*self.current.recent_child_idx.borrow()) {
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
    pub fn traverse_to(&mut self, target_id: usize) -> Vec<Box<dyn Fn(&mut ImageState, &mut DrawablesToUpdate)>> {
        if target_id == self.current.id() {
            return vec![];
        }

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
                    let target = neigh;
                    let mut curr = neigh;
                    let mut diff_chain: Vec<Box<dyn Fn(&mut ImageState, &mut DrawablesToUpdate)>> = vec![];
                    // `diff_chain` will gather the diff-functions to walk the tree:
                    // there is one diff-function per edge.
                    // To apply an edge: apply its child.
                    // To unapply an edge: unapply its parent.
                    // This sounds a little funny, because the nodes themselves are commits, yet
                    // they also represent the state *after* the respective commit is applied.
                    //                         Y
                    //          (unapply X) /     \ (apply TARGET)
                    //                     X      TARGET
                    //  (unapply CURRENT) /
                    //              CURRENT

                    // walk the edges, in reverse order
                    while let Some(pred) = &pi[&curr.id()] {
                        // it would be nice to use a let-chain here... (it's unstable though)
                        if curr.parent.as_ref().map(|parent| parent.upgrade().unwrap().id() == pred.id()).unwrap_or(false) {
                            // pred is parent: apply the edge (apply curr)
                            diff_chain.push(Box::new(
                                clone!(@strong curr => move |img, to_update| curr.value.borrow_mut().apply_to(img, to_update))
                            ));
                        } else {
                            // pred is child: unapply the edge (unapply pred)
                            diff_chain.push(Box::new(
                                clone!(@strong pred => move |img, to_update| pred.value.borrow_mut().unapply_to(img, to_update))
                            ));
                        }

                        curr = pred;
                    }

                    self.update_current(target.clone());
                    diff_chain.reverse();
                    return diff_chain;
                }
            }
        }

        panic!("Couldn't reach node with id {target_id}");
    }

    pub fn set_hooks(
        &mut self,
        mod_hist: Rc<dyn Fn(Box<dyn Fn(&mut ImageHistory)>)>,
        update_canvas: Rc<dyn Fn()>,
    ) {
        self.mod_hist = Some(mod_hist);
        self.update_canvas = Some(update_canvas);
        self.root.connect_hooks(&self);
    }
}
