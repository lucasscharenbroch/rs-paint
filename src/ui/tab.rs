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
    tabs: Vec<Tab>,
    active_idx: Option<usize>
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

    pub fn append_tab(&mut self, tab: Tab) -> usize {
        self.tabs.push(tab);
        self.tabs.len() - 1
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_idx.and_then(|i| self.tabs.get(i))
    }

    pub fn get_tab(&self, idx: usize) -> Option<&Tab> {
        self.tabs.get(idx)
    }

    pub fn set_tab(&mut self, idx: usize) {
        self.active_idx = Some(idx);
    }
}
