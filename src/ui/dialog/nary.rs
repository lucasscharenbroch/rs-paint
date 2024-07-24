use super::CloseDialog;

use gtk::{prelude::*};
use gtk::glib::object::IsA;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

macro_rules! first_ident {
    ($x:ident) => ($x);
    ($head:ident $(, $tail:ident )+) => ($head);
}

macro_rules! last_ident {
    ($x:ident) => ($x);
    ($head:ident $(, $tail:ident )+) => {
        last_ident!($($tail),*)
    };
}

// e.g. nary_dialog!(yes_no, yes, no)
macro_rules! nary_dialog {
    ( $name:ident $(, $variant:ident )+ ; $is_modal:expr ) => { paste::item! {
        pub fn [< $name _dialog >]<G, $([< F $variant>]),*>(
            parent: &impl IsA<gtk::Window>,
            title: &str,
            inner_content: &impl IsA<gtk::Widget>,
            $(
            [< on_ $variant >]: [< F $variant >],
            )*
            on_force_close: G,
        )
        where
            G: Fn() + 'static,
            $(
                [< F $variant >]: Fn() -> CloseDialog + 'static
            ),*
        {
            $(
                let [< $variant _str >] = {
                    let lower_string = stringify!($variant).to_string();
                    let mut chars = lower_string.chars();

                    // capitalize the first letter
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                };

                let [< $variant _button >] = gtk::Button::builder()
                .label([< $variant _str >])
                .margin_end(2)
                .build();
            )*

            let button_wrapper = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .halign(gtk::Align::Center)
                .build();

            $(
            button_wrapper.prepend(&[< $variant _button >]);
            )*
            button_wrapper.set_focus_child(Some(&first_ident!($([< $variant _button >]),*)));

            let content = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
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

            let dialog_window = gtk::Window::builder()
                .transient_for(parent)
                .title(title)
                .child(&content)
                .modal($is_modal)
                .build();

            dialog_window.connect_close_request(clone!(@strong inner_content => move |_| {
                content.remove(&inner_content);
                on_force_close();

                gtk::glib::Propagation::Proceed
            }));

            dialog_window.present();
            dialog_window.grab_focus();

            let window_p = Rc::new(RefCell::new(dialog_window));

            $(
                [< $variant _button >].connect_clicked(clone!(@strong window_p => move |_button| {
                    if let CloseDialog::Yes = [< on_ $variant >]() {
                        window_p.borrow().close();
                    }
                }));
            )*
        }

        pub fn [< $name _dialog_str >]<G, $([< F $variant>]),*>(
            parent: &impl IsA<gtk::Window>,
            title: &str,
            prompt: &str,
            $(
            [< on_ $variant >]: [< F $variant >],
            )*
            on_force_close: G,
        )
        where
            G: Fn() + 'static,
            $(
                [< F $variant >]: Fn() -> CloseDialog + 'static
            ),*
        {

            let text_label = gtk::Label::builder()
                .label(prompt)
                .selectable(true)
                .build();

            [< $name _dialog >](parent, title, &text_label, $([< on_ $variant >]),*, on_force_close);
        }
    } }
}

nary_dialog!(yes_no, yes, no; false);
nary_dialog!(ok, ok; false);
nary_dialog!(modal_ok, ok; true);
nary_dialog!(close, close; false);
nary_dialog!(ok_cancel, ok, cancel; false);
nary_dialog!(cancel_discard, cancel, discard; false);

pub fn ok_dialog_(
    parent: &impl IsA<gtk::Window>,
    title: &str,
    inner_content: &impl IsA<gtk::Widget>,
) {
    ok_dialog(parent, title, inner_content, || CloseDialog::Yes, || ());
}

pub fn ok_dialog_str_(
    parent: &impl IsA<gtk::Window>,
    title: &str,
    prompt: &str,
) {
    ok_dialog_str(parent, title, prompt, || CloseDialog::Yes, || ());
}

// no yes/no buttons: just a window with a title and content
pub fn no_button_dialog(
    parent: &impl IsA<gtk::Window>,
    title: &str,
    inner_content: &impl IsA<gtk::Widget>,
) -> gtk::Window {
    let wrapper = gtk::Box::builder()
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .spacing(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    wrapper.append(inner_content);

    let dialog_window = gtk::Window::builder()
        .transient_for(parent)
        .title(title)
        .child(&wrapper)
        .build();

    dialog_window.present();
    dialog_window.grab_focus();

    dialog_window.connect_close_request(clone!(@strong inner_content => move |_| {
        wrapper.remove(&inner_content);
        gtk::glib::Propagation::Proceed
    }));

    dialog_window
}
