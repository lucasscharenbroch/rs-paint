use gtk::{prelude::*, Orientation, Box as GBox};
use gtk::gdk::RGBA;

pub struct Pallete {
    widget: GBox,
}

impl Pallete {
    pub fn new(colors: Vec<RGBA>) -> Self {
        let widget = GBox::new(Orientation::Horizontal, 10);

        let text = gtk::Label::builder()
            .label("test pallete")
            .build();

        widget.append(&text);

        Pallete {
            widget,
        }
    }

    pub fn widget(&self) -> &GBox {
        &self.widget
    }

    pub fn primary_color(&self) -> RGBA {
        todo!("primary color")
    }
}
