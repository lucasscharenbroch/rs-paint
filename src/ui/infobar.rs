use gtk::prelude::*;

use super::canvas::Canvas;

pub struct Infobar {
    size_label: gtk::Label,
    cursor_pos_label: gtk::Label,
    widget: gtk::CenterBox,
}

impl Infobar {
    pub fn new() -> Self {
        let size_label = gtk::Label::new(None);
        let cursor_pos_label = gtk::Label::new(None);

        let widget = gtk::CenterBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .margin_start(15)
            .margin_end(15)
            .build();

        widget.set_start_widget(Some(&cursor_pos_label));
        widget.set_end_widget(Some(&size_label));

        Infobar {
            widget,
            size_label,
            cursor_pos_label,
        }
    }

    pub fn update(&self, canvas: &Canvas) {
        let width = canvas.image_width();
        let height = canvas.image_height();

        self.size_label.set_label(format!("{width} x {height}").as_str());

        let (x, y) = canvas.cursor_pos_pix_i();

        self.cursor_pos_label.set_label(format!("{x}, {y}").as_str());
    }

    pub fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.widget
    }
}

