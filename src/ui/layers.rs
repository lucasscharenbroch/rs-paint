use gtk::prelude::*;

use crate::image::LayeredImage;

/// Wrapper struct for the ui within the image layers dialog
pub struct LayersUi {
    widget: gtk::Box,
}

impl LayersUi {
    pub fn from_layered_image(image: &LayeredImage) -> Self {
        let mut res = LayersUi {
            widget: gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build(),
        };

        res.update(image);
        res
    }

    pub fn update(&mut self, image: &LayeredImage) {
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child)
        }

        // TODO
        self.widget.append(&gtk::Label::new(Some("hello")));
    }

    pub fn update_thumbnails(&self) {
        self.widget.queue_draw();
    }

    pub fn widget(&self) -> &impl gtk::prelude::IsA<gtk::Widget> {
        &self.widget
    }
}
