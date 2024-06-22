mod selection;
mod canvas;
mod toolbar;
mod dialog;
mod menu;
mod io;
mod tab;
mod form;

use canvas::Canvas;
use toolbar::Toolbar;
use dialog::{about_dialog, cancel_discard_dialog_str, expand_dialog, close_dialog, ok_dialog_str_, scale_dialog, CloseDialog};
use crate::image::{Image, UnifiedImage, generate::{NewImageProps, generate}};
use tab::{Tab, Tabbar};
use toolbar::mode::{MouseMode, RectangleSelectState};

use gtk::{gdk::RGBA, prelude::*};
use gtk::gdk::{Key, ModifierType};
use gtk::{Application, ApplicationWindow, EventControllerKey, Grid, Separator, Box as GBox, Window, Widget};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;
use gtk::glib::signal::Propagation;

#[macro_export]
macro_rules! icon_file {
    ($name:expr) => {
        format!("./icons/{}.png", $name)
    };
}

fn get_parent_window(widget: &impl IsA<Widget>) -> Option<Window> {
    let parent = widget.parent()?;

    if let Ok(window) = parent.clone().downcast::<gtk::Window>() {
        Some(window)
    } else {
        get_parent_window(&parent)
    }
}

pub struct UiState {
    tabbar: Tabbar,
    tabbar_widget: Option<GBox>,
    toolbar_p: Rc<RefCell<Toolbar>>,
    grid: Grid,
    window: ApplicationWindow,
    application: Application,
}

impl UiState {
    pub fn run_main_ui() -> gtk::glib::ExitCode {
        let app = Application::builder()
            .build();
        let ui_p = Self::new_p(app.clone());
        Self::setup_default_image(&ui_p);

        app.connect_activate(clone!(@strong ui_p => move |app| {
            ui_p.borrow().window.set_application(Some(app));
            ui_p.borrow().window.present();
        }));

        let (menu, menu_actions) = menu::mk_menu(ui_p.clone());

        let _ = app.register(None::<&gtk::gio::Cancellable>);
        app.set_menubar(Some(&menu));
        menu_actions.iter().for_each(|a| app.add_action(a));

        app.run()
    }

    fn update_tabbar_widget(ui_p: &Rc<RefCell<Self>>) {
        if let Some(ref w) = ui_p.borrow().tabbar_widget {
            ui_p.borrow().grid.remove(w);
        }

        let new_widget = ui_p.borrow_mut().tabbar.widget(ui_p);
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

    fn try_close_tab_then<F>(ui_p: &Rc<RefCell<Self>>, target_idx: usize, f: Rc<F>)
    where
        F: Fn(Result<(), ()>) + 'static
    {
        let close_it = clone!(@strong ui_p, @strong f => move || {
            ui_p.borrow_mut().close_tab(target_idx);
            UiState::update_tabbar_widget(&ui_p);
            f(Ok(()));
            dialog::CloseDialog::Yes
        });

        if let Some(target_tab) = ui_p.borrow().tabbar.tabs.get(target_idx) {
            if target_tab.modified_since_export() {
                cancel_discard_dialog_str(
                    ui_p.borrow().window(),
                    "Close tab",
                    format!("`{}` has been modified since last exporting. Discard unsaved changes?", target_tab.name()).as_str(),
                    move || {
                        f(Err(()));
                        dialog::CloseDialog::Yes
                    },
                    close_it,
                );
                return;
             }
             // fall through (let go of borrow in ui_p)
        } else {
            panic!("Trying to close non-existant tab index: {target_idx}");
        }

        close_it();
    }

    fn try_close_tab(ui_p: &Rc<RefCell<Self>>, target_idx: usize) {
        Self::try_close_tab_then(ui_p, target_idx, Rc::new(|_| ()));
    }

    fn close_tab(&mut self, target_idx: usize) {
        if let Some(target_tab) = self.tabbar.tabs.get(target_idx) {
            if self.tabbar.tabs.len() == 1 {
                self.grid.remove(target_tab.canvas_p.borrow().widget());
                self.tabbar.active_idx = None;
            } else if self.tabbar.active_idx.map(|i| i == target_idx).unwrap_or(false) {
                // removing the active tab: switch one to the left, unless it's 0
                if target_idx == 0 {
                    self.set_tab(1);
                    self.tabbar.active_idx = Some(0);
                } else {
                    self.set_tab(target_idx - 1);
                }
            } else {
                // active tab is not being removed: just adjust active_idx
                self.tabbar.active_idx = self.tabbar.active_idx.and_then(|i| {
                    if i < target_idx {
                        Some(i)
                    } else {
                        Some(i - 1)
                    }
                });
            }

            self.tabbar.tabs.remove(target_idx);
        }
    }

    fn active_tab(&self) -> Option<&Tab> {
        self.tabbar.active_idx.and_then(|i| self.tabbar.tabs.get(i))
    }

    fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabbar.active_idx.and_then(|i| self.tabbar.tabs.get_mut(i))
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

    fn new_p(application: Application) -> Rc<RefCell<UiState>> {
        let ui_p = Rc::new(RefCell::new(UiState {
            toolbar_p: Toolbar::new_p(),
            tabbar: Tabbar::new(),
            tabbar_widget: None,
            grid: Grid::new(),
            window: ApplicationWindow::builder()
                .show_menubar(true)
                .title("RS-Paint")
                .build(),
            application,
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
            Self::handle_keypress(&ui_p, key, mod_keys);
            Propagation::Proceed
        }));

        key_controller.connect_key_released(clone!(@strong ui_p => move |_, key, _, mod_keys| {
            ui_p.borrow_mut().handle_keyrelease(key, mod_keys);
        }));

        ui_p.borrow().window.add_controller(key_controller);

        ui_p.borrow().window.connect_close_request(clone!(@strong ui_p => move |app| {
            // don't close: call quit instead
            Self::quit(ui_p.clone());
            gtk::glib::signal::Propagation::Stop
        }));
    }

    // hack a mod-key-update handler:
    // (.connect_modifier reports the updated mod keys one event late)
    // this is called by handle_keypress and handle_keyrelease
    fn handle_mod_keys_update(&mut self, mod_keys: ModifierType) {
        if let Some(canvas_p) = self.active_canvas_p() {
            let mut toolbar = self.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_mod_key_update(&mod_keys, &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
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

    fn handle_keypress(ui_p: &Rc<RefCell<Self>>, key: Key, mod_keys: ModifierType) {
        // control-key bindings
        if mod_keys == ModifierType::CONTROL_MASK {
            match key {
                Key::equal => Self::zoom_in(ui_p.clone()),
                Key::minus => Self::zoom_out(ui_p.clone()),
                Key::z => Self::undo(ui_p.clone()),
                Key::y => Self::redo(ui_p.clone()),
                Key::h => Self::undo_history(ui_p.clone()),
                Key::a => about_dialog(&ui_p.borrow().window),
                Key::n => Self::new(ui_p.clone()),
                Key::i => Self::import(ui_p.clone()),
                Key::e => Self::export(ui_p.clone()),
                Key::q => Self::quit(ui_p.clone()),
                // Remember to add any new shortcuts to `dialog::info::keyboard_shortcuts_dialog`
                _ => (),
            }
        }

        if let Key::Delete = key {
            Self::delete_selection(ui_p.clone());
        }

        if let Some(mod_keys) = Self::try_update_mod_keys(key, mod_keys, true) {
            ui_p.borrow_mut().handle_mod_keys_update(mod_keys);
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

    pub fn notify_tab_successful_export(&mut self) {
        if let Some(tab) = self.active_tab_mut() {
            tab.notify_successful_export();
        }
    }

    fn setup_default_image(ui_p: &Rc<RefCell<Self>>) {
        const DEFAULT_IMAGE_PROPS: NewImageProps = NewImageProps {
            height: 512,
            width: 512,
            color: RGBA::new(0.0, 0.0, 0.0, 0.0),
        };

        let image = generate(DEFAULT_IMAGE_PROPS);
        UiState::new_tab(ui_p, image, "[untitled]");
    }

    pub fn history_popup(&self) {
        if let Some(canvas_p) = self.active_canvas_p() {
            let canvas = canvas_p.borrow();
            let history_widget = canvas.history_widget();

            close_dialog(self.window(), "Image History", history_widget, || CloseDialog::Yes);
        }
    }

    fn try_close_all_tabs_then<F>(ui_p: &Rc<RefCell<Self>>, f: F)
    where
        F: Fn(Result<(), ()>) + 'static
    {
        if ui_p.borrow().active_tab().is_some() {
            Self::try_close_tab_then(&ui_p.clone(), 0, Rc::new(clone!(@strong ui_p => move |tab_close_success| {
                if let Ok(()) = tab_close_success {
                    Self::quit(ui_p.clone());
                }
            })));
            f(Err(()));
        } else {
            f(Ok(()));
        }
    }

    fn try_clean_up_before_quit_then<F>(ui_p: &Rc<RefCell<Self>>, f: F)
    where
        F: Fn(Result<(), ()>) + 'static
    {
        // just a wrapper for now; maybe do other checks

        // close all tabs as normal, to raise any "unsaved-changes" dialogs
        Self::try_close_all_tabs_then(&ui_p.clone(), f);
    }

    pub fn quit(ui_p: Rc<RefCell<Self>>) {
        Self::try_clean_up_before_quit_then(&ui_p.clone(), move |ok_to_quit| {
            if let Ok(()) = ok_to_quit {
                ui_p.borrow().application.quit();
            }
        })
    }

    pub fn zoom_in(ui_p: Rc<RefCell<Self>>) {
        const ZOOM_INC: f64 = 1.0;

        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().inc_zoom(ZOOM_INC);
            canvas_p.borrow_mut().update();
        }
    }

    pub fn zoom_out(ui_p: Rc<RefCell<Self>>) {
        const ZOOM_INC: f64 = 1.0;

        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().inc_zoom(-ZOOM_INC);
            canvas_p.borrow_mut().update();
        }
    }

    pub fn undo(ui_p: Rc<RefCell<Self>>) {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().undo();
            canvas_p.borrow_mut().update();
        }
    }

    pub fn redo(ui_p: Rc<RefCell<Self>>) {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().redo();
            canvas_p.borrow_mut().update();
        }
    }

    pub fn undo_history(ui_p: Rc<RefCell<Self>>) {
        ui_p.borrow().history_popup();
    }

    fn crop_to_selection(ui_p: Rc<RefCell<UiState>>) {
        let ui = ui_p.borrow();
        let mut toolbar = ui.toolbar_p.borrow_mut();

        if let MouseMode::RectangleSelect(state) = toolbar.mouse_mode() {
            if let RectangleSelectState::Selected(x, y, w, h) = state {
                let canvas_p = ui.active_canvas_p().unwrap();
                canvas_p.borrow_mut().crop_to(*x as usize, *y as usize, *w as usize, *h as usize);
                toolbar.set_mouse_mode(MouseMode::RectangleSelect(RectangleSelectState::Unselected));
                canvas_p.borrow_mut().set_selection(selection::Selection::NoSelection);
                return;
            }
        }

        ok_dialog_str_(
            ui.window(),
            "Make a Selection First",
            "Use the rectangle select tool to select a region to crop."
        );
    }

    pub fn delete_selection(ui_p: Rc<RefCell<Self>>) {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            canvas_p.borrow_mut().delete_selection();
        }
    }

    fn scale(ui_p: Rc<RefCell<Self>>) {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            let w = canvas_p.borrow().image_width() as usize;
            let h = canvas_p.borrow().image_height() as usize;
            scale_dialog(&ui_p.borrow().window, w, h, clone!(@strong ui_p => move |action| {
                if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
                    canvas_p.borrow_mut().exec_doable_action(&action);
                }
            }));
        }
    }

    fn expand(ui_p: Rc<RefCell<Self>>) {
        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
            expand_dialog(&ui_p.borrow().window, clone!(@strong ui_p => move |action| {
                if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
                    canvas_p.borrow_mut().exec_undoable_action(Box::new(action));
                }
            }));
        }
    }
}
