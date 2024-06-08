use super::CloseDialog;

use gtk::{prelude::*, Align, Box as GBox, Button, Label, Orientation, Widget, Window};
use gtk::glib::object::IsA;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

macro_rules! first_ident {
    ($x:ident) => ($x);
    ($head:ident, $($tail:ident),*) => ($head);
}

// e.g. nary_dialog!(yes_no, yes, no)
macro_rules! nary_dialog {
    ( $name:ident, $( $variant:ident ),+ ) => { paste::item! {
        pub fn [< $name _dialog >]<$([< F $variant>]),*>(
            parent: &impl IsA<Window>,
            title: &str,
            inner_content: &impl IsA<Widget>,
            $(
            [< on_ $variant >]: [< F $variant >],
            )*
        )
        where
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

                let [< $variant _button >] = Button::builder()
                .label([< $variant _str >])
                .margin_end(2)
                .build();
            )*

            let button_wrapper = GBox::builder()
                .orientation(Orientation::Horizontal)
                .halign(Align::Center)
                .build();

            $(
            button_wrapper.prepend(&[< $variant _button >]);
            )*
            button_wrapper.set_focus_child(Some(&first_ident!($([< $variant _button >]),*)));

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

            dialog_window.connect_close_request(clone!(@strong inner_content => move |_| {
                content.remove(&inner_content);
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

        pub fn [< $name _dialog_str >]<$([< F $variant>]),*>(
            parent: &impl IsA<Window>,
            title: &str,
            prompt: &str,
            $(
            [< on_ $variant >]: [< F $variant >],
            )*
        )
        where
            $(
                [< F $variant >]: Fn() -> CloseDialog + 'static
            ),*
        {

            let text_label = Label::builder()
                .label(prompt)
                .selectable(true)
                .build();

            [< $name _dialog >](parent, title, &text_label, $([< on_ $variant >]),*);
        }
    } }
}

nary_dialog!(yes_no, yes, no);
nary_dialog!(ok, ok);
nary_dialog!(ok_cancel, ok, cancel);
nary_dialog!(cancel_discard, cancel, discard);

pub fn ok_dialog_(
    parent: &impl IsA<Window>,
    title: &str,
    inner_content: &impl IsA<Widget>,
) {
    ok_dialog(parent, title, inner_content, || CloseDialog::Yes);
}

pub fn ok_dialog_str_(
    parent: &impl IsA<Window>,
    title: &str,
    prompt: &str,
) {
    ok_dialog_str(parent, title, prompt, || CloseDialog::Yes);
}
