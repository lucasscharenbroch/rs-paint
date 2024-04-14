use gtk::{gio::Cancellable, ColorDialog, ApplicationWindow, gdk::RGBA};

pub fn choose_color() {
    let dialog = ColorDialog::builder()
        .with_alpha(true)
        .build();

    dialog.choose_rgba(ApplicationWindow::NONE, Some(&RGBA::new(0.5, 0.5, 0.5, 0.5)), None::<&Cancellable>, |x| {println!("{:?}", x)});
}