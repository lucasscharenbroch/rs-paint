use crate::image::undo::action::StaticMultiUndoableAction;
use crate::image::transform::*;

use super::dialog::{about_dialog, keyboard_shortcuts_dialog};
use super::UiState;

use gtk::gio;
use gtk::glib::Variant;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

struct MenuBuilder {
    menu: gio::Menu,
    actions: Vec<gio::SimpleAction>,
}

impl MenuBuilder {
    fn new() -> Self {
        MenuBuilder {
            menu: gio::Menu::new(),
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
        let action = gio::SimpleAction::new(action_name, None);
        action.connect_activate(move |_, _| action_fn());
        self.actions.push(action);
        self
    }

    fn item_with_keybind(mut self, keybind: &str, label: &str, action_name: &str, action_fn: Box<dyn Fn()>) -> MenuBuilder {
        let menu_item = gio::MenuItem::new(Some(label), Some(("app.".to_string() + action_name).as_str()));
        menu_item.set_attribute_value("accel", Some(&Variant::from(keybind)));
        self.menu.append_item(&menu_item);
        let action = gio::SimpleAction::new(action_name, None);
        action.connect_activate(move |_, _| action_fn());
        self.actions.push(action);
        self
    }

    fn build(self) -> (gio::Menu, Vec<gio::SimpleAction>) {
        (self.menu, self.actions)
    }
}

pub fn mk_menu(ui_state: Rc<RefCell<UiState>>) -> (gio::Menu, Vec<gio::SimpleAction>) {
    let file_menu = MenuBuilder::new()
        .item_with_keybind("<Ctrl>n", "New", "new", Box::new(clone!(@strong ui_state => move || UiState::new(ui_state.clone()))))
        .item_with_keybind("<Ctrl>i", "Import", "import", Box::new(clone!(@strong ui_state => move || UiState::import(ui_state.clone()))))
        .item_with_keybind("<Ctrl>e", "Export", "export", Box::new(clone!(@strong ui_state => move || UiState::export(ui_state.clone()))))
        .item_with_keybind("<Ctrl><Shift>i", "Import Project", "import-project", Box::new(clone!(@strong ui_state => move || UiState::import_project(ui_state.clone()))))
        .item_with_keybind("<Ctrl><Shift>s", "Save Project As", "save-project-as", Box::new(clone!(@strong ui_state => move || UiState::save_project_as(ui_state.clone()))))
        .item_with_keybind("<Ctrl>q", "Quit", "quit", Box::new(clone!(@strong ui_state => move || UiState::quit(ui_state.clone()))));

    let edit_menu = MenuBuilder::new()
        .item_with_keybind("<Ctrl>z", "Undo", "undop", Box::new(clone!(@strong ui_state => move || UiState::undo(ui_state.clone()))))
        .item_with_keybind("<Ctrl>y", "Redo", "redo", Box::new(clone!(@strong ui_state => move || UiState::redo(ui_state.clone()))))
        .item_with_keybind("<Ctrl>h", "History", "history", Box::new(clone!(@strong ui_state => move || UiState::undo_history_dialog(ui_state.clone()))))
        .item_with_keybind("<Ctrl>l", "Layers", "layers", Box::new(clone!(@strong ui_state => move || UiState::layers_dialog(ui_state.clone()))));

    // image menu helpers

    let mk_do_uaction = clone!(@strong ui_state => move |uaction: Box<dyn StaticMultiUndoableAction<_, LayerData = _>>| {
        clone!(@strong ui_state => move || {
            if let Some(canvas_p) = ui_state.borrow().active_canvas_p() {
                canvas_p.borrow_mut().exec_multi_undoable_action(uaction.dyn_clone());
            }
        })
    });

    let scale_fn = Box::new(clone!(@strong ui_state => move || UiState::scale(ui_state.clone())));
    let expand_fn = Box::new(clone!(@strong ui_state => move || UiState::expand(ui_state.clone())));
    let crop_fn = Box::new(clone!(@strong ui_state => move || UiState::crop_to_selection(ui_state.clone())));
    let truncate_fn = Box::new(clone!(@strong ui_state => move || UiState::truncate(ui_state.clone())));
    let flip_horiz_fn = Box::new(mk_do_uaction(Box::new(Flip::Horizontal)));
    let flip_vert_fn = Box::new(mk_do_uaction(Box::new(Flip::Vertical)));
    let flip_transpose_fn = Box::new(mk_do_uaction(Box::new(Flip::Transpose)));
    let rotate_clockwise_fn = Box::new(mk_do_uaction(Box::new(Rotate::Clockwise)));
    let rotate_counter_clockwise_fn = Box::new(mk_do_uaction(Box::new(Rotate::CounterClockwise)));
    let rotate_180_fn = Box::new(mk_do_uaction(Box::new(Rotate::OneEighty)));

    let image_menu = MenuBuilder::new()
        .item("Scale", "scale", scale_fn)
        .item("Expand", "expand", expand_fn)
        .item("Crop To Selection", "crop-to-selection", crop_fn)
        .item("Truncate", "truncate", truncate_fn)
        .submenu("Flip",
            MenuBuilder::new()
            .item("Horizontally", "flip-horiz", flip_horiz_fn)
            .item("Vertically", "flip-vert", flip_vert_fn)
            .item("Transpose", "flip-transpose", flip_transpose_fn))
        .submenu("Rotate",
            MenuBuilder::new()
            .item("90\u{00B0} Clockwise", "rotate-90-clockwise", rotate_clockwise_fn)
            .item("90\u{00B0} Counter-Clockwise", "rotate-90-counter-clockwise", rotate_counter_clockwise_fn)
            .item("180\u{00B0}", "rotate-180", rotate_180_fn));

    let help_menu = MenuBuilder::new()
        .item("Keyboard Shortcuts", "keyboard-shortcuts",
                Box::new(clone!(@strong ui_state => move || keyboard_shortcuts_dialog(&ui_state.borrow().window))))
        .item_with_keybind("<Ctrl>a", "About", "about",
                Box::new(clone!(@strong ui_state => move || about_dialog(&ui_state.borrow().window))));

    MenuBuilder::new()
        .submenu("File", file_menu)
        .submenu("Edit", edit_menu)
        .submenu("Image", image_menu)
        .submenu("Help", help_menu)
        .build()
}
