use gtk::prelude::*;
use gtk::FileFilter;
use gtk::gio::ListStore;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

use super::{dialog::choose_file, UiState};
use crate::image::Image;

fn mk_file_filter_list(extss: Vec<Vec<&str>>) -> ListStore {
    let list = ListStore::new::<FileFilter>();

    let supported = FileFilter::new();
    supported.set_name(Some("Supported Files"));
    list.append(&supported);

    extss.iter().for_each(|exts| {
        let ff = FileFilter::new();
        exts.iter().for_each(|ext| {
            ff.add_suffix(ext);
            supported.add_suffix(ext)
        });
        ff.set_name(Some((exts.iter().skip(1).fold(String::from(exts[0]), |s, e| s + ", " + e)).as_str()));
        list.append(&ff);
    });

    let all = FileFilter::new();
    all.set_name(Some("All Files"));
    all.add_pattern("*");
    list.append(&all);

    list
}

pub fn image_import_formats() -> Vec<Vec<&'static str>> {
    vec![
        vec!["png"],
        vec!["jpg", "jpeg"],
        vec!["gif"],
        vec!["webp"],
        vec!["bmp"],
        vec!["ico"],
        vec!["avif"],
        vec!["tiff", "tif"],
        vec!["dds"],
        vec!["ff"],
        vec!["hdr"],
        vec!["exr"],
        vec!["pnm"],
        vec!["tga"],
    ]
}

pub fn image_export_formats() -> Vec<Vec<&'static str>> {
    vec![
        vec!["png"],
        vec!["jpg", "jpeg"],
        vec!["gif"],
        vec!["webp"],
        vec!["bmp"],
    ]
}

pub fn import(ui_state: Rc<RefCell<UiState>>) {
    let valid_filetypes = mk_file_filter_list(image_import_formats());

    choose_file(&ui_state.borrow().window, "Choose an image to import",
                "Import", &valid_filetypes, false,
                clone!(@strong ui_state => move |res| {
        if let Ok(res) = res {
            let path = res.path().unwrap();
            let path = path.as_path();
            match Image::from_file(path) {
                Ok(img) => {
                    let ui = ui_state.borrow_mut();
                    let mut canvas = ui.canvas_p.borrow_mut();
                    canvas.image().set_image(img);
                    canvas.save_state_for_undo();
                    canvas.update();
                },
                Err(e) => {
                    panic!("Error loading file: {:?}", e); // TODO
                }
            }
        }
    }))
}

pub fn export(ui_state: Rc<RefCell<UiState>>) {
    let valid_filetypes = mk_file_filter_list(image_export_formats());

    choose_file(&ui_state.borrow().window, "Export image",
                "Export", &valid_filetypes, true,
                clone!(@strong ui_state => move |res| {
        if let Ok(res) = res {
            let path = res.path().unwrap();
            let path = path.as_path();
            if let Err(e) = ui_state.borrow().canvas_p.borrow().image_ref().image().to_file(path) {
                panic!("Error exporting file: {:?}", e); // TODO
            }
        }
    }))
}
