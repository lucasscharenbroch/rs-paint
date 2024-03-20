mod canvas;

use canvas::Canvas;
use super::image::{mk_test_image};

use gtk::prelude::*;
use gtk::gdk::{Key, ModifierType};
use gtk::{glib, Application, ApplicationWindow, Button, Frame, EventControllerKey, EventControllerScroll};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;
use gtk::glib::signal::Propagation;

#[derive(Clone)]
pub struct UiState {
    canvas_p: Rc<RefCell<Canvas>>,
    window: ApplicationWindow,
}

impl UiState {
    pub fn run_main_ui() -> gtk::glib::ExitCode {
        let state = Self::new();
        let app = Application::builder().build();

        app.connect_activate(clone!(@strong state => move |app| {
            state.borrow().window.set_application(Some(app));
            state.borrow().window.present();
        }));

        app.run()
    }

    fn new() -> Rc<RefCell<UiState>> {
        let state = Rc::new(RefCell::new(UiState {
            canvas_p: Canvas::new(mk_test_image()),
            window: ApplicationWindow::builder()
                .title("RS-Paint")
                .build(),
        }));

        state.borrow().window.set_child(Some(state.borrow().canvas_p.borrow().widget()));

        let key_controller = EventControllerKey::new();

        key_controller.connect_key_pressed(clone!(@strong state => move |_, key, _, modifier| {
            state.borrow_mut().handle_keypress(key, modifier);
            Propagation::Proceed
        }));

        state.borrow().window.add_controller(key_controller);

        state
    }

    fn handle_keypress(&mut self, key: Key, modifier: ModifierType) {
        const ZOOM_INC: f64 = 1.0;

        if modifier == ModifierType::CONTROL_MASK {
            if key == Key::equal {
                self.canvas_p.borrow_mut().inc_zoom(ZOOM_INC);
                self.canvas_p.borrow_mut().update();
            } else if(key == Key::minus) {
                self.canvas_p.borrow_mut().inc_zoom(-ZOOM_INC);
                self.canvas_p.borrow_mut().update();
            }
        }
    }
}
