use gtk::gio::{Menu, SimpleAction};

pub fn mk_menu() -> (Menu, Vec<SimpleAction>) {
    let menu = Menu::new();

    let file = Menu::new();
    file.append(Some("New"), Some("app.new"));
    file.append(Some("Import"), Some("app.import"));
    file.append(Some("Export"), Some("app.export"));

    let help = Menu::new();
    help.append(Some("About"), Some("app.about"));

    menu.append_submenu(Some("File"), &file);
    menu.append_submenu(Some("Help"), &help);

    // actions
    let new_action = SimpleAction::new("new", None);
    new_action.connect_activate(|_, _| println!("new"));

    let import_action = SimpleAction::new("import", None);
    import_action.connect_activate(|_, _| println!("import"));

    let export_action = SimpleAction::new("export", None);
    export_action.connect_activate(|_, _| println!("export"));

    let about_action = SimpleAction::new("about", None);
    about_action.connect_activate(|_, _| println!("about"));

    let actions = vec![new_action, import_action, export_action, about_action];

    (menu, actions)
}
