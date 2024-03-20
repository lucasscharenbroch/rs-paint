use gtk::{prelude::*, EventControllerScrollFlags};
use gtk::gdk::{Key, ModifierType};
use gtk::{glib, Application, ApplicationWindow, Button, DrawingArea, ScrolledWindow, EventControllerKey, EventControllerScroll};
use gtk::cairo::Context;
use super::image::{Image, mk_test_image, mk_transparent_pattern};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;
use gtk::cairo;
use gtk::glib::signal::Propagation;

#[derive(Clone)]
pub struct UiState {
    image: Image,
    image_zoom: f64,
    drawing_area: DrawingArea,
}

impl UiState {
    pub fn new() -> UiState {
        UiState {
            image: mk_test_image(),
            image_zoom: 2.0,
            drawing_area: DrawingArea::new(),
        }
    }

    pub fn run(self) -> glib::ExitCode {
        let app = Application::builder().build();
        app.connect_activate(move |app| self.build_ui(app));
        app.run()
    }

    fn draw_image_canvas(&self, area: &DrawingArea, cr: &Context, area_width: i32, area_height: i32) {
        let zoom = self.image_zoom;

        let img_width = self.image.pixels.len() as f64;
        let img_height = self.image.pixels[0].len() as f64;
        let x_offset = (area_width as f64 - img_width * zoom) / 2.0;
        let y_offset = (area_height as f64 - img_height * zoom) / 2.0;

        let image_surface_pattern = self.image.to_surface_pattern();
        let transparent_pattern = mk_transparent_pattern();

        cr.translate(x_offset as f64, y_offset as f64);
        cr.scale(zoom, zoom);
        cr.set_line_join(cairo::LineJoin::Bevel);

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

    fn update_image_canvas_sz(&mut self) {
        const CANVAS_SZ_MULT: f64 = 1.3;

        let image_width = self.image.pixels.len() as f64 * self.image_zoom * CANVAS_SZ_MULT;
        let image_height = self.image.pixels[0].len() as f64 * self.image_zoom * CANVAS_SZ_MULT;

        self.drawing_area.set_content_height(image_height as i32);
        self.drawing_area.set_content_width(image_width as i32);
    }

    fn inc_zoom(&mut self, inc: f64) {
        const MAX_ZOOM: f64 = 25.0;
        const MIN_ZOOM: f64 = 0.1;

        self.image_zoom += inc;
        if(self.image_zoom > MAX_ZOOM) {
            self.image_zoom = MAX_ZOOM;
        } else if(self.image_zoom < MIN_ZOOM) {
            self.image_zoom = MIN_ZOOM;
        }
    }

    fn handle_keypress(&mut self, key: Key, modifier: ModifierType) {
        const ZOOM_INC: f64 = 1.0;

        if modifier == ModifierType::CONTROL_MASK {
            if key == Key::equal {
                self.inc_zoom(ZOOM_INC);
                self.update_image_canvas_sz();
            } else if(key == Key::minus) {
                self.inc_zoom(-ZOOM_INC);
                self.update_image_canvas_sz();
            }
        }
    }

    fn handle_scroll(&mut self, event_controller: &EventControllerScroll, x: f64, y: f64) -> Propagation {
        if event_controller.current_event_state() == ModifierType::CONTROL_MASK {
            self.inc_zoom(y);
            self.update_image_canvas_sz();

            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    }

    fn build_ui(&self, app: &Application) {
        let state = Rc::new(RefCell::new(self.clone()));

        self.drawing_area.set_draw_func(clone!(@strong state => move |area, cr, width, height| {
            state.borrow_mut().update_image_canvas_sz();
            state.borrow().draw_image_canvas(area, cr, width, height);
        }));

        let main_frame = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Always)
            .vscrollbar_policy(gtk::PolicyType::Always)
            .child(&self.drawing_area)
            .kinetic_scrolling(true)
            .overlay_scrolling(true)
            .build();

        let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        scroll_controller.connect_scroll(clone!(@strong state => move |ecs, dx, dy| {
            state.borrow_mut().handle_scroll(ecs, dx, dy)
        }));

        main_frame.add_controller(scroll_controller);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("RS-Paint")
            .child(&main_frame)
            .build();

        let key_controller = EventControllerKey::new();

        key_controller.connect_key_pressed(clone!(@strong state => move |_, key, _, modifier| {
            state.borrow_mut().handle_keypress(key, modifier);
            Propagation::Proceed
        }));

        window.add_controller(key_controller);

        window.present();
    }
}
