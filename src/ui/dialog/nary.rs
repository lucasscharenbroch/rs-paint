use super::CloseDialog;

use gtk::{prelude::*, Align, Box as GBox, Button, Label, Orientation, Widget, Window};
use gtk::glib::object::IsA;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

fn unary_dialog(
    label: &str,
    parent: &impl IsA<Window>,
    title: &str,
    inner_content: &impl IsA<Widget>
) {
    let ok_button = Button::builder()
        .label(label)
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
    content.set_focus_child(Some(&ok_button));

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
    dialog_window.grab_focus();
}

fn binary_dialog<F, G>(
    yes_label: &str,
    no_label: &str,
    parent: &impl IsA<Window>,
    title: &str,
    inner_content: &impl IsA<Widget>,
    on_yes: F,
    on_no: G,
    focus_yes_button: bool,
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
    button_wrapper.set_focus_child(Some(if focus_yes_button {
        &yes_button
    } else {
        &no_button
    }));

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
    content.set_focus_child(Some(&button_wrapper));

    let dialog_window = Window::builder()
        .transient_for(parent)
        .title(title)
        .child(&content)
        .build();

    dialog_window.present();
    dialog_window.grab_focus();

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

pub fn ok_dialog(parent: &impl IsA<Window>, title: &str, inner_content: &impl IsA<Widget>) {
    unary_dialog("Ok", parent, title, inner_content);
}

pub fn ok_dialog_str(parent: &impl IsA<Window>, title: &str, mesg: &str) {
    let text_label = Label::builder()
        .label(mesg)
        .selectable(true)
        .build();

    ok_dialog(parent, title, &text_label)
}

fn yes_no_dialog<F, G>(
    parent: &impl IsA<Window>,
    title: &str,
    inner_content: &impl IsA<Widget>,
    on_yes: F,
    on_no: G,
)
where
    F: Fn() -> CloseDialog + 'static,
    G: Fn() + 'static
{
    binary_dialog(
            "Yes",
        "No",
        parent,
        title,
        inner_content,
        on_yes,
        on_no,
        true,
    )
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

pub fn ok_cancel_dialog<F, G>(
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
    binary_dialog(
        "Ok",
        "Cancel",
        parent,
        title,
        inner_content,
        on_ok,
        on_cancel,
        true
    );
}

fn discard_cancel_dialog<F, G>(
    parent: &impl IsA<Window>,
    title: &str,
    inner_content: &impl IsA<Widget>,
    on_discard: F,
    on_cancel: G
)
where
    F: Fn() -> CloseDialog + 'static,
    G: Fn() + 'static
{
    binary_dialog(
        "Discard",
        "Cancel",
        parent,
        title,
        inner_content,
        on_discard,
        on_cancel,
        false
    );
}

pub fn discard_cancel_dialog_str<F, G>(parent: &impl IsA<Window>, title: &str, prompt: &str, on_yes: F, on_no: G)
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

    discard_cancel_dialog(parent, title, &text_label, on_yes, on_no);
}
