use super::canvas::Canvas;
use super::UiState;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct Tab {
    pub canvas_p: Rc<RefCell<Canvas>>,
}

impl Tab {
    pub fn new(canvas_p: &Rc<RefCell<Canvas>>) -> Self {
        Tab {
            canvas_p: canvas_p.clone(),
        }
    }
}
