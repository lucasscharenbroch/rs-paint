use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button, DrawingArea, Box, Orientation, Frame};
use gtk::cairo::Context;
use super::image::{Image, mk_test_image, mk_transparent_pattern};
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

    fn draw_image_canvas(&self, area: &DrawingArea, cr: &Context, area_width: i32, area_height: i32) {
        // let x_offset = std::cmp::max(0, (width - self.image.width()) / 2);
        // let y_offset = std::cmp::max(0, (height - self.image.height()) / 2);

        // cr.translate(x_offset as f64, y_offset as f64);
        // cr.scale(scale_factor, scale_factor);

        let zoom = 4.5;
        let img_width = self.image.pixels.len() as f64;
        let img_height = self.image.pixels[0].len() as f64;

        let image_surface_pattern = self.image.to_surface_pattern();
        let transparent_pattern = mk_transparent_pattern();

        cr.scale(zoom, zoom);

        const TRANSPARENT_CHECKER_SZ: f64 = 10.0;
        let trans_scale = TRANSPARENT_CHECKER_SZ / zoom;
        cr.scale(trans_scale, trans_scale);
        cr.rectangle(0.0, 0.0, img_width / trans_scale, img_height / trans_scale);
        cr.set_source(transparent_pattern);
        cr.fill();
        cr.scale(1.0 / trans_scale, 1.0 / trans_scale);

        cr.rectangle(0.0, 0.0, img_width, img_height);
        cr.set_source(image_surface_pattern);
        cr.fill();

        const BORDER_WIDTH: f64 = 3.0;
        cr.rectangle(0.0, 0.0, img_width, img_height);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(BORDER_WIDTH / zoom);
        cr.stroke();
    }

    fn build_ui(&self, app: &Application) {
        let state = Rc::new(RefCell::new(self.clone()));

        let drawing_area = DrawingArea::new();

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
