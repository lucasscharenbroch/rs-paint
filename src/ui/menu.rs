use crate::image::undo::action::{StaticUndoableAction, UndoableAction};
use crate::image::transform::*;

use super::dialog::about_dialog;
use super::io::{export, import, new};
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
    let file_menu = MenuBuilder::new()
        .item("New", "new", Box::new(clone!(@strong ui_state => move || new(ui_state.clone()))))
        .item("Import", "import", Box::new(clone!(@strong ui_state => move || import(ui_state.clone()))))
        .item("Export", "export", Box::new(clone!(@strong ui_state => move || export(ui_state.clone()))));

    // image menu helpers

    let mk_do_uaction = clone!(@strong ui_state => move |uaction: Box<dyn StaticUndoableAction>| {
        clone!(@strong ui_state => move || {
            if let Some(canvas_p) = ui_state.borrow().active_canvas_p() {
                canvas_p.borrow_mut().exec_undoable_action(uaction.dyn_clone());
            }
        })
    });

    let flip_horiz_fn = Box::new(mk_do_uaction(Box::new(Flip::Horizontal)));
    let flip_vert_fn = Box::new(mk_do_uaction(Box::new(Flip::Vertical)));

    let image_menu = MenuBuilder::new()
        .submenu("Flip",
            MenuBuilder::new()
            .item("Flip Horizontally", "flip-horiz", flip_horiz_fn)
            .item("Flip Vertically", "flip-vert", flip_vert_fn));


    let help_menu = MenuBuilder::new()
        .item("About", "about",
                Box::new(clone!(@strong ui_state => move || about_dialog(&ui_state.borrow().window))));

    MenuBuilder::new()
        .submenu("File", file_menu)
        .submenu("Image", image_menu)
        .submenu("Help", help_menu)
        .build()
}
