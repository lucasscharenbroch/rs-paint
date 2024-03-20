use super::super::image::{Image, mk_transparent_pattern};
use super::UiState;

use gtk::prelude::*;
use gtk::{Frame};
use gtk::gdk::{ModifierType};
use gtk::{DrawingArea, EventControllerScroll, EventControllerScrollFlags};
use gtk::cairo::Context;
use gtk::cairo;
use gtk::glib::signal::Propagation;
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use glib_macros::clone;

#[derive(Clone)]
pub struct Canvas {
    image: Image,
    zoom: f64,
    drawing_area: DrawingArea,
    frame: Frame,
}

impl Canvas {
    pub fn new(image: Image) -> Rc<RefCell<Canvas>> {
        let drawing_area =  DrawingArea::new();
        let frame =  Frame::builder()
            .child(&drawing_area)
            .build();

        let state = Rc::new(RefCell::new(Canvas {
            image,
            zoom: 1.0,
            drawing_area,
            frame,
        }));

        state.borrow().drawing_area.set_draw_func(clone!(@strong state => move |area, cr, width, height| {
            state.borrow().draw(area, cr, width, height);
        }));

        let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::BOTH_AXES);
        scroll_controller.connect_scroll(clone!(@strong state => move |ecs, dx, dy| {
            state.borrow_mut().handle_scroll(ecs, dx, dy)
        }));

        state.borrow_mut().frame.add_controller(scroll_controller);

        state
    }

    pub fn widget(&self) -> &Frame {
        &self.frame
    }

    fn update_image_canvas_sz(&mut self) {
        const CANVAS_SZ_MULT: f64 = 1.3;

        let image_width = self.image.pixels.len() as f64 * self.zoom * CANVAS_SZ_MULT;
        let image_height = self.image.pixels[0].len() as f64 * self.zoom * CANVAS_SZ_MULT;

        self.drawing_area.set_content_height(image_height as i32);
        self.drawing_area.set_content_width(image_width as i32);
    }

    pub fn inc_zoom(&mut self, inc: f64) {
        const MAX_ZOOM: f64 = 25.0;
        const MIN_ZOOM: f64 = 0.1;

        self.zoom += inc;
        if(self.zoom > MAX_ZOOM) {
            self.zoom = MAX_ZOOM;
        } else if(self.zoom < MIN_ZOOM) {
            self.zoom = MIN_ZOOM;
        }
    }

    fn draw(&self, area: &DrawingArea, cr: &Context, area_width: i32, area_height: i32) {
        let img_width = self.image.pixels.len() as f64;
        let img_height = self.image.pixels[0].len() as f64;
        let x_offset = (area_width as f64 - img_width * self.zoom) / 2.0;
        let y_offset = (area_height as f64 - img_height * self.zoom) / 2.0;

        let image_surface_pattern = self.image.to_surface_pattern();
        let transparent_pattern = mk_transparent_pattern();

        cr.translate(x_offset as f64, y_offset as f64);
        cr.scale(self.zoom, self.zoom);
        cr.set_line_join(cairo::LineJoin::Bevel);

        const TRANSPARENT_CHECKER_SZ: f64 = 10.0;
        let trans_scale = TRANSPARENT_CHECKER_SZ / self.zoom;
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
        cr.set_line_width(BORDER_WIDTH / self.zoom);
        cr.stroke();
    }

    pub fn queue_redraw(&self) {
        self.drawing_area.queue_draw();
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
}
