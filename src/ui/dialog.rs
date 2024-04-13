use gtk::{prelude::*, Window, Widget, TextView, TextBuffer, FileDialog, Button, Label, Orientation, Align};
use gtk::glib::{object::IsA, error::Error as GError};
use gtk::gio::{File, Cancellable};

fn run_window_with(parent: &impl IsA<Window>, title: &str, content: &impl IsA<Widget>) {
    let dialog_window = Window::builder()
        .transient_for(parent)
        .title(title)
        .child(content)
        .build();

    dialog_window.present();
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

    let content = gtk::Box::builder()
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

pub fn run_about_dialog(parent: &impl IsA<Window>) {
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

pub fn choose_file<P: FnOnce(Result<File, GError>) + 'static>(
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

pub fn popup_mesg(parent: &impl IsA<Window>, title: &str, mesg: &str) {
    let text_label = Label::builder()
        .label(mesg)
        .selectable(true)
        .build();

    ok_dialog(parent, title, &text_label)
}
