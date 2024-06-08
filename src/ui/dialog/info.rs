use gtk::{prelude::*, ShortcutsGroup, Window};
use gtk::glib::object::IsA;

pub fn about_dialog(parent: &impl IsA<Window>) {
    let dialog = gtk::AboutDialog::builder()
        .program_name("RS-Paint")
        .comments("A lightweight image editor, written in Rust using GTK4.")
        .website_label("Github")
        .website("https://github.com/lucasscharenbroch/rs-paint")
        .authors(vec!["Lucas Scharenbroch"])
        .version("1.0")
        .deletable(true)
        .transient_for(parent)
        .build();

    dialog.present();
}

pub fn keyboard_shortcuts_dialog(parent: &impl IsA<Window>) {
    fn shortcut_from_specs((name, keys): &(&str, &str)) -> gtk::ShortcutsShortcut {
        gtk::ShortcutsShortcut::builder()
            .title(*name)
            .shortcut_type(gtk::ShortcutType::Accelerator)
            .accelerator(*keys)
            .build()
    }

    fn group_from_specs(title: &str, specs: &[(&str, &str)]) -> ShortcutsGroup {
        let res = gtk::ShortcutsGroup::builder()
            .title(title)
            .build();

        specs.iter().for_each(|specs| res.add_shortcut(&shortcut_from_specs(specs)));

        res
    }

    let zoom = group_from_specs("Zoom", &[
        ("Zoom In", "<Ctrl>equal"),
        ("Zoom Out", "<Ctrl>minus"),
    ]);

    let undo = group_from_specs("Undo", &[
        ("Undo", "<Ctrl>z"),
        ("Redo", "<Ctrl>y"),
        ("Undo History", "<Ctrl>h"),
    ]);

    let misc = group_from_specs("Miscellaneous", &[
        ("About RS-Paint", "<Ctrl>a"),
        ("Quit", "<Ctrl>q"),
    ]);

    let io = group_from_specs("I/O", &[
        ("New Image", "<Ctrl>n"),
        ("Import Image", "<Ctrl>i"),
        ("Export Image", "<Ctrl>e"),
    ]);


    let main_section = gtk::ShortcutsSection::builder()
        .build();

    main_section.add_group(&zoom);
    main_section.add_group(&undo);
    main_section.add_group(&misc);
    main_section.add_group(&io);

    let dialog = gtk::ShortcutsWindow::builder()
        .transient_for(parent)
        .build();

    dialog.add_section(&main_section);

    dialog.present();
}