use gtk::{prelude::*, Window, Widget, TextView, TextBuffer};
use gtk::glib::object::IsA;

fn run_window_with(parent: &impl IsA<Window>, title: &str, content: &impl IsA<Widget>) {
    let dialog_window = Window::builder()
        .transient_for(parent)
        .title(title)
        .child(content)
        .build();

    dialog_window.present();
}

pub fn run_about_dialog(parent: &impl IsA<Window>) {
    let text_content = TextBuffer::builder()
        .text("Information about RS-Paint")
        .build();

    let content = TextView::builder()
        .buffer(&text_content)
        .build();

    run_window_with(parent, "About Rs-Paint", &content)
}
