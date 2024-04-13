use super::canvas::Canvas;
use super::UiState;
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use gtk::pango;

use gtk::{prelude::*, Box as GBox, Orientation, Label};

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

    pub fn widget(&self, is_active: bool) -> GBox {
        let res = GBox::builder()
            .orientation(Orientation::Horizontal)
            .build();

        let attributes = pango::AttrList::new();
        if is_active {
            let bold = pango::AttrInt::new_weight(pango::Weight::Bold);
            attributes.insert(bold);
        }

        let text_label = Label::builder()
            .label(self.name.as_str())
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .attributes(&attributes)
            .build();

        res.append(&text_label);

        res
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

    pub fn widget(&self) -> GBox {
        let res = GBox::builder()
            .orientation(Orientation::Horizontal)
            .build();

        self.tabs.iter().zip(0..)
            .for_each(|(tab, i)| res.append(&tab.widget(self.active_idx.map(|ai| ai == i).unwrap_or(false))));

        res
    }
}
