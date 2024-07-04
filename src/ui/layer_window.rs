use gtk::{prelude::*, Ordering};

use crate::image::{LayeredImage, LayerIndex};

use super::canvas::Canvas;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

/// Wrapper for the box/frame that represents a layer in `LayerWindow`:
/// this object has no direct ties to any specific image (it accesses
/// its image by suppling an index to a `Rc<RefCell<Canvas>>`: both
/// of which are stored in the draw-function closure of `thumbnail_widget`)
/// and contains no stateful information (except the index)
struct LayerTab {
    widget: gtk::CenterBox,
    thumbnail_widget: gtk::DrawingArea,
}

impl LayerTab {
    fn new(canvas_p: &Rc<RefCell<Canvas>>, layer_index: LayerIndex, aspect_ratio: f64) -> Self {
        let (w, h) = Self::wh_from_aspect_ratio(aspect_ratio);

        let thumbnail_widget = gtk::DrawingArea::builder()
            .content_width(w)
            .content_height(h)
            .build();

        thumbnail_widget.set_draw_func(clone!(@strong canvas_p => move |area, cr, width, height| {
            canvas_p.borrow_mut().draw_layer_thumbnail(area, cr, width, height, layer_index);
        }));

        let inner_widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();

        let close_button = gtk::Button::builder()
            .child(&gtk::Image::from_file(crate::icon_file!("x")))
            .build();

        let lock_button = gtk::Button::builder()
            .child(&gtk::Image::from_file(crate::icon_file!("lock")))
            .build();

        let visible_button = gtk::Button::builder()
            .child(&gtk::Image::from_file(crate::icon_file!("eyeball")))
            .build();

        let button_widget = gtk::CenterBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .start_widget(&close_button)
            .center_widget(&lock_button)
            .end_widget(&visible_button)
            .build();

        // don't allow removal of base layer
        if layer_index == LayerIndex::BaseLayer {
            button_widget.set_start_widget(None::<&gtk::Box>);
        }

        inner_widget.append(&thumbnail_widget);
        inner_widget.append(&gtk::Label::new(Some(format!("{layer_index:?}").as_str())));

        let widget = gtk::CenterBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["layer-tab"])
            .start_widget(&inner_widget)
            .end_widget(&button_widget)
            .build();

        let click_handler = gtk::GestureClick::new();

        click_handler.connect_pressed(clone!(@strong canvas_p => move |_, _, _, _| {
            canvas_p.borrow_mut().focus_layer(layer_index);
        }));

        close_button.connect_clicked(clone!(@strong canvas_p => move |_button| {
            canvas_p.borrow_mut().remove_layer(layer_index);
        }));

        widget.add_controller(click_handler);

        Self {
            widget,
            thumbnail_widget,
        }
    }

    fn wh_from_aspect_ratio(aspect_ratio: f64) -> (i32, i32) {
        const MAX_DIMENSION: i32 = 100;

        let (w, h) = if aspect_ratio >= 1.0 {
            (MAX_DIMENSION, (MAX_DIMENSION as f64 / aspect_ratio).ceil() as i32)
        } else {
            ((MAX_DIMENSION as f64 * aspect_ratio).ceil() as i32, MAX_DIMENSION)
        };

        (w, h)
    }

    fn update_aspect_ratio(&self, aspect_ratio: f64) {
        let (w, h) = Self::wh_from_aspect_ratio(aspect_ratio);

        self.thumbnail_widget.set_content_width(w);
        self.thumbnail_widget.set_content_height(h);
        self.thumbnail_widget.queue_resize();
    }
}

/// Wrapper struct for the ui within the image layers dialog
pub struct LayerWindow {
    tab_wrapper: gtk::Box,
    scrolled_window: gtk::ScrolledWindow,
    outer_wrapper: gtk::Box,
    layer_tabs: RefCell<Vec<LayerTab>>,
    canvas_p: Option<Rc<RefCell<Canvas>>>,
    /// Which LayerTab has the "active" visual cue (if any)
    last_active_idx: RefCell<Option<usize>>,
    /// Aspect ratio of all thumbnails in `layer_tabs`
    last_aspect_ratio: RefCell<f64>,
}

impl LayerWindow {
    pub fn new() -> Self {
        let tab_wrapper = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();

        let scrolled_window = gtk::ScrolledWindow::builder()
            .min_content_height(400)
            .min_content_width(250)
            .overlay_scrolling(true)
            .child(&tab_wrapper)
            .build();

        let outer_wrapper = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(4)
                .build();

        outer_wrapper.append(&scrolled_window);

        LayerWindow {
            tab_wrapper,
            scrolled_window,
            outer_wrapper,
            layer_tabs: RefCell::new(Vec::new()),
            canvas_p: None,
            last_active_idx: RefCell::new(None),
            last_aspect_ratio: RefCell::new(1.0),
        }
    }

    fn new_tab(&self, aspect_ratio: f64) {
        let layer_idx = LayerIndex::from_usize(self.layer_tabs.borrow().len());
        let new_tab = LayerTab::new(
            &self.canvas_p.as_ref().unwrap(),
            layer_idx,
            aspect_ratio,
        );
        self.tab_wrapper.prepend(&new_tab.widget);
        self.layer_tabs.borrow_mut().push(new_tab);
    }

    fn pop_tab(&self) {
        let tab = self.layer_tabs.borrow_mut().pop().unwrap();
        self.tab_wrapper.remove(&tab.widget);
    }

    /// Finishes initialization (populating the widget)
    pub fn finish_init(&mut self, canvas_p: Rc<RefCell<Canvas>>) {
        self.canvas_p = Some(canvas_p.clone());

        let aspect_ratio = canvas_p.borrow().image_width() as f64 /
                                canvas_p.borrow().image_height() as f64;
        *self.last_aspect_ratio.borrow_mut() = aspect_ratio;

        for _ in 0..canvas_p.borrow().image_ref().num_layers() {
            self.new_tab(aspect_ratio);
        }

        let button_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .halign(gtk::Align::Center)
            .build();

        let new_button = gtk::Button::builder()
            .label("New")
            .build();

        new_button.connect_clicked(clone!(@strong canvas_p => move |_button| {
            canvas_p.borrow_mut().append_layer(
                gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
            );
        }));

        let up_icon = gtk::Image::builder()
            .file(crate::icon_file!("up-arrow"))
            .build();

        let down_icon = gtk::Image::builder()
            .file(crate::icon_file!("down-arrow"))
            .build();

        let up_button = gtk::Button::builder()
            .child(&up_icon)
            .build();

        up_button.connect_clicked(clone!(@strong canvas_p => move |_button| {
            let res = canvas_p.borrow_mut().try_move_active_layer_up();
            if let Ok(target_idx) = res {
                canvas_p.borrow_mut().focus_layer(target_idx)
            }
        }));

        let down_button = gtk::Button::builder()
            .child(&down_icon)
            .build();

        down_button.connect_clicked(clone!(@strong canvas_p => move |_button| {
            let res = canvas_p.borrow_mut().try_move_active_layer_down() ;
            if let Ok(target_idx) = res {
                canvas_p.borrow_mut().focus_layer(target_idx)
            }
        }));

        let merge_button = gtk::Button::builder()
            .label("Merge Down")
            .build();

        button_container.append(&new_button);
        button_container.append(&up_button);
        button_container.append(&down_button);
        button_container.append(&merge_button);

        self.outer_wrapper.append(&button_container);
    }

    pub fn widget(&self) -> impl gtk::prelude::IsA<gtk::Widget> {
        self.outer_wrapper.clone()
    }

    /// Redraw, add/remove tabs if necessary
    pub fn update(&self, num_layers: usize, active_idx: LayerIndex, aspect_ratio: f64) {
        if let Some(i) = self.last_active_idx.borrow().as_ref() {
            self.layer_tabs.borrow()[*i].widget.remove_css_class("active-layer-tab")
        }

        while self.layer_tabs.borrow().len() > num_layers {
            self.pop_tab();
        }

        if aspect_ratio != *self.last_aspect_ratio.borrow() {
            *self.last_aspect_ratio.borrow_mut() = aspect_ratio;
            for tab in self.layer_tabs.borrow_mut().iter_mut() {
                tab.update_aspect_ratio(aspect_ratio)
            }
        }

        while self.layer_tabs.borrow().len() < num_layers {
            self.new_tab(aspect_ratio);
        }

        *self.last_active_idx.borrow_mut() = Some(active_idx.to_usize());
        self.layer_tabs.borrow_mut()[active_idx.to_usize()].widget.add_css_class("active-layer-tab");

        self.layer_tabs.borrow().iter().for_each(|tab| {
            tab.thumbnail_widget.queue_draw()
        });
    }
}
