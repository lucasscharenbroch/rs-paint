use gtk::prelude::*;
use gtk::glib::object::IsA;

pub fn about_dialog(parent: &impl IsA<gtk::Window>) {
    let icon_texture = &*crate::ui::icon::LOGO;

    let dialog = gtk::AboutDialog::builder()
        .program_name(crate::PROGRAM_NAME)
        .comments(crate::PROGRAM_DESCRIPTION)
        .website_label("Github")
        .website("https://github.com/lucasscharenbroch/rs-paint")
        .authors(vec!["Lucas Scharenbroch"])
        .version(crate::SEMANTIC_VERSION)
        .deletable(true)
        .transient_for(parent)
        .logo(icon_texture)
        .build();

    dialog.present();
}

pub fn keyboard_shortcuts_dialog(parent: &impl IsA<gtk::Window>) {
    fn shortcut_from_specs((name, keys): &(&str, &str)) -> gtk::ShortcutsShortcut {
        gtk::ShortcutsShortcut::builder()
            .title(*name)
            .shortcut_type(gtk::ShortcutType::Accelerator)
            .accelerator(*keys)
            .build()
    }

    fn group_from_specs(title: &str, specs: &[(&str, &str)]) -> gtk::ShortcutsGroup {
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
        ("Lock Aspect Ratio", "<Shift>"),
    ]);

    let magic_wand_mode = group_from_specs("Magic Wand Mode", &[
        ("Clear Selection", "Pointer_Right"),
    ]);

    let eyedropper_mode = group_from_specs("Eyedropper Mode", &[
        ("Eyedrop into Palette", "<Ctrl>Pointer_Left"),
    ]);

    let free_transform_mode = group_from_specs("Free Transform Mode", &[
        ("Lock Aspect Ratio", "<Shift>"),
    ]);

    let all_modes = group_from_specs("All Modes", &[
        ("Select All", "<Ctrl>a"),
        ("Copy Selection", "<Ctrl>c"),
        ("Cut Selection", "<Ctrl>x"),
        ("Paste as Transformation", "<Ctrl>p"),
        ("Paste as Tab", "<Ctrl><Shift>p"),
    ]);

    let palette = group_from_specs("Palette", &[
        ("Change Palette Color", "Pointer_Right"),
        ("Remove Palette Color", "<Ctrl>Pointer_Right"),
    ]);

    let undo = group_from_specs("Undo", &[
        ("Undo", "<Ctrl>z"),
        ("Redo", "<Ctrl>y"),
        ("Undo History", "<Ctrl>h"),
    ]);

    let misc = group_from_specs("Miscellaneous", &[
        ("About RS-Paint", "<Ctrl><Shift>a"),
        ("Quit", "<Ctrl>q"),
    ]);

    let io = group_from_specs("I/O", &[
        ("New Image", "<Ctrl>n"),
        ("Import Image", "<Ctrl>i"),
        ("Export Image", "<Ctrl>e"),
        ("Import onto Canvas", "<Ctrl>o"),
        ("Load Project File", "<Ctrl><Shift>l"),
        ("Save Project As", "<Ctrl><Shift>s"),
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
    main_section.add_group(&free_transform_mode);
    main_section.add_group(&all_modes);
    main_section.add_group(&palette);
    main_section.add_group(&misc);
    main_section.add_group(&io);

    let dialog = gtk::ShortcutsWindow::builder()
        .transient_for(parent)
        .build();

    dialog.add_section(&main_section);

    dialog.present();
}