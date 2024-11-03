extern crate image as image_lib;

use gtk::{gdk, gdk_pixbuf};
use std::sync::LazyLock;

macro_rules! static_icon_texture {
    ($filename:literal, $identifier:ident) => {
        pub static $identifier: LazyLock<gdk::Texture> = LazyLock::new(|| gdk::Texture::for_pixbuf(
            &gdk_pixbuf::Pixbuf::from_read(
                std::io::Cursor::new(include_bytes!($filename)
            )).unwrap()
        ));
    };
}

// Main ribbon
static_icon_texture!("../../icons/cursor.png", CURSOR);
static_icon_texture!("../../icons/pencil.png", PENCIL);
static_icon_texture!("../../icons/eyedropper.png", EYEDROPPER);
static_icon_texture!("../../icons/rectangle-select.png", RECTANGLE_SELECT);
static_icon_texture!("../../icons/magic-wand.png", MAGIC_WAND);
static_icon_texture!("../../icons/fill.png", FILL);
static_icon_texture!("../../icons/free-transform.png", FREE_TRANSFORM);
static_icon_texture!("../../icons/shape.png", SHAPE);
static_icon_texture!("../../icons/text.png", TEXT);

// About
static_icon_texture!("../../icons/logo.png", LOGO);

// Free transform toolbar
static_icon_texture!("../../icons/checkmark.png", CHECKMARK);
static_icon_texture!("../../icons/dotted-checkmark.png", DOTTED_CHECKMARK);
static_icon_texture!("../../icons/big-red-x.png", BIG_RED_X);

// Palette toolbar
static_icon_texture!("../../icons/right-arrow.png", RIGHT_ARROW);
static_icon_texture!("../../icons/swap.png", SWAP);

// Layer window
static_icon_texture!("../../icons/x.png", X);
static_icon_texture!("../../icons/lock.png", LOCK);
static_icon_texture!("../../icons/eyeball.png", EYEBALL);
static_icon_texture!("../../icons/plus.png", PLUS);
static_icon_texture!("../../icons/up-arrow.png", UP_ARROW);
static_icon_texture!("../../icons/down-arrow.png", DOWN_ARROW);

