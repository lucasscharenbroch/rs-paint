use crate::image::undo::action::{DoableAction, UndoableAction};

use super::super::image::{Image, UnifiedImage, DrawableImage, mk_transparent_checkerboard};
use super::super::image::undo::{ImageHistory, action::ActionName};
use super::super::image::resize::Crop;
use super::selection::Selection;
use super::UiState;
use super::toolbar::Toolbar;

use gtk::{prelude::*, Widget};
use gtk::{Grid, Scrollbar, Orientation, Adjustment};
use gtk::gdk::ModifierType;
use gtk::{DrawingArea, EventControllerScroll, EventControllerScrollFlags, GestureDrag, EventControllerMotion};
use gtk::cairo::Context;
use gtk::cairo;
use gtk::glib::signal::Propagation;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::glib::SignalHandlerId;
use glib_macros::clone;

pub struct Canvas {
    image_hist: ImageHistory,
    zoom: f64,
    pan: (f64, f64),
    cursor_pos: (f64, f64),
    drawing_area: DrawingArea,
    grid: Grid,
    selection: Selection,
    v_scrollbar: Scrollbar,
    h_scrollbar: Scrollbar,
    scrollbar_update_handlers: Option<(SignalHandlerId, SignalHandlerId)>,
    single_shot_draw_hooks: Vec<Box<dyn Fn(&Context)>>,
    draw_hook: Option<Box<dyn Fn(&Context)>>,
    transparent_checkerboard: DrawableImage,
    ui_p: Rc<RefCell<UiState>>,
}

impl Canvas {
    pub fn new_p(ui_p: &Rc<RefCell<UiState>>, image: UnifiedImage) -> Rc<RefCell<Canvas>> {
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

        let canvas_p = Rc::new(RefCell::new(Canvas {
            image_hist: ImageHistory::new(image),
            zoom: 1.0,
            pan: (0.0, 0.0),
            cursor_pos: (0.0, 0.0),
            drawing_area,
            grid,
            selection: Selection::NoSelection,
            v_scrollbar,
            h_scrollbar,
            scrollbar_update_handlers: None,
            single_shot_draw_hooks: vec![],
            draw_hook: None,
            transparent_checkerboard: mk_transparent_checkerboard(),
            ui_p: ui_p.clone(),
        }));

        let mod_hist = Rc::new(clone!(@strong canvas_p => move |f: Box<dyn Fn(&mut ImageHistory)>| {
            f(&mut canvas_p.borrow_mut().image_hist);
        }));

        let update_canvas = Rc::new(clone!(@strong canvas_p => move || {
            canvas_p.borrow_mut().update();
        }));

        canvas_p.borrow_mut().image_hist.set_hooks(mod_hist, update_canvas);

        Self::init_internal_connections(&canvas_p);
        Self::init_ui_state_connections(&canvas_p, &ui_p);

        canvas_p
    }

    fn init_internal_connections(canvas_p: &Rc<RefCell<Self>>) {
        // drawing area draw-function

        canvas_p.borrow().drawing_area.set_draw_func(clone!(@strong canvas_p => move |area, cr, width, height| {
            canvas_p.borrow_mut().draw(area, cr, width, height);

            // draw selection
            canvas_p.borrow().selection.draw_outline(&canvas_p.borrow(), cr);

            // run hooks
            canvas_p.borrow().draw_hook.iter().for_each(|f| f(cr));
            canvas_p.borrow().single_shot_draw_hooks.iter().for_each(|f| f(cr));
            canvas_p.borrow_mut().single_shot_draw_hooks = vec![];
        }));

        // scroll

        let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::BOTH_AXES);
        scroll_controller.connect_scroll(clone!(@strong canvas_p => move |ecs, dx, dy| {
            canvas_p.borrow_mut().handle_scroll(ecs, dx, dy)
        }));

        canvas_p.borrow_mut().grid.add_controller(scroll_controller);

        let h_sb_handler = canvas_p.borrow_mut().h_scrollbar.adjustment().connect_value_changed(clone!(@strong canvas_p => move |adjustment| {
            canvas_p.borrow_mut().set_h_pan(adjustment.value());
            canvas_p.borrow_mut().update();
        }));

        let v_sb_handler = canvas_p.borrow_mut().v_scrollbar.adjustment().connect_value_changed(clone!(@strong canvas_p => move |adjustment| {
            canvas_p.borrow_mut().set_v_pan(adjustment.value());
            canvas_p.borrow_mut().update();
        }));

        canvas_p.borrow_mut().scrollbar_update_handlers = Some((h_sb_handler, v_sb_handler));

        canvas_p.borrow_mut().drawing_area.connect_resize(clone!(@strong canvas_p => move |_, _, _| {
            canvas_p.borrow_mut().update();
        }));

        canvas_p.borrow_mut().update_scrollbars();
    }

    fn init_ui_state_connections(canvas_p: &Rc<RefCell<Self>>, ui_p: &Rc<RefCell<UiState>>) {
        // drag

        let drag_controller = GestureDrag::new();
        drag_controller.connect_begin(clone!(@strong ui_p, @strong canvas_p => move |dc, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_drag_start(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        drag_controller.connect_drag_update(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_drag_update(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        drag_controller.connect_drag_end(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_drag_end(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        canvas_p.borrow().drawing_area().add_controller(drag_controller);

        // mouse movement

        let motion_controller = EventControllerMotion::new();

        motion_controller.connect_motion(clone!(@strong ui_p, @strong canvas_p => move |ecm, x, y| {
            canvas_p.borrow_mut().update_cursor_pos(x, y);
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_motion(&ecm.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        canvas_p.borrow().drawing_area().add_controller(motion_controller);

        // drawing

        canvas_p.borrow_mut().set_draw_hook(Box::new(clone!(@strong ui_p, @strong canvas_p => move |cr| {
            ui_p.borrow().toolbar_p.borrow_mut().mouse_mode().draw(&canvas_p.borrow(), cr);
        })));

        // mouse-mode-change

        ui_p.borrow_mut().toolbar_p.borrow_mut().set_mode_change_hook(Box::new(clone!(@strong ui_p, @strong canvas_p => move |_toolbar: &Toolbar| {
            canvas_p.borrow_mut().update();
        })));
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

    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn zoom(&self) -> &f64 {
        &self.zoom
    }

    // give the cursor_pos in terms of pixels in the image
    pub fn cursor_pos_pix(&self) -> (f64, f64) {
        let area_width = self.drawing_area.width();
        let area_height = self.drawing_area.height();
        let img_width = self.image_width() as f64;
        let img_height = self.image_height() as f64;
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

    pub fn update_cursor_pos(&mut self, x: f64, y: f64) {
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
        let img_width = self.image_width() as f64;
        let img_height = self.image_height() as f64;

        let win_width = self.drawing_area.width() as f64;
        let win_height = self.drawing_area.height() as f64;

        const FREE_SLACK: f64 = 300.0;

        let x_slack = FREE_SLACK + self.zoom * img_width - win_width;
        let y_slack = FREE_SLACK + self.zoom * img_height - win_height;

        (x_slack.max(0.0) / 2.0 / self.zoom, y_slack.max(0.0) / 2.0 / self.zoom)
    }


    fn draw(&mut self, _drawing_area: &DrawingArea, cr: &Context, area_width: i32, area_height: i32) {
        let img_width = self.image_width() as f64;
        let img_height = self.image_height() as f64;
        let x_offset = (area_width as f64 - img_width * self.zoom) / 2.0;
        let y_offset = (area_height as f64 - img_height * self.zoom) / 2.0;

        let image_surface_pattern = self.image_hist.now_mut().drawable().to_surface_pattern();
        let transparent_pattern = self.transparent_checkerboard.to_repeated_surface_pattern();

        cr.translate(x_offset as f64, y_offset as f64);
        cr.scale(self.zoom, self.zoom);
        cr.translate(self.pan.0, self.pan.1);
        cr.set_line_join(cairo::LineJoin::Bevel);

        const TRANSPARENT_CHECKER_SZ: f64 = 10.0;
        let trans_scale = TRANSPARENT_CHECKER_SZ / self.zoom;
        cr.scale(trans_scale, trans_scale);
        cr.rectangle(0.0, 0.0, img_width / trans_scale, img_height / trans_scale);
        let _ = cr.set_source(transparent_pattern);
        let _ = cr.fill();
        cr.scale(1.0 / trans_scale, 1.0 / trans_scale);

        cr.rectangle(0.0, 0.0, img_width, img_height);
        let _ = cr.set_source(image_surface_pattern);
        let _ = cr.fill();

        const BORDER_WIDTH: f64 = 1.5;
        const DASH_LENGTH: f64 = 6.0;
        cr.set_line_width(BORDER_WIDTH / self.zoom);

        cr.set_dash(&[DASH_LENGTH / self.zoom, DASH_LENGTH / self.zoom], 0.0);
        cr.rectangle(0.0, 0.0, img_width, img_height);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let _ = cr.stroke();

        cr.set_dash(&[DASH_LENGTH / self.zoom, DASH_LENGTH / self.zoom], DASH_LENGTH / self.zoom);
        cr.set_source_rgb(1.0, 1.0, 1.0);
        let _ = cr.stroke();

        cr.set_dash(&[], 0.0);
    }

    pub fn draw_thumbnail(&mut self, _drawing_area: &DrawingArea, cr: &Context, area_width: i32, _area_height: i32) {
        let img_width = self.image_width() as f64;
        let img_height = self.image_height() as f64;
        let scale = area_width as f64 / img_width as f64;

        let image_surface_pattern = self.image_hist.now_mut().drawable().to_surface_pattern();
        let transparent_pattern = self.transparent_checkerboard.to_repeated_surface_pattern();

        cr.scale(scale, scale);

        const TRANSPARENT_CHECKER_SZ: f64 = 10.0;
        let trans_scale = TRANSPARENT_CHECKER_SZ / scale;
        cr.scale(trans_scale, trans_scale);
        cr.rectangle(0.0, 0.0, img_width / trans_scale, img_height / trans_scale);
        let _ = cr.set_source(transparent_pattern);
        let _ = cr.fill();
        cr.scale(1.0 / trans_scale, 1.0 / trans_scale);


        cr.rectangle(0.0, 0.0, img_width, img_height);
        let _ = cr.set_source(image_surface_pattern);
        let _ = cr.fill();

        /* TODO border?
        const BORDER_WIDTH: f64 = 1.5;
        cr.set_line_width(BORDER_WIDTH / scale);

        cr.rectangle(0.0, 0.0, img_width, img_height);
        cr.set_source_rgb(1.0, 1.0, 0.0);
        cr.stroke();
        */
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
        if let Ok(ui) = self.ui_p.try_borrow() {
            if let Some(tab) = ui.active_tab() {
                tab.redraw_thumbnail();
            }
        }
    }

    pub fn update_with(&mut self, draw_hook: Box<dyn Fn(&Context)>) {
        self.single_shot_draw_hooks.push(draw_hook);
        self.update();
    }

    pub fn set_draw_hook(&mut self, draw_hook: Box<dyn Fn(&Context)>) {
        self.draw_hook = Some(draw_hook);
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

    pub fn image(&mut self) -> &mut UnifiedImage {
        self.image_hist.now_mut()
    }

    pub fn undo_id(&self) -> usize {
        self.image_hist.now_id()
    }

    pub fn image_ref(&self) -> &UnifiedImage {
        self.image_hist.now()
    }

    pub fn image_height(&self) -> i32 {
        self.image_hist.now().height()
    }

    pub fn image_width(&self) -> i32 {
        self.image_hist.now().width()
    }

    pub fn undo(&mut self) {
        self.image_hist.undo();
    }

    pub fn redo(&mut self) {
        self.image_hist.redo();
    }

    pub fn save_state_for_undo(&mut self, culprit: ActionName) {
        self.image_hist.push_current_state(culprit);
    }

    pub fn exec_doable_action(&mut self, action: &impl DoableAction) {
        self.image_hist.exec_doable_action(action);
        self.update();
    }

    pub fn exec_undoable_action(&mut self, action: Box<dyn UndoableAction>) {
        self.image_hist.exec_undoable_action(action);
        self.update();
    }

    pub fn history_widget(&self) -> &impl IsA<Widget> {
        self.image_hist.widget_scrolled_to_active_commit()
    }

    pub fn crop_to(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let img_width = self.image().width() as usize;
        let img_height = self.image().height() as usize;

        if w == 0 || h == 0 || x + w >= img_width || y + h >= img_height {
            panic!("Out of bounds crop: x={x} y={y} w={w} h={h} img_width={img_width} img_height={img_height}");
        }

        let crop = Crop::new(x, y, w, h);
        self.exec_undoable_action(Box::new(crop));
    }
}
