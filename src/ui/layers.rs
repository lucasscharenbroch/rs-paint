use gtk::prelude::*;

use crate::image::{LayeredImage, LayerIndex};

use super::canvas::Canvas;
use std::rc::Rc;
use std::cell::RefCell;

/// Wrapper for the box/frame that represents
/// a layer in `LayersUi`
struct LayerTab {
    widget: gtk::Box,
    thumbnail_widget: gtk::DrawingArea,
    layer_index: LayerIndex,
}

impl LayerTab {
    fn new(canvas_p: &Rc<RefCell<Canvas>>, layer_index: LayerIndex) -> Self {
        let thumbnail_widget = gtk::DrawingArea::new();

        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["layer-tab"])
            .build();

        widget.append(&thumbnail_widget);
        widget.append(&gtk::Label::new(Some(format!("{layer_index:?}").as_str())));

        Self {
            widget,
            thumbnail_widget,
            layer_index,
        }
    }
}

/// Wrapper struct for the ui within the image layers dialog
pub struct LayersUi {
    widget: gtk::Box,
    layer_buttons: Vec<LayerTab>,
    canvas_p: Option<Rc<RefCell<Canvas>>>,
}

impl LayersUi {
    pub fn new() -> Self {
        LayersUi {
            widget: gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build(),
            layer_buttons: Vec::new(),
            canvas_p: None,
        }
    }

    /// Finishes initialization (populating the widget)
    pub fn finish_init(&mut self, canvas_p: Rc<RefCell<Canvas>>) {
        for layer_idx in canvas_p.borrow().image_ref().layer_indices() {
            let new_button = LayerTab::new(&canvas_p, layer_idx);
            self.widget.append(&new_button.widget);
            self.layer_buttons.push(new_button);
        }

        self.canvas_p = Some(canvas_p);
    }

    pub fn widget(&self) -> impl gtk::prelude::IsA<gtk::Widget> {
        self.widget.clone()
    }
}
