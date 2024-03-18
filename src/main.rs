mod image;

use gtk::cairo::ffi::cairo_format_stride_for_width;
use gtk::cairo::ImageSurface;
use gtk::cairo::Format;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button, DrawingArea};

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {

    let drawing_area = DrawingArea::new();

    // Create a button with label and margins
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();



    drawing_area.set_draw_func(|_area, cr, width, height| {
        let image: image::Image = image::mk_test_image();
        let height = image.pixels.len();
        let width = image.pixels[0].len();

        let surface: ImageSurface = ImageSurface::create_for_data(image.to_u8_vec(), Format::ARgb32, width as i32, height as i32, 4 * width as i32).unwrap();
        cr.set_source_surface(surface, 0.0, 0.0);
        cr.paint();

        cr.scale(width as f64, height as f64);
        cr.set_line_width(0.1);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.rectangle(0.25, 0.25, 0.5, 0.5);
        cr.stroke();
    });

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


