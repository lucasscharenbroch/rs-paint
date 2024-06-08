use crate::image::generate::NewImageProps;
use crate::ui::form::ColorField;
use super::form::{DropdownField, RadioField};
use super::form::{Form, NaturalField, CheckboxField, gadget::AspectRatioGadget};
use crate::image::scale::{Scale, ScaleMethod};

use gtk::{prelude::*, Align, Box as GBox, Button, FileDialog, Label, Orientation, ShortcutsGroup, ShortcutsShortcut, TextBuffer, Widget, Window};
use gtk::ColorDialog;
use gtk::glib::{object::IsA, error::Error as GError};
use gtk::gio::{File, Cancellable};
use gtk::gdk::RGBA;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

pub enum CloseDialog {
    Yes,
    No,
}

pub fn ok_dialog(parent: &impl IsA<Window>, title: &str, inner_content: &impl IsA<Widget>) {
    let ok_button = Button::builder()
        .label("Ok")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .halign(Align::Center)
        .build();

    let content = GBox::builder()
        .orientation(Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .spacing(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    content.append(inner_content);
    content.append(&ok_button);

    let dialog_window = Window::builder()
        .transient_for(parent)
        .title(title)
        .child(&content)
        .build();

    dialog_window.connect_close_request(clone!(@strong inner_content => move |_| {
        content.remove(&inner_content);
        gtk::glib::Propagation::Proceed
    }));

    ok_button.connect_clicked(clone!(@strong dialog_window => move |_button| {
        dialog_window.close();
    }));

    dialog_window.present();
}

fn binary_dialog<F, G>(
    yes_label: &str,
    no_label: &str,
    parent: &impl IsA<Window>,
    title: &str,
    inner_content: &impl IsA<Widget>,
    on_yes: F,
    on_no: G
)
where
    F: Fn() -> CloseDialog + 'static,
    G: Fn() + 'static
{
    let yes_button = Button::builder()
        .label(yes_label)
        .margin_end(2)
        .build();

    let no_button = Button::builder()
        .label(no_label)
        .margin_end(2)
        .build();

    let button_wrapper = GBox::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::Center)
        .build();

    button_wrapper.append(&no_button);
    button_wrapper.append(&yes_button);

    let content = GBox::builder()
        .orientation(Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .spacing(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    content.append(inner_content);
    content.append(&button_wrapper);

    let dialog_window = Window::builder()
        .transient_for(parent)
        .title(title)
        .child(&content)
        .build();

    dialog_window.present();

    let window_p = Rc::new(RefCell::new(dialog_window));

    yes_button.connect_clicked(clone!(@strong window_p => move |_button| {
        if let CloseDialog::Yes = on_yes() {
            window_p.borrow().close();
        }
    }));

    no_button.connect_clicked(clone!(@strong window_p => move |_button| {
        on_no();
        window_p.borrow().close();
    }));
}

fn yes_no_dialog<F, G>(
    parent: &impl IsA<Window>,
    title: &str,
    inner_content: &impl IsA<Widget>,
    on_yes: F,
    on_no: G
)
where
    F: Fn() -> CloseDialog + 'static,
    G: Fn() + 'static
{
    binary_dialog("Yes", "No", parent, title, inner_content, on_yes, on_no)
}

fn ok_cancel_dialog<F, G>(
    parent: &impl IsA<Window>,
    title: &str,
    inner_content: &impl IsA<Widget>,
    on_ok: F,
    on_cancel: G
)
where
    F: Fn() -> CloseDialog + 'static,
    G: Fn() + 'static
{
    binary_dialog("Ok", "Cancel", parent, title, inner_content, on_ok, on_cancel)
}

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

pub fn choose_file_dialog<P: FnOnce(Result<File, GError>) + 'static>(
    parent: &impl IsA<Window>,
    title: &str, accept_label: &str,
    valid_filetypes: &impl IsA<gtk::gio::ListModel>,
    make_target_file: bool,
    callback: P
) {
    let dialog = FileDialog::builder()
        .title(title)
        .accept_label(accept_label)
        .filters(valid_filetypes)
        .build();

    if make_target_file {
        dialog.save(Some(parent), None::<&Cancellable>, callback);
    } else {
        dialog.open(Some(parent), None::<&Cancellable>, callback);
    }
}

pub fn ok_dialog_str(parent: &impl IsA<Window>, title: &str, mesg: &str) {
    let text_label = Label::builder()
        .label(mesg)
        .selectable(true)
        .build();

    ok_dialog(parent, title, &text_label)
}

pub fn yes_no_dialog_str<F, G>(parent: &impl IsA<Window>, title: &str, prompt: &str, on_yes: F, on_no: G)
where
    F: Fn() + 'static,
    G: Fn() + 'static,
{
    let text_label = Label::builder()
        .label(prompt)
        .selectable(true)
        .build();

    let on_yes = move || {
        on_yes();
        CloseDialog::Yes
    };

    yes_no_dialog(parent, title, &text_label, on_yes, on_no);
}

pub fn choose_color_dialog<P: FnOnce(Result<RGBA, GError>) + 'static>(
    parent: Option<&impl IsA<Window>>,
    callback: P
) {
    let dialog = ColorDialog::builder()
        .with_alpha(true)
        .build();

    dialog.choose_rgba(parent,
                       Some(&RGBA::new(0.5, 0.5, 0.5, 0.5)),
                       None::<&Cancellable>,
                       callback);
}
pub fn new_image_dialog<P: Fn(NewImageProps) + 'static>(
    parent: &impl IsA<Window>,
    callback: P
) {
    const DEFAULT_IMAGE_WIDTH: usize = 512;
    const DEFAULT_IMAGE_HEIGHT: usize = 512;
    const DEFAULT_FILL_COLOR: RGBA = RGBA::new(0.0, 0.0, 0.0, 0.0);

    let height_width_gadget = AspectRatioGadget::new_p(DEFAULT_IMAGE_WIDTH, DEFAULT_IMAGE_HEIGHT);
    let color_button = ColorField::new(Some("Fill Color"), DEFAULT_FILL_COLOR);

    let form = Form::builder()
        .title("New Image")
        .with_gadget(&*height_width_gadget.borrow())
        .with_field(&color_button)
        .build();

    let on_ok = move || {
        let props = NewImageProps {
            width: height_width_gadget.borrow().width(),
            height: height_width_gadget.borrow().height(),
            color: color_button.value(),
        };
        callback(props);
        CloseDialog::Yes
    };

    let on_cancel = || ();

    ok_cancel_dialog(parent, "New Image", form.widget(), on_ok, on_cancel)
}

pub fn scale_dialog<P: Fn(Scale) + 'static>(
    parent: &impl IsA<Window>,
    default_w: usize,
    default_h: usize,
    callback: P
) {
    let height_width_gadget = AspectRatioGadget::new_p(default_w, default_h);
    let methods = vec![
        ("Nearest Neighbor", ScaleMethod::NearestNeighbor),
        ("Bilinear", ScaleMethod::Bilinear),
    ];
    let method_field = DropdownField::new(Some("Scaling Algorithm"), methods, 0);

    let form = Form::builder()
        .title("Scale Image")
        .with_gadget(&*height_width_gadget.borrow())
        .with_field(&method_field)
        .build();

    let on_ok = move || {
        let hw = height_width_gadget.borrow();
        let action = Scale::new(hw.width(), hw.height(), method_field.value().clone());
        callback(action);
        CloseDialog::Yes
    };

    let on_cancel = || ();

    ok_cancel_dialog(parent, "Scale", form.widget(), on_ok, on_cancel);
}