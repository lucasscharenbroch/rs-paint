use super::canvas::Canvas;
use super::UiState;

use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;
use gtk::prelude::*;

pub struct Tab {
    pub canvas_p: Rc<RefCell<Canvas>>,
    name: String,
    last_export_id: usize,
    drawing_area: Rc<RefCell<gtk::DrawingArea>>,
    widget: gtk::Box,
    container: gtk::Box,
    x_button: gtk::Button,
    x_button_signal_handler_id: Option<gtk::glib::SignalHandlerId>,
    click_handler: gtk::GestureClick,
}

impl Tab {
    pub fn new(canvas_p: &Rc<RefCell<Canvas>>, name: &str) -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["tab"])
            .build();

        let text_label = gtk::Label::builder()
            .label(name)
            .build();

        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .build();

        let canvas_p = canvas_p.clone();
        let aspect_ratio = canvas_p.borrow().image_width() as f64 /
                           canvas_p.borrow().image_height() as f64;

        const MAX_DIMENSION: i32 = 30;

        let (w, h) = if aspect_ratio >= 1.0 {
            (MAX_DIMENSION, (MAX_DIMENSION as f64 / aspect_ratio).ceil() as i32)
        } else {
            ((MAX_DIMENSION as f64 * aspect_ratio).ceil() as i32, MAX_DIMENSION)
        };

        let thumbnail_area = gtk::DrawingArea::builder()
            .content_width(w)
            .content_height(h)
            .margin_start(3)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();

        thumbnail_area.set_draw_func(clone!(@strong canvas_p => move |area, cr, width, height| {
            canvas_p.borrow_mut().draw_thumbnail(area, cr, width, height);
        }));

        container.append(&thumbnail_area);
        container.append(&text_label);

        let click_handler = gtk::GestureClick::new();
        container.add_controller(click_handler.clone());

        let drawing_area = Rc::new(RefCell::new(
            thumbnail_area
        ));

        let x_button = gtk::Button::builder()
            .label("x")
            .css_classes(["tab-x-button"])
            .build();

        widget.append(&container);
        widget.append(&x_button);

        let last_export_id = canvas_p.borrow().undo_id();

        Tab {
            canvas_p: canvas_p.clone(),
            name: String::from(name),
            last_export_id,
            drawing_area,
            widget,
            container,
            x_button,
            x_button_signal_handler_id: None,
            click_handler,
        }
    }

    pub fn update_ui_hooks(&mut self, ui_p: &Rc<RefCell<UiState>>, idx: usize) {
        if let Some(id) = std::mem::replace(&mut self.x_button_signal_handler_id, None) {
            self.x_button.disconnect(id)
        }

        self.x_button_signal_handler_id = Some(
            self.x_button.connect_clicked(clone!(@strong ui_p => move |_| {
                UiState::try_close_tab(&ui_p, idx);
            }))
        );

        self.click_handler.connect_pressed(clone!(@strong ui_p => move |_, _, _, _| {
            ui_p.borrow_mut().set_tab(idx);
        }));
    }

    /// Update the visual cue for this tab (active/not)
    pub fn update_activity_visual(&self, is_active: bool) {
        if is_active {
            self.widget.add_css_class("active-tab");
        } else {
            self.widget.remove_css_class("active-tab");
        }
    }

    pub fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.widget
    }

    pub fn modified_since_export(&self) -> bool {
        self.last_export_id != self.canvas_p.borrow().undo_id()
    }

    pub fn notify_successful_export(&mut self) {
        self.last_export_id = self.canvas_p.borrow().undo_id();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn redraw_thumbnail(&self) {
        self.drawing_area.borrow().queue_draw();
    }
}

pub struct Tabbar {
    pub tabs: Vec<Tab>,
    pub active_idx: Option<usize>,
    widget: gtk::Box,
}

impl Tabbar {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();

        Tabbar {
            tabs: vec![],
            active_idx: None,
            widget,
        }
    }

    pub fn update_widget(&mut self, ui_p: &Rc<RefCell<UiState>>) {
        self.update_activity_visual();

        // detach the current widget from its children, if it exists
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child)
        }

        self.tabs.iter_mut().zip(0..)
            .for_each(|(tab, i)| {
                tab.update_ui_hooks(ui_p, i);
                self.widget.append(tab.widget());
            });
    }

    pub fn update_activity_visual(&self) {
        self.tabs.iter().zip(0..)
            .for_each(|(tab, i)| {
                tab.update_activity_visual(self.active_idx.map(|ai| ai == i).unwrap_or(false));
            });
    }

    pub fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.widget
    }
}
