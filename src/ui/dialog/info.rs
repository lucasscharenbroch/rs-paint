use super::super::super::icon_file;

use gtk::{prelude::*, ShortcutsGroup, Window};
use gtk::glib::object::IsA;

pub fn about_dialog(parent: &impl IsA<Window>) {
    let icon_texture = gtk::gdk::Texture::from_filename(&icon_file!("logo"));

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

    if let Ok(texture) = icon_texture {
        dialog.set_logo(Some(&texture));
    }

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

    let editing = group_from_specs("Editing", &[
        ("Delete Selection", "Delete"),
    ]);

    let draw_mode = group_from_specs("Draw Mode", &[
        ("Draw Straight Line", "<Shift>Pointer_Left"),
    ]);

    let rectangle_select_mode = group_from_specs("Rectangle Select Mode", &[
        ("Crop to Selection", "<Ctrl>Pointer_Left"),
        ("Clear Selection", "Pointer_Right"),
    ]);

    let magic_wand_mode = group_from_specs("Magic Wand Mode", &[
        ("Clear Selection", "Pointer_Right"),
    ]);

    let eyedropper_mode = group_from_specs("Eyedropper Mode", &[
        ("Eyedrop into palette", "<Ctrl>Pointer_Left"),
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
    main_section.add_group(&editing);
    main_section.add_group(&draw_mode);
    main_section.add_group(&rectangle_select_mode);
    main_section.add_group(&magic_wand_mode);
    main_section.add_group(&eyedropper_mode);
    main_section.add_group(&misc);
    main_section.add_group(&io);

    let dialog = gtk::ShortcutsWindow::builder()
        .transient_for(parent)
        .build();

    dialog.add_section(&main_section);

    dialog.present();
}