use super::canvas::Canvas;
use super::UiState;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

use gtk::{prelude::*, Box as GBox, Orientation, Label};

pub struct Tab {
    pub canvas_p: Rc<RefCell<Canvas>>,
}

impl Tab {
    pub fn new(canvas_p: &Rc<RefCell<Canvas>>) -> Self {
        Tab {
            canvas_p: canvas_p.clone(),
        }
    }

    pub fn widget(&self) -> GBox {
        let res = GBox::builder()
            .orientation(Orientation::Horizontal)
            .build();

        let text_label = Label::builder()
            .label("hi")
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

        self.tabs.iter().for_each(|tab| res.append(&tab.widget()));

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
