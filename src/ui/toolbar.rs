mod pallete;
mod mode;

use mode::MouseMode;
use super::canvas::Canvas;
use super::UiState;

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

const INITIAL_MODE: MouseMode = MouseMode::cursor_default();

impl Toolbar {
    pub fn new_p() -> Rc<RefCell<Toolbar>> {
        let tbox =  Box::new(Orientation::Horizontal, 10);

        let toolbar_p = Rc::new(RefCell::new(Toolbar {
            tbox,
            mouse_mode: INITIAL_MODE,
            mouse_mode_buttons: vec![],
            mode_change_hook: None,
        }));

        toolbar_p
    }

    pub fn init_ui_hooks(ui_p: &Rc<RefCell<UiState>>) {
        let toolbar_p = ui_p.borrow().toolbar_p.clone();

        let button_info: Vec<(&str, fn(&Canvas) -> MouseMode, fn() -> MouseMode)> = vec![
            ("Cursor", MouseMode::cursor, MouseMode::cursor_default),
            ("Pencil", MouseMode::pencil, MouseMode::pencil_default),
            ("Rectangle Select", MouseMode::rectangle_select, MouseMode::rectangle_select_default),
        ];

        toolbar_p.borrow_mut().mouse_mode_buttons = button_info.into_iter()
            .map(|(text, mode_constructor, mode_constructor_default)| {
                let button = ToggleButton::builder()
                    .label(text)
                    .build();

                button.connect_clicked(clone!(@strong toolbar_p, @strong ui_p => move |b| {
                    if b.is_active() {
                        let mode =
                            if let Some(canvas_p) = ui_p.borrow().active_canvas_p()  {
                                mode_constructor(&canvas_p.borrow())
                            } else {
                                mode_constructor_default()
                            };

                        toolbar_p.borrow_mut().mouse_mode = mode.clone();
                        for other_button in toolbar_p.borrow().mouse_mode_buttons.iter() {
                            if other_button.mode.variant() != mode.variant() {
                                other_button.widget.set_active(false);
                            }
                        }

                        if let Some(ref f) = toolbar_p.borrow().mode_change_hook {
                            f(&toolbar_p.borrow());
                        }
                    } else {
                        // the only way to deactivate is to activate a different modal button
                        b.set_active(true);
                    }
                }));

                toolbar_p.borrow_mut().tbox.append(&button);

                MouseModeButton {
                    widget: button,
                    mode: mode_constructor_default(),
                }
            })
            .collect::<Vec<_>>();

        // activate INITIAL_MODE button
        toolbar_p.borrow_mut().mouse_mode_buttons.iter().for_each(|b| {
            if b.mode.variant() == INITIAL_MODE.variant() {
                b.widget.set_active(true);
            }
        });
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
