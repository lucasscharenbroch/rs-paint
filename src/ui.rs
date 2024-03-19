use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button, DrawingArea, Box, Orientation, Frame};
use gtk::cairo::Context;
use super::image::{Image, mk_test_image};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

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
        let app = Application::builder().build();
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

        let drawing_area = DrawingArea::builder()
            .content_height(100)
            .build();

        drawing_area.set_draw_func(clone!(@strong state => move |area, cr, width, height|
                                                                state.borrow().draw_image_canvas(area, cr, width, height)));

        let main_frame = Frame::new(None);
        main_frame.set_child(Some(&drawing_area));

        // Create a window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("RS-Paint")
            .child(&main_frame)
            .build();

        // Present window
        window.present();
    }
}
