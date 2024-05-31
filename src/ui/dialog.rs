use crate::image::generate::NewImageProps;
use super::form::{Form, FormBuilder, NaturalField, RadioField, TextField};

use gtk::{prelude::*, Window, Widget, TextView, TextBuffer, FileDialog, Button, Label, Orientation, Align, Box as GBox};
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

fn ok_dialog(parent: &impl IsA<Window>, title: &str, inner_content: &impl IsA<Widget>) {
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

    dialog_window.present();

    ok_button.connect_clicked(move |_button| {
        dialog_window.close();
    });
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
    let text_content = TextBuffer::builder()
        .text("Information about RS-Paint")
        .build();

    let content = TextView::builder()
        .buffer(&text_content)
        .editable(false)
        .cursor_visible(false)
        .vexpand(true)
        .hexpand(true)
        .build();

    ok_dialog(parent, "About Rs-Paint", &content)
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

pub fn new_image_dialog<P: FnOnce(Result<NewImageProps, GError>) + 'static>(
    parent: &impl IsA<Window>,
    callback: P
) {
    let content = GBox::builder()
        .orientation(Orientation::Vertical)
        .build();

    let variants = vec![
        ("one", 1),
        ("two", 2),
        ("three", 3),
    ];

    let a = RadioField::new(Some("label 1"), variants.clone(), 0);
    let b = RadioField::new(None, variants, 10);

    let form = Form::builder()
        .title("New Image")
        .with_field(&a)
        .with_field(&b)
        .build();

    let on_ok = move || {
        println!("Got `{:?}` `{:?}`", a.value(), b.value());
        todo!()
    };

    let on_cancel = || ();

    ok_cancel_dialog(parent, "New Image", form.widget(), on_ok, on_cancel)
}