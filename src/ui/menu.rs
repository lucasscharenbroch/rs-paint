use super::dialog::run_about_dialog;
use super::io::import;
use super::UiState;

use gtk::gio::{Menu, SimpleAction};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;


struct MenuBuilder {
    menu: Menu,
    actions: Vec<SimpleAction>,
}

impl MenuBuilder {
    fn new() -> Self {
        MenuBuilder {
            menu: Menu::new(),
            actions: vec![],
        }
    }

    fn submenu(mut self, label: &str, other: MenuBuilder) -> MenuBuilder {
        let (other_menu, mut other_actions) = other.build();
        self.menu.append_submenu(Some(label), &other_menu);
        self.actions.append(&mut other_actions);
        self
    }

    fn item(mut self, label: &str, action_name: &str, action_fn: Box<dyn Fn()>) -> MenuBuilder {
        self.menu.append(Some(label), Some(("app.".to_string() + action_name).as_str()));
        let action = SimpleAction::new(action_name, None);
        action.connect_activate(move |_, _| action_fn());
        self.actions.push(action);
        self
    }

    fn build(self) -> (Menu, Vec<SimpleAction>) {
        (self.menu, self.actions)
    }
}

pub fn mk_menu(ui_state: Rc<RefCell<UiState>>) -> (Menu, Vec<SimpleAction>) {
    MenuBuilder::new()
        .submenu("File",
            MenuBuilder::new()
                .item("New", "new", Box::new(|| println!("new")))
                .item("Import", "import", Box::new(clone!(@strong ui_state => move || import(ui_state.clone()))))
                .item("Export", "export", Box::new(|| println!("export"))))
        .submenu("Help",

            MenuBuilder::new()
                .item("About", "about",
                      Box::new(clone!(@strong ui_state => move || run_about_dialog(&ui_state.borrow().window)))))
        .build()
}
