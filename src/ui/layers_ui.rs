use gtk::{prelude::*, Ordering};

use crate::image::{LayeredImage, LayerIndex};

use super::canvas::Canvas;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

/// Wrapper for the box/frame that represents a layer in `LayersUi`:
/// this object has no direct ties to any specific image (it accesses
/// its image by suppling an index to a `Rc<RefCell<Canvas>>`: both
/// of which are stored in the draw-function closure of `thumbnail_widget`)
/// and contains no stateful information (except the index)
struct LayerTab {
    widget: gtk::Box,
    thumbnail_widget: gtk::DrawingArea,
}

impl LayerTab {
    fn new(canvas_p: &Rc<RefCell<Canvas>>, layer_index: LayerIndex) -> Self {
        const MAX_DIMENSION: i32 = 100;

        let thumbnail_widget = gtk::DrawingArea::builder()
            .content_width(MAX_DIMENSION)
            .content_height(MAX_DIMENSION)
            .build();

        thumbnail_widget.set_draw_func(clone!(@strong canvas_p => move |area, cr, width, height| {
            canvas_p.borrow_mut().draw_layer_thumbnail(area, cr, width, height, layer_index);
        }));

        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["layer-tab"])
            .spacing(10)
            .build();

        widget.append(&thumbnail_widget);
        widget.append(&gtk::Label::new(Some(format!("{layer_index:?}").as_str())));

        Self {
            widget,
            thumbnail_widget,
        }
    }

    fn update_aspect_ratio(&self, canvas_p: &Rc<RefCell<Canvas>>) {
        let aspect_ratio = canvas_p.borrow().image_width() as f64 /
                           canvas_p.borrow().image_height() as f64;

        const MAX_DIMENSION: i32 = 100;

        let (w, h) = if aspect_ratio >= 1.0 {
            (MAX_DIMENSION, (MAX_DIMENSION as f64 / aspect_ratio).ceil() as i32)
        } else {
            ((MAX_DIMENSION as f64 * aspect_ratio).ceil() as i32, MAX_DIMENSION)
        };

        self.thumbnail_widget.set_content_width(w);
        self.thumbnail_widget.set_content_height(h);
        self.thumbnail_widget.queue_resize();
    }
}

/// Wrapper struct for the ui within the image layers dialog
pub struct LayersUi {
    widget: gtk::Box,
    layer_tabs: RefCell<Vec<LayerTab>>,
    canvas_p: Option<Rc<RefCell<Canvas>>>,
    /// Which LayerTab has the "active" visual cue (if any)
    last_active_idx: RefCell<Option<usize>>,
}

impl LayersUi {
    pub fn new() -> Self {
        LayersUi {
            widget: gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build(),
            layer_tabs: RefCell::new(Vec::new()),
            canvas_p: None,
            last_active_idx: RefCell::new(None),
        }
    }

    fn new_tab(&self) {
        let layer_idx = LayerIndex::from_usize(self.layer_tabs.borrow().len());
        let new_tab = LayerTab::new(&self.canvas_p.as_ref().unwrap(), layer_idx);
        self.widget.prepend(&new_tab.widget);
        self.layer_tabs.borrow_mut().push(new_tab);
    }

    fn pop_tab(&self) {
        let tab = self.layer_tabs.borrow_mut().pop().unwrap();
        self.widget.remove(&tab.widget);
    }

    /// Finishes initialization (populating the widget)
    pub fn finish_init(&mut self, canvas_p: Rc<RefCell<Canvas>>) {
        self.canvas_p = Some(canvas_p.clone());

        for _ in 0..canvas_p.borrow().image_ref().num_layers() {
            self.new_tab();
        }

        let new_button = gtk::Button::builder()
            .label("New")
            .build();

        new_button.connect_clicked(move |_button| {
            canvas_p.borrow_mut().append_layer(
                gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
            );
            canvas_p.borrow_mut().update();
        });

        self.widget.append(&new_button);
    }

    pub fn widget(&self) -> impl gtk::prelude::IsA<gtk::Widget> {
        self.widget.clone()
    }

    /// Redraw, add/remove tabs if necessary
    pub fn update(&self, num_layers: usize, active_idx: LayerIndex) {
        if let Some(i) = self.last_active_idx.borrow().as_ref() {
            self.layer_tabs.borrow()[*i].widget.remove_css_class("active-layer-tab")
        }

        while self.layer_tabs.borrow().len() < num_layers {
            self.new_tab()
        }

        while self.layer_tabs.borrow().len() > num_layers {
            self.pop_tab();
        }

        *self.last_active_idx.borrow_mut() = Some(active_idx.to_usize());
        self.layer_tabs.borrow_mut()[active_idx.to_usize()].widget.add_css_class("active-layer-tab");

        self.layer_tabs.borrow().iter().for_each(|tab| {
            tab.thumbnail_widget.queue_draw()
        });
    }
}