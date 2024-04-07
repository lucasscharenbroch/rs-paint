use gtk::{prelude::*, Window, Widget, TextView, TextBuffer, FileDialog};
use gtk::glib::{object::IsA, error::Error as GError};
use gtk::gio::{File, Cancellable};

fn run_window_with(parent: &impl IsA<Window>, title: &str, content: &impl IsA<Widget>) {
    let dialog_window = Window::builder()
        .transient_for(parent)
        .title(title)
        .child(content)
        .default_width(300)
        .default_height(300)
        .build();

    dialog_window.present();
}

pub fn run_about_dialog(parent: &impl IsA<Window>) {
    let text_content = TextBuffer::builder()
        .text("Information about RS-Paint")
        .build();

    let content = TextView::builder()
        .buffer(&text_content)
        .editable(false)
        .cursor_visible(false)
        .build();

    run_window_with(parent, "About Rs-Paint", &content)
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
