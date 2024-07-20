use gtk::prelude::*;

pub struct Infobar {
    widget: gtk::CenterBox,
}

impl Infobar {
    pub fn new() -> Self {
        let widget = gtk::CenterBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .margin_start(15)
            .margin_end(15)
            .build();

        widget.set_start_widget(Some(&gtk::Label::new(Some("info"))));
        widget.set_end_widget(Some(&gtk::Label::new(Some("here"))));

        Infobar {
            widget,
        }
    }

    pub fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.widget
    }
}

