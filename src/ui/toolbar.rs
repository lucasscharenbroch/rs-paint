use gtk::prelude::*;
use gtk::{Box, Orientation, Button};
use std::rc::Rc;
use std::cell::RefCell;

pub struct Toolbar {
    tbox: Box,
}

impl Toolbar {
    pub fn new() -> Rc<RefCell<Toolbar>> {
        let tbox =  Box::new(Orientation::Horizontal, 10);

        let cursor_button = Button::builder()
            .label("Cursor")
            .build();

        let pencil_button = Button::builder()
            .label("Pencil")
            .build();

        tbox.append(&cursor_button);
        tbox.append(&pencil_button);

        Rc::new(RefCell::new(Toolbar {
            tbox
        }))
    }

    pub fn widget(&self) -> &Box {
        &self.tbox
    }
}
