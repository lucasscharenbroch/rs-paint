use super::canvas::Canvas;
use super::UiState;
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use gtk::{pango, DrawingArea};
use glib_macros::clone;

use gtk::{prelude::*, Box as GBox, Orientation, Label, Button};

pub struct Tab {
    pub canvas_p: Rc<RefCell<Canvas>>,
    name: String,
}

impl Tab {
    pub fn new(canvas_p: &Rc<RefCell<Canvas>>, name: &str) -> Self {
        Tab {
            canvas_p: canvas_p.clone(),
            name: String::from(name),
        }
    }

    pub fn widget(&self, ui_p: &Rc<RefCell<UiState>>, idx: usize, is_active: bool) -> GBox {
        let res = GBox::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        let attributes = pango::AttrList::new();
        if is_active {
            let bold = pango::AttrInt::new_weight(pango::Weight::Bold);
            attributes.insert(bold);
        }

        let text_label = Label::builder()
            .label(self.name.as_str())
            .attributes(&attributes)
            .build();

        let container = GBox::builder()
            .orientation(Orientation::Horizontal)
            .build();

        let canvas_p = self.canvas_p.clone();
        let aspect_ratio = self.canvas_p.borrow().image_width() as f64 /
                           self.canvas_p.borrow().image_height() as f64;

        const MAX_DIMENSION: i32 = 30;

        let (w, h) = if aspect_ratio >= 1.0 {
            (MAX_DIMENSION, (MAX_DIMENSION as f64 / aspect_ratio).ceil() as i32)
        } else {
            ((MAX_DIMENSION as f64 * aspect_ratio).ceil() as i32, MAX_DIMENSION)
        };

        let thumbnail_area = DrawingArea::builder()
            .content_width(w)
            .content_height(h)
            .margin_start(3)
            .build();

        thumbnail_area.set_draw_func(clone!(@strong canvas_p => move |area, cr, width, height| {
            canvas_p.borrow_mut().draw_thumbnail(area, cr, width, height);
        }));

        container.append(&text_label);
        container.append(&thumbnail_area);

        let name_button = Button::builder()
            .child(&container)
            .build();

        name_button.connect_clicked(clone!(@strong ui_p => move |_| {
            ui_p.borrow_mut().set_tab(idx);
            UiState::update_tabbar_widget(&ui_p);
        }));

        let x_button = Button::builder()
            .label("x")
            .build();

        x_button.connect_clicked(clone!(@strong ui_p => move |_| {
            ui_p.borrow_mut().close_tab(idx);
            UiState::update_tabbar_widget(&ui_p);
        }));

        res.append(&name_button);
        res.append(&x_button);

        res
    }

    pub fn confirm_close(&self, ui: &UiState) -> bool {
        true // TODO (!modified_since_saved) ? true : prompt
    }
}

pub struct Tabbar {
    pub tabs: Vec<Tab>,
    pub active_idx: Option<usize>
}

impl Tabbar {
    pub fn new() -> Self {
        Tabbar {
            tabs: vec![],
            active_idx: None,
        }
    }

    pub fn widget(&self, ui_p: &Rc<RefCell<UiState>>) -> GBox {
        let res = GBox::builder()
            .orientation(Orientation::Horizontal)
            .build();

        self.tabs.iter().zip(0..)
            .for_each(|(tab, i)| res.append(&tab.widget(ui_p, i, self.active_idx.map(|ai| ai == i).unwrap_or(false))));

        res
    }
}
