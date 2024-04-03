use gtk::FileFilter;
use gtk::gio::ListStore;

use super::{dialog::choose_file, UiState};

// File Filters

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

pub fn import(ui_state: &mut UiState) {
    let extensions = vec![
        vec!["png"],
        vec!["jpg", "jpeg"],
    ];

    let valid_filetypes = mk_file_filter_list(extensions);

    choose_file(&ui_state.window, "Choose an image to import", "Import", &valid_filetypes, |res| {
        println!("got file: {:?}", res)
    })
}
