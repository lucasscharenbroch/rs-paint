mod canvas;
mod toolbar;
mod mode;
mod selection;
mod menu;
mod dialog;

use canvas::Canvas;
use toolbar::Toolbar;
use super::image::{mk_test_image};
use dialog::run_about_dialog;

use gtk::prelude::*;
use gtk::gdk::{Key, ModifierType};
use gtk::{Application, ApplicationWindow, EventControllerKey, Grid, Separator, GestureDrag, EventControllerMotion};
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
        let app = Application::builder()
            .build();
        let state = Self::new();

        app.connect_activate(clone!(@strong state => move |app| {
            state.borrow().window.set_application(Some(app));
            state.borrow().window.present();
        }));

        let (menu, menu_actions) = menu::mk_menu(state.clone());

        app.register(None::<&gtk::gio::Cancellable>);
        app.set_menubar(Some(&menu));
        menu_actions.iter().for_each(|a| app.add_action(a));


        app.run()
    }

    fn new() -> Rc<RefCell<UiState>> {
        let canvas_p = Canvas::new_p(mk_test_image());

        let state = Rc::new(RefCell::new(UiState {
            toolbar_p: Toolbar::new_p(canvas_p.clone()),
            canvas_p,
            window: ApplicationWindow::builder()
                .show_menubar(true)
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

        key_controller.connect_key_pressed(clone!(@strong state => move |_, key, _, mod_keys| {
            state.borrow_mut().handle_keypress(key, mod_keys);
            Propagation::Proceed
        }));

        key_controller.connect_key_released(clone!(@strong state => move |_, key, _, mod_keys| {
            state.borrow_mut().handle_keyrelease(key, mod_keys);
        }));

        state.borrow().window.add_controller(key_controller);

        // drag

        let drag_controller = GestureDrag::new();
        drag_controller.connect_begin(clone!(@strong state => move |dc, _| {
            let state = state.borrow();
            state.toolbar_p.borrow_mut().mouse_mode().handle_drag_start(&dc.current_event_state(), &mut state.canvas_p.borrow_mut());
        }));

        drag_controller.connect_drag_update(clone!(@strong state => move |dc, _, _| {
            let state = state.borrow();
            state.toolbar_p.borrow_mut().mouse_mode().handle_drag_update(&dc.current_event_state(), &mut state.canvas_p.borrow_mut());
        }));

        drag_controller.connect_drag_end(clone!(@strong state => move |dc, _, _| {
            let state = state.borrow();
            state.toolbar_p.borrow_mut().mouse_mode().handle_drag_end(&dc.current_event_state(), &mut state.canvas_p.borrow_mut());
        }));

        state.borrow().canvas_p.borrow().drawing_area().add_controller(drag_controller);

        // mouse movement

        let motion_controller = EventControllerMotion::new();

        motion_controller.connect_motion(clone!(@strong state => move |ecm, x, y| {
            let mut state = state.borrow_mut();

            state.canvas_p.borrow_mut().update_cursor_pos(x, y);
            state.toolbar_p.borrow_mut().mouse_mode().handle_motion(&ecm.current_event_state(), &mut state.canvas_p.borrow_mut());
        }));

        state.borrow().canvas_p.borrow().drawing_area().add_controller(motion_controller);

        // drawing

        state.borrow().canvas_p.borrow_mut().set_draw_hook(Box::new(clone!(@strong state => move |cr| {
            let state = state.borrow();
            state.toolbar_p.borrow_mut().mouse_mode().draw(&state.canvas_p.borrow(), cr);
        })));

        // mouse-mode-change

        state.borrow_mut().toolbar_p.borrow_mut().set_mode_change_hook(Box::new(clone!(@strong state => move |_toolbar: &Toolbar| {
            state.borrow_mut().canvas_p.borrow_mut().update();
        })));

        state
    }

    // hack a mod-key-update handler:
    // (.connect_modifier reports the updated mod keys one event late)
    // this is called by handle_keypress and handle_keyrelease
    fn handle_mod_keys_update(&mut self, mod_keys: ModifierType) {
        self.toolbar_p.borrow_mut().mouse_mode().handle_mod_key_update(&mod_keys, &mut self.canvas_p.borrow_mut());
    }

    // apply `key` to `mod_keys`, if it's a mod key
    fn try_update_mod_keys(key: Key, mod_keys: ModifierType, is_down: bool) -> Option<ModifierType> {
        let join = |m: ModifierType, b: ModifierType| Some(if is_down {
            m.union(b)
        } else {
            m.difference(b)
        });

        match key {
            Key::Shift_L | Key::Shift_R => join(mod_keys, ModifierType::SHIFT_MASK),
            Key::Control_L | Key::Control_R => join(mod_keys, ModifierType::CONTROL_MASK),
            Key::Alt_L | Key::Alt_R => join(mod_keys, ModifierType::ALT_MASK),
            _ => None,
        }
    }

    fn handle_keypress(&mut self, key: Key, mod_keys: ModifierType) {
        const ZOOM_INC: f64 = 1.0;

        // control-key bindings
        if mod_keys == ModifierType::CONTROL_MASK {
            match key {
                Key::equal => {
                    self.canvas_p.borrow_mut().inc_zoom(ZOOM_INC);
                    self.canvas_p.borrow_mut().update();
                },
                Key::minus => {
                    self.canvas_p.borrow_mut().inc_zoom(-ZOOM_INC);
                    self.canvas_p.borrow_mut().update();
                },
                Key::z => {
                    self.canvas_p.borrow_mut().undo();
                    self.canvas_p.borrow_mut().update();
                },
                Key::y => {
                    self.canvas_p.borrow_mut().redo();
                    self.canvas_p.borrow_mut().update();
                },
                Key::a => {
                    run_about_dialog(&self.window);
                }
                _ => (),
            }
        }

        if let Some(mod_keys) = Self::try_update_mod_keys(key, mod_keys, true) {
            self.handle_mod_keys_update(mod_keys);
        }

    }

    fn handle_keyrelease(&mut self, key: Key, mod_keys: ModifierType) {
        if let Some(mod_keys) = Self::try_update_mod_keys(key, mod_keys, false) {
            self.handle_mod_keys_update(mod_keys);
        }
    }
}
