mod canvas;
mod toolbar;
mod mode;

use canvas::Canvas;
use toolbar::Toolbar;
use super::image::{mk_test_image};

use gtk::prelude::*;
use gtk::gdk::{Key, ModifierType};
use gtk::{Application, ApplicationWindow, EventControllerKey, Grid, Separator, GestureDrag};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;
use gtk::glib::signal::Propagation;

pub struct UiState {
    canvas_p: Rc<RefCell<Canvas>>,
    toolbar_p: Rc<RefCell<Toolbar>>,
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
            canvas_p: Canvas::new_p(mk_test_image()),
            toolbar_p: Toolbar::new_p(),
            window: ApplicationWindow::builder()
                .title("RS-Paint")
                .build(),
        }));

        let grid = Grid::new();
        grid.attach(state.borrow().toolbar_p.borrow().widget(), 0, 0, 1, 1);
        grid.attach(&Separator::new(gtk::Orientation::Horizontal), 0, 1, 1, 1);
        grid.attach(state.borrow().canvas_p.borrow().widget(), 0, 2, 1, 1);

        state.borrow().window.set_child(Some(&grid));

        // keypresses

        let key_controller = EventControllerKey::new();

        key_controller.connect_key_pressed(clone!(@strong state => move |_, key, _, modifier| {
            state.borrow_mut().handle_keypress(key, modifier);
            Propagation::Proceed
        }));

        state.borrow().window.add_controller(key_controller);

        // drag

        let drag_controller = GestureDrag::new();
        drag_controller.connect_begin(clone!(@strong state => move |_, _| {
            let state = state.borrow();
            state.toolbar_p.borrow().mouse_mode().handle_drag_start(&mut state.canvas_p.borrow_mut());
        }));

        drag_controller.connect_drag_update(clone!(@strong state => move |_, _, _| {
            let state = state.borrow();
            state.toolbar_p.borrow().mouse_mode().handle_drag_update(&mut state.canvas_p.borrow_mut());
        }));

        drag_controller.connect_drag_end(clone!(@strong state => move |_, _, _| {
            let state = state.borrow();
            state.toolbar_p.borrow().mouse_mode().handle_drag_end(&mut state.canvas_p.borrow_mut());
        }));

        state.borrow().canvas_p.borrow().drawing_area().add_controller(drag_controller);

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
