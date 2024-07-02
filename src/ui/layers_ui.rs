use gtk::prelude::*;

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
        let aspect_ratio = canvas_p.borrow().image_width() as f64 /
                           canvas_p.borrow().image_height() as f64;

        const MAX_DIMENSION: i32 = 100;

        let (w, h) = if aspect_ratio >= 1.0 {
            (MAX_DIMENSION, (MAX_DIMENSION as f64 / aspect_ratio).ceil() as i32)
        } else {
            ((MAX_DIMENSION as f64 * aspect_ratio).ceil() as i32, MAX_DIMENSION)
        };

        let thumbnail_widget = gtk::DrawingArea::builder()
            .content_width(w)
            .content_height(h)
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
}

/// Wrapper struct for the ui within the image layers dialog
pub struct LayersUi {
    widget: gtk::Box,
    layer_tabs: Vec<LayerTab>,
    canvas_p: Option<Rc<RefCell<Canvas>>>,
}

impl LayersUi {
    pub fn new() -> Self {
        LayersUi {
            widget: gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build(),
            layer_tabs: Vec::new(),
            canvas_p: None,
        }
    }

    fn new_tab_with_index(&mut self, canvas_p: &Rc<RefCell<Canvas>>, layer_idx: LayerIndex) {
        let new_tab = LayerTab::new(&canvas_p, layer_idx);
        self.widget.prepend(&new_tab.widget);
        self.layer_tabs.push(new_tab);
    }

    /// Finishes initialization (populating the widget)
    pub fn finish_init(&mut self, canvas_p: Rc<RefCell<Canvas>>) {
        for layer_idx in canvas_p.borrow().image_ref().layer_indices() {
            self.new_tab_with_index(&canvas_p, layer_idx);
        }

        let new_button = gtk::Button::builder()
            .label("New")
            .build();

        new_button.connect_clicked(clone!(@strong canvas_p => move |_button| {
            let self_p = canvas_p.borrow().layers_ui_p().clone();
            let idx = canvas_p.borrow_mut().append_layer(
                        gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
                );

            self_p.borrow_mut()
                .new_tab_with_index(
                    &canvas_p,
                    idx
                );
        }));

        self.widget.append(&new_button);

        self.canvas_p = Some(canvas_p);
    }

    pub fn widget(&self) -> impl gtk::prelude::IsA<gtk::Widget> {
        self.widget.clone()
    }

    pub fn redraw(&self) {
        self.layer_tabs.iter().for_each(|tab| tab.thumbnail_widget.queue_draw());
    }
}
