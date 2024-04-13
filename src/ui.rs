mod canvas;
mod toolbar;
mod mode;
mod selection;
mod menu;
mod dialog;
mod io;
mod tab;

use canvas::Canvas;
use toolbar::Toolbar;
use dialog::run_about_dialog;
use crate::image::{Image, UnifiedImage};
use tab::{Tab, Tabbar};

use gtk::prelude::*;
use gtk::gdk::{Key, ModifierType};
use gtk::{Application, ApplicationWindow, EventControllerKey, Grid, Separator, Box as GBox};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;
use gtk::glib::signal::Propagation;

pub struct UiState {
    tabbar: Tabbar,
    tabbar_widget: Option<GBox>,
    toolbar_p: Rc<RefCell<Toolbar>>,
    grid: Grid,
    window: ApplicationWindow,
}

impl UiState {
    pub fn run_main_ui() -> gtk::glib::ExitCode {
        let app = Application::builder()
            .build();
        let ui_p = Self::new();

        app.connect_activate(clone!(@strong ui_p => move |app| {
            ui_p.borrow().window.set_application(Some(app));
            ui_p.borrow().window.present();
        }));

        let (menu, menu_actions) = menu::mk_menu(ui_p.clone());

        app.register(None::<&gtk::gio::Cancellable>);
        app.set_menubar(Some(&menu));
        menu_actions.iter().for_each(|a| app.add_action(a));

        app.run()
    }

    fn update_tabbar_widget(ui_p: &Rc<RefCell<Self>>) {
        if let Some(ref w) = ui_p.borrow().tabbar_widget {
            ui_p.borrow().grid.remove(w);
        }

        let new_widget = ui_p.borrow().tabbar.widget(ui_p);
        ui_p.borrow().grid.attach(&new_widget, 0, 0, 1, 1);
        ui_p.borrow_mut().tabbar_widget = Some(new_widget);
    }

    fn set_tab(&mut self, target_idx: usize) {
        if let Some(target_tab) = self.tabbar.tabs.get(target_idx) {
            let target_canvas_p = &target_tab.canvas_p;
            if let Some(current_canvas_p) = self.active_canvas_p() {
                self.grid.remove(current_canvas_p.borrow().widget());
            }

            self.grid.attach(target_canvas_p.borrow().widget(), 0, 2, 1, 1);
            self.tabbar.active_idx = Some(target_idx);
        }
    }

    fn active_tab(&self) -> Option<&Tab> {
        self.tabbar.active_idx.and_then(|i| self.tabbar.tabs.get(i))
    }

    fn active_canvas_p(&self) -> Option<&Rc<RefCell<Canvas>>> {
        self.active_tab().map(|t| &t.canvas_p)
    }

    fn new_tab(ui_p: &Rc<RefCell<UiState>>, image: Image, name: &str) -> usize {
        let canvas_p = Canvas::new_p(&ui_p, UnifiedImage::from_image(image));
        let new_tab = Tab::new(&canvas_p, name);
        let new_idx = ui_p.borrow().tabbar.tabs.len();
        ui_p.borrow_mut().tabbar.tabs.push(new_tab);
        ui_p.borrow_mut().set_tab(new_idx);
        Self::update_tabbar_widget(ui_p);
        new_idx
    }

    fn new() -> Rc<RefCell<UiState>> {
        let ui_p = Rc::new(RefCell::new(UiState {
            toolbar_p: Toolbar::new_p(),
            tabbar: Tabbar::new(),
            tabbar_widget: None,
            grid: Grid::new(),
            window: ApplicationWindow::builder()
                .show_menubar(true)
                .title("RS-Paint")
                .build(),
        }));

        Toolbar::init_ui_hooks(&ui_p);

        ui_p.borrow().grid.attach(&ui_p.borrow().tabbar.widget(&ui_p), 0, 0, 1, 1);
        ui_p.borrow().grid.attach(ui_p.borrow().toolbar_p.borrow().widget(), 0, 1, 1, 1);
        ui_p.borrow().grid.attach(&Separator::new(gtk::Orientation::Horizontal), 0, 2, 1, 1);

        ui_p.borrow().window.set_child(Some(&ui_p.borrow().grid));

        Self::init_internal_connections(&ui_p);

        ui_p
    }

    fn init_internal_connections(ui_p: &Rc<RefCell<Self>>) {
        // keypresses

        let key_controller = EventControllerKey::new();

        key_controller.connect_key_pressed(clone!(@strong ui_p => move |_, key, _, mod_keys| {
            ui_p.borrow_mut().handle_keypress(key, mod_keys);
            Propagation::Proceed
        }));

        key_controller.connect_key_released(clone!(@strong ui_p => move |_, key, _, mod_keys| {
            ui_p.borrow_mut().handle_keyrelease(key, mod_keys);
        }));

        ui_p.borrow().window.add_controller(key_controller);
    }

    // hack a mod-key-update handler:
    // (.connect_modifier reports the updated mod keys one event late)
    // this is called by handle_keypress and handle_keyrelease
    fn handle_mod_keys_update(&mut self, mod_keys: ModifierType) {
        if let Some(canvas_p) = self.active_canvas_p() {
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
                    if let Some(canvas_p) = self.active_canvas_p() {
                        canvas_p.borrow_mut().inc_zoom(ZOOM_INC);
                        canvas_p.borrow_mut().update();
                    }
                },
                Key::minus => {
                    if let Some(canvas_p) = self.active_canvas_p() {
                        canvas_p.borrow_mut().inc_zoom(-ZOOM_INC);
                        canvas_p.borrow_mut().update();
                    }
                },
                Key::z => {
                    if let Some(canvas_p) = self.active_canvas_p() {
                        canvas_p.borrow_mut().undo();
                        canvas_p.borrow_mut().update();
                    }
                },
                Key::y => {
                    if let Some(canvas_p) = self.active_canvas_p() {
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
