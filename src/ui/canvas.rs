use super::super::image::{Image, mk_transparent_pattern};

use gtk::prelude::*;
use gtk::{Grid, Scrollbar, Orientation, Adjustment};
use gtk::gdk::{ModifierType};
use gtk::{DrawingArea, EventControllerScroll, EventControllerScrollFlags, EventControllerMotion};
use gtk::cairo::Context;
use gtk::cairo;
use gtk::glib::signal::Propagation;
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use gtk::glib::{SignalHandlerId};
use glib_macros::clone;

pub struct Canvas {
    image: Image,
    zoom: f64,
    pan: (f64, f64),
    cursor_pos: (f64, f64),
    drawing_area: DrawingArea,
    grid: Grid,
    v_scrollbar: Scrollbar,
    h_scrollbar: Scrollbar,
    scrollbar_update_handlers: Option<(SignalHandlerId, SignalHandlerId)>,
}

impl Canvas {
    pub fn new_p(image: Image) -> Rc<RefCell<Canvas>> {
        let grid = Grid::new();

        let drawing_area =  DrawingArea::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        grid.attach(&drawing_area, 0, 0, 1, 1);

        let v_scrollbar = Scrollbar::new(Orientation::Vertical, Adjustment::NONE);
        let h_scrollbar = Scrollbar::new(Orientation::Horizontal, Adjustment::NONE);

        grid.attach(&v_scrollbar, 1, 0, 1, 1);
        grid.attach(&h_scrollbar, 0, 1, 1, 1);

        let state = Rc::new(RefCell::new(Canvas {
            image,
            zoom: 1.0,
            pan: (0.0, 0.0),
            cursor_pos: (0.0, 0.0),
            drawing_area,
            grid,
            v_scrollbar,
            h_scrollbar,
            scrollbar_update_handlers: None,
        }));

        state.borrow().drawing_area.set_draw_func(clone!(@strong state => move |area, cr, width, height| {
            state.borrow_mut().draw(area, cr, width, height);
        }));

        // mouse movement

        let motion_controller = EventControllerMotion::new();

        motion_controller.connect_motion(clone!(@strong state => move |_, x, y| {
            state.borrow_mut().update_cursor_pos(x, y);
        }));

        state.borrow_mut().grid.add_controller(motion_controller);

        // scroll

        let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::BOTH_AXES);
        scroll_controller.connect_scroll(clone!(@strong state => move |ecs, dx, dy| {
            state.borrow_mut().handle_scroll(ecs, dx, dy)
        }));

        state.borrow_mut().grid.add_controller(scroll_controller);

        let h_sb_handler = state.borrow_mut().h_scrollbar.adjustment().connect_value_changed(clone!(@strong state => move |adjustment| {
            state.borrow_mut().set_h_pan(adjustment.value());
            state.borrow_mut().update();
        }));

        let v_sb_handler = state.borrow_mut().v_scrollbar.adjustment().connect_value_changed(clone!(@strong state => move |adjustment| {
            state.borrow_mut().set_v_pan(adjustment.value());
            state.borrow_mut().update();
        }));

        state.borrow_mut().scrollbar_update_handlers = Some((h_sb_handler, v_sb_handler));

        state.borrow_mut().drawing_area.connect_resize(clone!(@strong state => move |_, _, _| {
            state.borrow_mut().update();
        }));

        state.borrow_mut().update_scrollbars();

        state
    }

    pub fn widget(&self) -> &Grid {
        &self.grid
    }

    pub fn drawing_area(&self) -> &DrawingArea {
        &self.drawing_area
    }

    pub fn cursor_pos(&self) -> &(f64, f64) {
        &self.cursor_pos
    }

    // give the cursor_pos in terms of pixels in the image
    pub fn cursor_pos_pix(&self) -> (f64, f64) {
        let area_width = self.drawing_area.width();
        let area_height = self.drawing_area.height();
        let img_width = self.image.width() as f64;
        let img_height = self.image.height() as f64;
        let x_offset = (area_width as f64 - img_width * self.zoom) / 2.0;
        let y_offset = (area_height as f64 - img_height * self.zoom) / 2.0;

        let (x, y) = self.cursor_pos;

        let (top_left_x, top_left_y) = (x_offset + self.zoom * self.pan.0,
                                        y_offset + self.zoom * self.pan.1);

        ((x - top_left_x) / self.zoom,
         (y - top_left_y) / self.zoom)
    }

    pub fn inc_zoom(&mut self, inc: f64) {
        const MAX_ZOOM: f64 = 500.0;
        const MIN_ZOOM: f64 = 0.1;
        const ZOOM_FACTOR: f64 = 0.2;

        self.zoom += inc * self.zoom * ZOOM_FACTOR;
        if self.zoom > MAX_ZOOM {
            self.zoom = MAX_ZOOM;
        } else if self.zoom < MIN_ZOOM {
            self.zoom = MIN_ZOOM;
        }
    }

    fn update_cursor_pos(&mut self, x: f64, y: f64) {
        self.cursor_pos = (x, y);
    }

    fn inc_zoom_around_cursor(&mut self, inc: f64) {
        let old_zoom = self.zoom;

        self.inc_zoom(inc);

        let canvas_height = self.drawing_area.height() as f64;
        let canvas_width = self.drawing_area.width() as f64;
        let (cursor_x, cursor_y) = self.cursor_pos;
        let target_x = (canvas_width / 2.0) - cursor_x;
        let target_y = (canvas_height / 2.0) - cursor_y;

        self.pan.0 += target_x / old_zoom - target_x / self.zoom;
        self.pan.1 += target_y / old_zoom - target_y / self.zoom;
        self.clamp_pan();
    }

    pub fn inc_pan(&mut self, dx: f64, dy: f64) {
        const PAN_FACTOR: f64 = 50.0;

        self.pan = (self.pan.0 + dx / self.zoom * PAN_FACTOR,
                    self.pan.1 + dy / self.zoom * PAN_FACTOR);

        self.clamp_pan();
    }

    fn set_h_pan(&mut self, val: f64) {
        let h_window = self.drawing_area.width() as f64 / self.zoom;
        self.pan.0 = -val - h_window / 2.0;
        self.clamp_pan();
    }

    fn set_v_pan(&mut self, val: f64) {
        let v_window = self.drawing_area.height() as f64 / self.zoom;
        self.pan.1 = -val - v_window / 2.0;
        self.clamp_pan();
    }

    fn clamp_pan(&mut self) {
        let (max_x, max_y) = self.get_max_pan();
        if self.pan.0 < -max_x {
            self.pan.0 = -max_x;
        } else if self.pan.0 > max_x {
            self.pan.0 = max_x;
        }

        if self.pan.1 < -max_y {
            self.pan.1 = -max_y;
        } else if self.pan.1 > max_y {
            self.pan.1 = max_y;
        }
    }

    fn get_max_pan(&self) -> (f64, f64) {
        let img_width = self.image.width() as f64;
        let img_height = self.image.height() as f64;

        let win_width = self.drawing_area.width() as f64;
        let win_height = self.drawing_area.height() as f64;

        const FREE_SLACK: f64 = 300.0;

        let x_slack = FREE_SLACK + self.zoom * img_width - win_width;
        let y_slack = FREE_SLACK + self.zoom * img_height - win_height;

        (x_slack.max(0.0) / 2.0 / self.zoom, y_slack.max(0.0) / 2.0 / self.zoom)
    }


    fn draw(&mut self, _drawing_area: &DrawingArea, cr: &Context, area_width: i32, area_height: i32) {
        let img_width = self.image.width() as f64;
        let img_height = self.image.height() as f64;
        let x_offset = (area_width as f64 - img_width * self.zoom) / 2.0;
        let y_offset = (area_height as f64 - img_height * self.zoom) / 2.0;

        let image_surface_pattern = self.image.to_surface_pattern();
        let transparent_pattern = mk_transparent_pattern();

        cr.translate(x_offset as f64, y_offset as f64);
        cr.scale(self.zoom, self.zoom);
        cr.translate(self.pan.0, self.pan.1);
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

        const BORDER_WIDTH: f64 = 1.5;
        const DASH_LENGTH: f64 = 6.0;
        cr.set_line_width(BORDER_WIDTH / self.zoom);

        cr.set_dash(&[DASH_LENGTH / self.zoom, DASH_LENGTH / self.zoom], 0.0);
        cr.rectangle(0.0, 0.0, img_width, img_height);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.stroke();

        cr.set_dash(&[DASH_LENGTH / self.zoom, DASH_LENGTH / self.zoom], DASH_LENGTH / self.zoom);
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.stroke();
    }

    fn update_scrollbars(&mut self) {
        let v_window = self.drawing_area.height() as f64 / self.zoom;
        let h_window = self.drawing_area.width() as f64 / self.zoom;
        let (h_max, v_max) = self.get_max_pan();
        let h_max = h_max + h_window / 2.0;
        let v_max = v_max + v_window / 2.0;
        let h_value = -self.pan.0 - h_window / 2.0;
        let v_value = -self.pan.1 - v_window / 2.0;

        if let Some((ref h_sb_handler_id, ref v_sb_handler_id)) = self.scrollbar_update_handlers {
            self.h_scrollbar.adjustment().block_signal(h_sb_handler_id);
            self.v_scrollbar.adjustment().block_signal(v_sb_handler_id);
        }

        let h_adj = self.h_scrollbar.adjustment();
        let v_adj = self.v_scrollbar.adjustment();

        h_adj.set_lower(-h_max);
        h_adj.set_upper(h_max);
        h_adj.set_value(h_value);
        h_adj.set_page_size(h_window);

        v_adj.set_lower(-v_max);
        v_adj.set_upper(v_max);
        v_adj.set_value(v_value);
        v_adj.set_page_size(v_window);

        if let Some((ref h_sb_handler_id, ref v_sb_handler_id)) = self.scrollbar_update_handlers {
            self.h_scrollbar.adjustment().unblock_signal(h_sb_handler_id);
            self.v_scrollbar.adjustment().unblock_signal(v_sb_handler_id);
        }
    }

    pub fn update(&mut self) {
        self.update_scrollbars();
        self.drawing_area.queue_draw();
    }

    fn handle_scroll(&mut self, event_controller: &EventControllerScroll, dx: f64, dy: f64) -> Propagation {
        match event_controller.current_event_state() {
            ModifierType::CONTROL_MASK => self.inc_zoom_around_cursor(-dy),
            ModifierType::SHIFT_MASK => self.inc_pan(-dy, -dx),
            _ => self.inc_pan(-dx, -dy),
        }

        self.update();

        Propagation::Stop
    }

    pub fn image(&mut self) -> &mut Image {
        &mut self.image
        // TODO handle undo
    }
}
