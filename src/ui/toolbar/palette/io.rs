use crate::image::Pixel;

use crate::ui::UiState;

use std::rc::Rc;
use std::cell::RefCell;
use regex::Regex;

// Use dead-simple encoding: one color per line,
// each line matches one of the following:
// '#' {3, 4 or 6, or 8 hex digits}
// {0-255},{0-255},{0-255}[,{0-255}[,]]

const HEX3_PATTERN: &str = r"#([[:xdigit:]])([[:xdigit:]])([[:xdigit:]])";
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

const HEX4_PATTERN: &str = r"#([[:xdigit:]])([[:xdigit:]])([[:xdigit:]])([[:xdigit:]])";
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

const HEX6_PATTERN: &str = r"#([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})";
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

const HEX8_PATTERN: &str = r"#([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})";
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

const RGB_PATTERN: &str = r"(\d+),(\d+),(\d+)";
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

const RGBA_PATTERN: &str = r"(\d+),(\d+),(\d+),(\d+),?";
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

fn string_to_colors(string: String) -> Result<Vec<Pixel>, String> {
    for line in string.split("\n") {

    }
    todo!()
}

fn colors_to_string(colors: Vec<Pixel>) -> String {
    todo!()
}

impl UiState {
    pub fn import_palette(ui_p: Rc<RefCell<UiState>>) {
        todo!("import palette");
    }

    pub fn export_palette(ui_p: Rc<RefCell<UiState>>) {
        todo!("export palette");
    }
}
