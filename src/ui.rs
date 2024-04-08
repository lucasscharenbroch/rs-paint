mod canvas;
mod toolbar;
mod mode;
mod selection;
mod menu;
mod dialog;
mod io;

use canvas::Canvas;
use toolbar::Toolbar;
use dialog::run_about_dialog;
use crate::image::{Image, UnifiedImage};

use gtk::prelude::*;
use gtk::gdk::{Key, ModifierType};
use gtk::{Application, ApplicationWindow, EventControllerKey, Grid, Separator};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;
use gtk::glib::signal::Propagation;

pub struct UiState {
    tabs: Vec<Rc<RefCell<Canvas>>>,
    active_tab: Option<Rc<RefCell<Canvas>>>,
    toolbar_p: Rc<RefCell<Toolbar>>,
    grid: Grid,
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

    fn set_tab(&mut self, target_idx: usize) {
        if let Some(canvas_p) = self.tabs.get(target_idx) {
            if let Some(current_canvas_p) = self.active_tab() {
                self.grid.remove(current_canvas_p.borrow().widget());
            }

            self.grid.attach(canvas_p.borrow().widget(), 0, 2, 1, 1);
            self.active_tab = Some(canvas_p.clone());
        }
    }

    fn active_tab(&self) -> &Option<Rc<RefCell<Canvas>>> {
        &self.active_tab
    }

    fn new_tab(ui_p: &Rc<RefCell<UiState>>, image: Image) -> usize {
        let canvas_p = Canvas::new_p(&ui_p, UnifiedImage::from_image(image));
        ui_p.borrow_mut().tabs.push(canvas_p);
        ui_p.borrow().tabs.len() - 1
    }

    fn new() -> Rc<RefCell<UiState>> {
        let state = Rc::new(RefCell::new(UiState {
            toolbar_p: Toolbar::new_p(),
            tabs: vec![],
            active_tab: None,
            grid: Grid::new(),
            window: ApplicationWindow::builder()
                .show_menubar(true)
                .title("RS-Paint")
                .build(),
        }));

        Toolbar::init_ui_hooks(&state);

        state.borrow().grid.attach(state.borrow().toolbar_p.borrow().widget(), 0, 0, 1, 1);
        state.borrow().grid.attach(&Separator::new(gtk::Orientation::Horizontal), 0, 1, 1, 1);

        state.borrow().window.set_child(Some(&state.borrow().grid));

        Self::init_internal_connections(&state);

        state
    }

    fn init_internal_connections(state: &Rc<RefCell<Self>>) {
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
    }

    // hack a mod-key-update handler:
    // (.connect_modifier reports the updated mod keys one event late)
    // this is called by handle_keypress and handle_keyrelease
    fn handle_mod_keys_update(&mut self, mod_keys: ModifierType) {
        if let Some(canvas_p) = self.active_tab() {
            self.toolbar_p.borrow_mut().mouse_mode().handle_mod_key_update(&mod_keys, &mut canvas_p.borrow_mut());
        }
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
                    if let Some(canvas_p) = self.active_tab() {
                        canvas_p.borrow_mut().inc_zoom(ZOOM_INC);
                        canvas_p.borrow_mut().update();
                    }
                },
                Key::minus => {
                    if let Some(canvas_p) = self.active_tab() {
                        canvas_p.borrow_mut().inc_zoom(-ZOOM_INC);
                        canvas_p.borrow_mut().update();
                    }
                },
                Key::z => {
                    if let Some(canvas_p) = self.active_tab() {
                        canvas_p.borrow_mut().undo();
                        canvas_p.borrow_mut().update();
                    }
                },
                Key::y => {
                    if let Some(canvas_p) = self.active_tab() {
                        canvas_p.borrow_mut().redo();
                        canvas_p.borrow_mut().update();
                    }
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

    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }
}
