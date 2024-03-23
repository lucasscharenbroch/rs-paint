use gtk::prelude::*;
use gtk::{Box, Orientation, ToggleButton};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

#[derive(Clone, PartialEq)]
pub enum MouseMode {
    Cursor,
    Pencil,
}

pub struct Toolbar {
    tbox: Box,
    mouse_mode: MouseMode,
    mouse_mode_buttons: Vec<MouseModeButton>,
}

struct MouseModeButton {
    mode: MouseMode,
    button: ToggleButton,
}

impl Toolbar {
    pub fn new() -> Rc<RefCell<Toolbar>> {
        let tbox =  Box::new(Orientation::Horizontal, 10);

        let state = Rc::new(RefCell::new(Toolbar {
            tbox,
            mouse_mode: MouseMode::Cursor,
            mouse_mode_buttons: vec![],
        }));

        const button_info: &'static [(&'static str, MouseMode)] = &[
            ("Cursor", MouseMode::Cursor),
            ("Pencil", MouseMode::Pencil),
        ];

        state.borrow_mut().mouse_mode_buttons = button_info.iter()
            .map(|(text, mode)| {
                let button = ToggleButton::builder()
                    .label(*text)
                    .build();

                button.connect_clicked(clone!(@strong state => move |b| {
                    if b.is_active() {
                        state.borrow_mut().mouse_mode = mode.clone();
                        for other_button in state.borrow().mouse_mode_buttons.iter() {
                            if other_button.mode != *mode {
                                other_button.button.set_active(false);
                            }
                        }
                    } else {
                        // the only way to deactivate is to activate a different modal button
                        b.activate();
                    }
                }));

                state.borrow_mut().tbox.append(&button);

                MouseModeButton {
                    button,
                    mode: mode.clone(),
                }
            })
            .collect::<Vec<_>>();

        state
    }

    pub fn mouse_mode(&self) -> &MouseMode {
        &self.mouse_mode
    }

    pub fn widget(&self) -> &Box {
        &self.tbox
    }
}
