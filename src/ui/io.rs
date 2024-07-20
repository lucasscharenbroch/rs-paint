use super::dialog::new_image_dialog;
use super::{dialog::{choose_file_dialog, ok_dialog_str_}, UiState};
use crate::image::io::LayeredImage;
use crate::image::{Image, generate::generate};
use crate::ui::MouseMode;

use gtk::prelude::*;
use gtk::gio::ListStore;
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

fn mk_file_filter_list(extss: Vec<Vec<&str>>) -> ListStore {
    let list = ListStore::new::<gtk::FileFilter>();

    let supported = gtk::FileFilter::new();
    supported.set_name(Some("Supported Files"));
    list.append(&supported);

    extss.iter().for_each(|exts| {
        let ff = gtk::FileFilter::new();
        exts.iter().for_each(|ext| {
            ff.add_suffix(ext);
            supported.add_suffix(ext)
        });
        ff.set_name(Some((exts.iter().skip(1).fold(String::from(exts[0]), |s, e| s + ", " + e)).as_str()));
        list.append(&ff);
    });

    let all = gtk::FileFilter::new();
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

pub fn image_project_formats() -> Vec<Vec<&'static str>> {
    vec![
        vec!["rsp"]
    ]
}

impl UiState {
    pub fn import(ui_p: Rc<RefCell<UiState>>) {
        let valid_filetypes = mk_file_filter_list(image_import_formats());

        choose_file_dialog(&ui_p.borrow().window, "Choose an image to import",
                    "Import", &valid_filetypes, false,
                    clone!(@strong ui_p => move |res| {
            if let Ok(res) = res {
                let path = res.path().unwrap();
                let path = path.as_path();
                let name = path.file_name().and_then(|os| os.to_str()).unwrap_or("[Untitled]");
                match Image::from_path(path) {
                    Ok(img) => {
                        UiState::new_tab(&ui_p, img, name);
                    },
                    Err(mesg) => {
                        ok_dialog_str_(
                            ui_p.borrow().window(),
                            "Import Error",
                            format!("Error during import: {}", mesg).as_str()
                        );
                    }
                }
            }
        }))
    }

    /// Prompt user for image file, load that image, then
    /// place it as a selection onto the current canvas
    pub fn import_onto(ui_p: Rc<RefCell<UiState>>) {
        let valid_filetypes = mk_file_filter_list(image_import_formats());

        choose_file_dialog(&ui_p.borrow().window, "Choose an image to import",
                    "Import", &valid_filetypes, false,
                    clone!(@strong ui_p => move |res| {
            if let Ok(res) = res {
                let path = res.path().unwrap();
                let path = path.as_path();
                match Image::from_path(path) {
                    Ok(img) => {
                        if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
                            canvas_p.borrow_mut().place_image(img);
                            ui_p.borrow().toolbar_p.borrow_mut().set_mouse_mode(MouseMode::free_transform(&mut canvas_p.borrow_mut()));
                        }
                    },
                    Err(mesg) => {
                        ok_dialog_str_(
                            ui_p.borrow().window(),
                            "Import Error",
                            format!("Error during import: {}", mesg).as_str()
                        );
                    }
                }
            }
        }))
    }

    pub fn export(ui_p: Rc<RefCell<UiState>>) {
        let valid_filetypes = mk_file_filter_list(image_export_formats());

        choose_file_dialog(&ui_p.borrow().window, "Export image",
                    "Export", &valid_filetypes, true,
                    clone!(@strong ui_p => move |res| {
            if let Ok(res) = res {
                let path = res.path().unwrap();
                let path = path.as_path();
                if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
                    if let Err(mesg) = canvas_p.borrow().layered_image().gen_entire_blended_image().to_file(path) {
                        ok_dialog_str_(
                            ui_p.borrow().window(),
                            "Export Error",
                            format!("Error during export: {}", mesg).as_str()
                        );
                        return;
                    }
                } else {
                    ok_dialog_str_(
                        ui_p.borrow().window(),
                        "Export Error",
                        "No image to export"
                    );
                    return;
                }

                // export success
                ui_p.borrow_mut().notify_tab_successful_export();
            }
        }))
    }

    pub fn load_project(ui_p: Rc<RefCell<UiState>>) {
        let valid_filetypes = mk_file_filter_list(image_project_formats());

        choose_file_dialog(&ui_p.borrow().window, "Choose a project to load",
                    "Load", &valid_filetypes, false,
                    clone!(@strong ui_p => move |res| {
            if let Ok(res) = res {
                let path = res.path().unwrap();
                let path = path.as_path();
                let name = path.file_name().and_then(|os| os.to_str()).unwrap_or("[Untitled]");

                match LayeredImage::from_path(path) {
                    Ok(layered_image) => {
                        UiState::new_tab_from_layered_image(&ui_p, layered_image, name);
                    },
                    Err(mesg) => {
                        ok_dialog_str_(
                            ui_p.borrow().window(),
                            "Load Error",
                            format!("Error while loading project: {}", mesg).as_str()
                        );
                    }
                }
            }
        }))
    }

    pub fn save_project_as(ui_p: Rc<RefCell<UiState>>) {
        let valid_filetypes = mk_file_filter_list(image_project_formats());

        choose_file_dialog(&ui_p.borrow().window, "Save project as",
                    "Save", &valid_filetypes, true,
                    clone!(@strong ui_p => move |res| {
            if let Ok(res) = res {
                let path = res.path().unwrap();
                let path = path.as_path();
                if let Some(canvas_p) = ui_p.borrow().active_canvas_p() {
                    if let Err(mesg) = canvas_p.borrow().layered_image().unfused().to_file(path) {
                        ok_dialog_str_(
                            ui_p.borrow().window(),
                            "Save Error",
                            format!("Error while saving project: {}", mesg).as_str()
                        );
                        return;
                    }
                } else {
                    ok_dialog_str_(
                        ui_p.borrow().window(),
                        "Save Error",
                        "No image to save"
                    );
                    return;
                }

                // export success
                ui_p.borrow_mut().notify_tab_successful_export();
            }
        }))
    }

    pub fn new(ui_p: Rc<RefCell<UiState>>) {
        new_image_dialog(&ui_p.borrow().window, clone!(@strong ui_p => move |props| {
            let image = generate(props);
            UiState::new_tab(&ui_p, image, "[untitled]");
        }))
    }
}
