use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button, DrawingArea};
use gtk::cairo::Context;
use super::image::{Image, mk_test_image};
use std::rc::Rc;
use std::cell::RefCell;

const APP_ID: &str = "rs-paint";

#[derive(Clone)]
pub struct UiState {
    image: Image,
}

impl UiState {
    pub fn new() -> UiState {
        UiState {
            image: mk_test_image(),
        }
    }

    pub fn run(self) -> glib::ExitCode {
        let app = Application::builder().application_id(APP_ID).build();
        app.connect_activate(move |app| self.build_ui(app));
        app.run()
    }

    fn draw_image_canvas(&self, area: &DrawingArea, cr: &Context, width: i32, height: i32) {
        let image = mk_test_image();
        let x_offset = std::cmp::max(0, (width - image.width()) / 2);
        let y_offset = std::cmp::max(0, (height - image.height()) / 2);
        cr.set_source_surface(image.to_surface(), x_offset as f64, y_offset as f64);
        cr.paint();

        cr.scale(width as f64, height as f64);
        cr.set_line_width(0.1);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.rectangle(0.25, 0.25, 0.5, 0.5);
        cr.stroke();
    }

    fn build_ui(&self, app: &Application) {
        let state = Rc::new(RefCell::new(self.clone()));

        let button = Button::builder()
            .label("Press me!")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let drawing_area = DrawingArea::new();

        drawing_area.set_draw_func(move |area, cr, width, height|
                                   state.borrow().draw_image_canvas(area, cr, width, height));

        // Connect to "clicked" signal of `button`
        button.connect_clicked(|button| {
            // Set the label to "Hello World!" after the button has been clicked on
            button.set_label("Hello World!");
        });

        // Create a window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("My GTK App")
            .child(&button)
            .child(&drawing_area)
            .build();

        // Present window
        window.present();
    }
}
