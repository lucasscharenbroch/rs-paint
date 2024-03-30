use gtk::{gio::Menu};

pub fn mk_menu() -> Menu {
    let res = Menu::new();

    let file = Menu::new();
    file.append(Some("New"), None);
    file.append(Some("Import"), None);
    file.append(Some("Export"), None);

    let help = Menu::new();
    help.append(Some("About"), None);

    res.append_submenu(Some("File"), &file);
    res.append_submenu(Some("Help"), &help);

    res
}