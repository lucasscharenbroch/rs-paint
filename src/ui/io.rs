use gtk::prelude::*;
use gtk::FileFilter;
use gtk::gio::ListStore;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

use super::{dialog::{choose_file, popup_mesg}, UiState};
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

pub fn import(ui_p: Rc<RefCell<UiState>>) {
    let valid_filetypes = mk_file_filter_list(image_import_formats());

    choose_file(&ui_p.borrow().window, "Choose an image to import",
                "Import", &valid_filetypes, false,
                clone!(@strong ui_p => move |res| {
        if let Ok(res) = res {
            let path = res.path().unwrap();
            let path = path.as_path();
            let name = path.file_name().and_then(|os| os.to_str()).unwrap_or("[Untitled]");
            match Image::from_file(path) {
                Ok(img) => {
                    UiState::new_tab(&ui_p, img, name);
                },
                Err(mesg) => {
                    popup_mesg(ui_p.borrow().window(), "Import Error",
                               format!("Error during import: {}", mesg).as_str());
                }
            }
        }
    }))
}

pub fn export(ui_p: Rc<RefCell<UiState>>) {
    let valid_filetypes = mk_file_filter_list(image_export_formats());

    choose_file(&ui_p.borrow().window, "Export image",
                "Export", &valid_filetypes, true,
                clone!(@strong ui_p => move |res| {
        if let Ok(res) = res {
            let path = res.path().unwrap();
            let path = path.as_path();
            if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
                if let Err(mesg) = canvas_p.borrow().image_ref().image().to_file(path) {
                    popup_mesg(ui_p.borrow().window(), "Export Error",
                                format!("Error during export: {}", mesg).as_str());
                } else {
                    // export success
                    ui_p.borrow_mut().notify_tab_successful_export();
                }
            } else {
                popup_mesg(ui_p.borrow().window(), "Export Error",
                           "No image to export");
            }
        }
    }))
}
