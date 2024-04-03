use gtk::FileFilter;
use gtk::gio::ListStore;

use super::{dialog::choose_file, UiState};

pub fn import(ui_state: &mut UiState) {
    choose_file(&ui_state.window, "Choose an image to import", "Import", &ListStore::new::<FileFilter>(), |res| {
        println!("got file: {:?}", res)
    })
}
