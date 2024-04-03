use super::mode::MouseMode;
use super::canvas::Canvas;

use gtk::prelude::*;
use gtk::{Box, Orientation, ToggleButton};
use std::rc::Rc;
use std::cell::RefCell;
use std::boxed;
use glib_macros::clone;

pub struct Toolbar {
    tbox: Box,
    mouse_mode: MouseMode,
    mouse_mode_buttons: Vec<MouseModeButton>,
    mode_change_hook: Option<boxed::Box<dyn Fn(&Toolbar)>>,
}

struct MouseModeButton {
    mode: MouseMode,
    widget: ToggleButton,
}

impl Toolbar {
    pub fn new_p(canvas_p: Rc<RefCell<Canvas>>) -> Rc<RefCell<Toolbar>> {
        let tbox =  Box::new(Orientation::Horizontal, 10);
        let initial_mode = MouseMode::cursor(&canvas_p.borrow());

        let state = Rc::new(RefCell::new(Toolbar {
            tbox,
            mouse_mode: initial_mode,
            mouse_mode_buttons: vec![],
            mode_change_hook: None,
        }));

        let button_info: Vec<(&str, fn(&Canvas) -> MouseMode)> = vec![
            ("Cursor", MouseMode::cursor),
            ("Pencil", MouseMode::pencil),
            ("Rectangle Select", MouseMode::rectangle_select),
        ];

        state.borrow_mut().mouse_mode_buttons = button_info.into_iter()
            .map(|(text, mode_constructor)| {
                let button = ToggleButton::builder()
                    .label(text)
                    .build();

                button.connect_clicked(clone!(@strong state, @strong canvas_p => move |b| {
                    if b.is_active() {
                        let mode = mode_constructor(&canvas_p.borrow());
                        state.borrow_mut().mouse_mode = mode.clone();
                        for other_button in state.borrow().mouse_mode_buttons.iter() {
                            if other_button.mode != mode {
                                other_button.widget.set_active(false);
                            }
                        }

                        if let Some(ref f) = state.borrow().mode_change_hook {
                            f(&state.borrow());
                        }
                    } else {
                        // the only way to deactivate is to activate a different modal button
                        b.set_active(true);
                    }
                }));

                state.borrow_mut().tbox.append(&button);

                MouseModeButton {
                    widget: button,
                    mode: mode_constructor(&canvas_p.borrow()),
                }
            })
            .collect::<Vec<_>>();

        // activate initial_mode button
        state.borrow_mut().mouse_mode_buttons.iter().for_each(|b| {
            if b.mode == initial_mode {
                b.widget.set_active(true);
            }
        });

        state
    }

    pub fn mouse_mode(&mut self) -> &mut MouseMode {
        &mut self.mouse_mode
    }

    pub fn widget(&self) -> &Box {
        &self.tbox
    }

    pub fn set_mode_change_hook(&mut self, f: boxed::Box<dyn Fn(&Toolbar)>) {
        self.mode_change_hook = Some(f);
    }
}
