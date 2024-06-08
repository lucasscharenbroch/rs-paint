mod nary;
mod info;

pub use info::{about_dialog, keyboard_shortcuts_dialog};
pub use nary::*;

use crate::image::generate::NewImageProps;
use crate::ui::form::ColorField;
use super::form::DropdownField;
use super::form::{Form, gadget::AspectRatioGadget};
use crate::image::scale::{Scale, ScaleMethod};

use gtk::{prelude::*, FileDialog, Window};
use gtk::ColorDialog;
use gtk::glib::{object::IsA, error::Error as GError};
use gtk::gio::{File, Cancellable};
use gtk::gdk::RGBA;

pub enum CloseDialog {
    Yes,
    No,
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
        ("Bilinear", ScaleMethod::Bilinear),
        ("Nearest Neighbor", ScaleMethod::NearestNeighbor),
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
