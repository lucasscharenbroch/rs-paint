use crate::image::{DrawableImage, mk_transparent_checkerboard};

use gtk::{prelude::*, Orientation, DrawingArea, Button, Box as GBox};
use std::rc::Rc;
use std::cell::RefCell;
use gtk::gdk::RGBA;
use glib_macros::clone;

struct ColorButton {
    widget: Button,
    drawing_area: DrawingArea,
    checkerboard: DrawableImage,
    color: RGBA,
}

impl ColorButton {
    fn new_p(color: RGBA) -> Rc<RefCell<Self>> {
        let widget = Button::new();
        let drawing_area =  DrawingArea::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        widget.set_child(Some(&drawing_area));

        let checkerboard = mk_transparent_checkerboard();

        let state_p = Rc::new(RefCell::new(ColorButton {
            widget,
            drawing_area,
            checkerboard,
            color,
        }));

        state_p.borrow().drawing_area.set_draw_func(clone!(@strong state_p => move |_drawing_area, cr, _width, _height| {
            let transparent_pattern = state_p.borrow_mut().checkerboard.to_repeated_surface_pattern();
            let _ = cr.set_source(&transparent_pattern);
            let color = state_p.borrow().color;
            cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
        }));

        state_p
    }
}

pub struct Pallete {
    widget: GBox,
    color_buttons: Vec<Rc<RefCell<ColorButton>>>,
}


impl Pallete {
    pub fn new(colors: Vec<RGBA>) -> Self {
        let widget = GBox::new(Orientation::Horizontal, 10);

        let color_buttons = colors.iter()
            .map(|rgba| ColorButton::new_p(*rgba))
            .collect::<Vec<_>>();

        for w in color_buttons.iter() {
            widget.append(&w.borrow().widget);

        }

        Pallete {
            widget,
            color_buttons,
        }
    }

    pub fn widget(&self) -> &GBox {
        &self.widget
    }

    pub fn primary_color(&self) -> RGBA {
        todo!("primary color")
    }
}
