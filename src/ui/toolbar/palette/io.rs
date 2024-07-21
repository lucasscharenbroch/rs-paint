use crate::image::Pixel;

use crate::ui::UiState;
use crate::ui::dialog::{choose_file_dialog, ok_dialog_str_};

use std::rc::Rc;
use std::cell::RefCell;
use gtk::prelude::FileExt;
use regex::Regex;
use glib_macros::clone;
use std::fs::File;

// Use dead-simple encoding: one color per line,
// each line matches one of the following:
// '#' {3, 4 or 6, or 8 hex digits}
// {0-255},{0-255},{0-255}[,{0-255}[,]]

const HEX3_PATTERN: &str = r"^#([[:xdigit:]])([[:xdigit:]])([[:xdigit:]])$";
fn parse_hex3(s: &str) -> Option<Pixel> {
    let re = Regex::new(HEX3_PATTERN).unwrap();
    if let Some((_match_str, [r, g, b])) = re.captures(s).map(|c| c.extract()) {
        // multiplying by 17 (0x11) has the effect of doubling the digit (0x1 => 0x11, 0xf => 0xff)
        Some(Pixel::from_rgb(
            u8::from_str_radix(r, 16).unwrap() * 17,
            u8::from_str_radix(g, 16).unwrap() * 17,
            u8::from_str_radix(b, 16).unwrap() * 17,
        ))
    } else {
        None
    }
}

const HEX4_PATTERN: &str = r"^#([[:xdigit:]])([[:xdigit:]])([[:xdigit:]])([[:xdigit:]])$";
fn parse_hex4(s: &str) -> Option<Pixel> {
    let re = Regex::new(HEX4_PATTERN).unwrap();
    if let Some((_match_str, [r, g, b, a])) = re.captures(s).map(|c| c.extract()) {
        Some(Pixel::from_rgba(
            u8::from_str_radix(r, 16).unwrap() * 17,
            u8::from_str_radix(g, 16).unwrap() * 17,
            u8::from_str_radix(b, 16).unwrap() * 17,
            u8::from_str_radix(a, 16).unwrap() * 17,
        ))
    } else {
        None
    }
}

const HEX6_PATTERN: &str = r"^#([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})$";
fn parse_hex6(s: &str) -> Option<Pixel> {
    let re = Regex::new(HEX6_PATTERN).unwrap();
    if let Some((_match_str, [r, g, b])) = re.captures(s).map(|c| c.extract()) {
        Some(Pixel::from_rgb(
            u8::from_str_radix(r, 16).unwrap(),
            u8::from_str_radix(g, 16).unwrap(),
            u8::from_str_radix(b, 16).unwrap(),
        ))
    } else {
        None
    }
}

const HEX8_PATTERN: &str = r"^#([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})$";
fn parse_hex8(s: &str) -> Option<Pixel> {
    let re = Regex::new(HEX8_PATTERN).unwrap();
    if let Some((_match_str, [r, g, b, a])) = re.captures(s).map(|c| c.extract()) {
        Some(Pixel::from_rgba(
            u8::from_str_radix(r, 16).unwrap(),
            u8::from_str_radix(g, 16).unwrap(),
            u8::from_str_radix(b, 16).unwrap(),
            u8::from_str_radix(a, 16).unwrap(),
        ))
    } else {
        None
    }
}

const RGB_PATTERN: &str = r"^(\d+),(\d+),(\d+)$";
fn parse_rgb(s: &str) -> Option<Pixel> {
    let re = Regex::new(RGB_PATTERN).unwrap();
    if let Some((_match_str, [r, g, b])) = re.captures(s).map(|c| c.extract()) {
        Some(Pixel::from_rgb(
            u8::from_str_radix(r, 10).unwrap_or(255),
            u8::from_str_radix(g, 10).unwrap_or(255),
            u8::from_str_radix(b, 10).unwrap_or(255),
        ))
    } else {
        None
    }
}

const RGBA_PATTERN: &str = r"^(\d+),(\d+),(\d+),(\d+),?$";
fn parse_rgba(s: &str) -> Option<Pixel> {
    let re = Regex::new(RGBA_PATTERN).unwrap();
    if let Some((_match_str, [r, g, b, a])) = re.captures(s).map(|c| c.extract()) {
        Some(Pixel::from_rgba(
            u8::from_str_radix(r, 10).unwrap_or(255),
            u8::from_str_radix(g, 10).unwrap_or(255),
            u8::from_str_radix(b, 10).unwrap_or(255),
            u8::from_str_radix(a, 10).unwrap_or(255),
        ))
    } else {
        None
    }
}

fn string_to_colors(string: &str) -> Result<Vec<Pixel>, String> {
    let parsers = [
        parse_hex3,
        parse_hex4,
        parse_hex6,
        parse_hex8,
        parse_rgb,
        parse_rgba,
    ];

    let mut res = Vec::new();

    for (i, line) in string.split("\n").enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        if let Some(color) = parsers.iter()
            .map(|p| p(line.trim()))
            .filter_map(|opt| opt)
            .next()
        {
            res.push(color);
        } else {
            return Err(format!("Couldn't parse line {} (`{line}`)", i + 1));
        }
    }

    Ok(res)
}

fn colors_to_string(colors: Vec<Pixel>) -> String {
    todo!()
}

impl UiState {
    pub fn import_palette(ui_p: Rc<RefCell<UiState>>) {
        let valid_filetypes = gtk::gio::ListStore::new::<gtk::FileFilter>();
        let all = gtk::FileFilter::new();
        all.set_name(Some("All Files"));
        all.add_pattern("*");
        valid_filetypes.append(&all);

        fn gfile_to_colors(gfile: gtk::gio::File) -> Result<Vec<Pixel>, String> {
            let path = gfile.path().unwrap();
            let mut file = File::open(path).map_err(|e| e.to_string())?;
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut file, &mut contents).map_err(|e| e.to_string())?;
            string_to_colors(&contents)
        }

        choose_file_dialog(&ui_p.borrow().window, "Choose an palette to import",
            "Import Palette", &valid_filetypes, false,
            clone!(@strong ui_p => move |res| {
                if let Ok(gfile) = res {
                    match gfile_to_colors(gfile) {
                        Ok(colors) => {
                            let colors = colors.iter().map(|pix| Pixel::to_rgba_struct(&pix)).collect::<Vec<_>>();
                            ui_p.borrow().toolbar_p.borrow().palette_p.borrow_mut().overwrite_colors(colors);
                        },
                        Err(mesg) => {
                            ok_dialog_str_(
                                ui_p.borrow().window(),
                                "Palette Import Error",
                                format!("Error during import: {}", mesg).as_str()
                            );
                        },
                    }
                }
            })
        );
        println!("{:?}", string_to_colors("#01020304\n      233,432,33\n1,1,1,1,\n\n"));
        // todo!("import palette");
    }

    pub fn export_palette(ui_p: Rc<RefCell<UiState>>) {
        todo!("export palette");
    }
}
