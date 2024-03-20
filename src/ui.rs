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
}

impl UiState {
    pub fn run_main_ui() -> gtk::glib::ExitCode {
        let app = Application::builder().build();

        app.connect_activate(move |app| Self::build_ui(app));
        app.run()
    }

    fn new() -> UiState {
        UiState {
            canvas_p: Canvas::new(mk_test_image()),
        }
    }

    fn handle_keypress(&mut self, key: Key, modifier: ModifierType) {
        const ZOOM_INC: f64 = 1.0;

        if modifier == ModifierType::CONTROL_MASK {
            if key == Key::equal {
                self.canvas_p.borrow_mut().inc_zoom(ZOOM_INC);
                self.canvas_p.borrow_mut().queue_redraw();
            } else if(key == Key::minus) {
                self.canvas_p.borrow_mut().inc_zoom(-ZOOM_INC);
                self.canvas_p.borrow_mut().queue_redraw();
            }
        }
    }

    fn build_ui(app: &Application) {
        let state = Rc::new(RefCell::new(Self::new()));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("RS-Paint")
            .child(state.borrow().canvas_p.borrow().widget())
            .build();

        let key_controller = EventControllerKey::new();

        key_controller.connect_key_pressed(clone!(@strong state => move |_, key, _, modifier| {
            state.borrow_mut().handle_keypress(key, modifier);
            Propagation::Proceed
        }));

        window.add_controller(key_controller);

        window.present();
    }
}
