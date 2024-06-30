use crate::image::undo::action::{DoableAction, UndoableAction};

use super::super::image::{Image, FusedImage, DrawableImage, mk_transparent_checkerboard};
use super::super::image::bitmask::DeletePix;
use super::super::image::undo::{ImageHistory, action::ActionName};
use super::super::image::resize::Crop;
use super::selection::Selection;
use super::UiState;
use super::toolbar::Toolbar;
use super::toolbar::mode::MouseMode;
use crate::image::{ImageLike, blend::BlendingMode};

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
use std::collections::HashMap;

pub struct Canvas {
    image_hist: ImageHistory,
    zoom: f64,
    pan: (f64, f64),
    /// The cursor's position, in terms of the ui,
    /// NOT THE IMAGE (use `*_pos_pix` for pixel-relative coords)
    cursor_pos: (f64, f64),
    drawing_area: DrawingArea,
    grid: Grid,
    pub selection: Selection,
    v_scrollbar: Scrollbar,
    h_scrollbar: Scrollbar,
    scrollbar_update_handlers: Option<(SignalHandlerId, SignalHandlerId)>,
    single_shot_draw_hooks: Vec<Box<dyn Fn(&Context)>>,
    draw_hook: Option<Box<dyn Fn(&Context)>>,
    transparent_checkerboard: Rc<RefCell<DrawableImage>>,
    ui_p: Rc<RefCell<UiState>>,
    /// Mapping from state ids to the cursor positions
    /// *at time of the respective commit's completion*
    /// (position is relative to the image's pixel coords)
    history_id_to_cursor_pos_pix: HashMap<usize, (f64, f64)>,
    /// A mask corresponding to the flattened image pixel
    /// vector: `pencil_mask[i]` == `pencil_mask_counter`
    /// iff the pixel at index `i` has been drawn on
    /// during the current pencil stroke
    pencil_mask: Vec<usize>,
    pencil_mask_counter: usize,
}

impl Canvas {
    pub fn new_p(ui_p: &Rc<RefCell<UiState>>, image: FusedImage) -> Rc<RefCell<Canvas>> {
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

        let image_net_size = image.height() as usize * image.width() as usize;

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
            transparent_checkerboard: Rc::new(RefCell::new(mk_transparent_checkerboard())),
            ui_p: ui_p.clone(),
            history_id_to_cursor_pos_pix: HashMap::new(),
            pencil_mask: vec![0; image_net_size],
            pencil_mask_counter: 1,
        }));

        let mod_hist = Rc::new(clone!(@strong canvas_p => move |f: Box<dyn Fn(&mut ImageHistory)>| {
            f(&mut canvas_p.borrow_mut().image_hist);
        }));

        let update_canvas = Rc::new(clone!(@strong canvas_p => move || {
            canvas_p.borrow_mut().update_after_undo_or_redo();
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
            canvas_p.borrow_mut().draw_selection_outline(cr);

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
        // left click drag

        let left_drag_controller = GestureDrag::builder()
            .button(1) // left click
            .build();
        left_drag_controller.connect_begin(clone!(@strong ui_p, @strong canvas_p => move |dc, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_drag_start(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        left_drag_controller.connect_drag_update(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_drag_update(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        left_drag_controller.connect_drag_end(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_drag_end(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        canvas_p.borrow().drawing_area().add_controller(left_drag_controller);

        // right click drag

        let right_drag_controller = GestureDrag::builder()
            .button(3) // right click
            .build();
        right_drag_controller.connect_begin(clone!(@strong ui_p, @strong canvas_p => move |dc, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_right_drag_start(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        right_drag_controller.connect_drag_update(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_right_drag_update(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        right_drag_controller.connect_drag_end(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mut mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.handle_right_drag_end(&dc.current_event_state(), &mut canvas_p.borrow_mut(), &mut toolbar);
            toolbar.set_mouse_mode(mouse_mode);
        }));

        canvas_p.borrow().drawing_area().add_controller(right_drag_controller);

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
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.draw(&canvas_p.borrow(), cr, &mut toolbar);
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

    /// Get the cursor's position, in terms of the ui,
    /// NOT THE IMAGE (use `*_pos_pix` for pixel-relative coords)
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

    /// Get the cursor_pos in terms of pixels in the image
    pub fn cursor_pos_pix_f(&self) -> (f64, f64) {
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

    pub fn cursor_pos_pix_u(&self) -> (usize, usize) {
        let (x, y) = self.cursor_pos_pix_f();
        (x.floor() as usize, y.floor() as usize)
    }

    pub fn cursor_pos_pix_i(&self) -> (i32, i32) {
        let (x, y) = self.cursor_pos_pix_f();
        (x.floor() as i32, y.floor() as i32)
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

    pub fn transparent_checkerboard_pattern(&self) -> gtk::cairo::SurfacePattern {
        self.transparent_checkerboard.borrow_mut().to_repeated_surface_pattern()
    }

    fn draw(&mut self, _drawing_area: &DrawingArea, cr: &Context, area_width: i32, area_height: i32) {
        let img_width = self.image_width() as f64;
        let img_height = self.image_height() as f64;
        let x_offset = (area_width as f64 - img_width * self.zoom) / 2.0;
        let y_offset = (area_height as f64 - img_height * self.zoom) / 2.0;

        let image_surface_pattern = self.image_hist.now_mut().drawable().to_surface_pattern();
        let transparent_pattern = self.transparent_checkerboard.borrow_mut().to_repeated_surface_pattern();

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
        let scale = (area_width as f64 - 0.1) / img_width as f64;

        let image_surface_pattern = self.image_hist.now_mut().drawable().to_surface_pattern();
        let transparent_pattern = self.transparent_checkerboard.borrow_mut().to_repeated_surface_pattern();

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

        const BORDER_WIDTH: f64 = 1.5;
        cr.set_line_width(BORDER_WIDTH / scale);

        cr.rectangle(0.0, 0.0, img_width, img_height);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let _ = cr.stroke();
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

    pub fn update_after_undo_or_redo(&mut self) {
        let last_cursor_pos_pix = self.last_cursor_pos_pix();
        if let MouseMode::Pencil(ref mut pencil_state) = self.ui_p.borrow_mut()
                .toolbar_p.borrow_mut().mouse_mode_mut() {
            pencil_state.set_last_cursor_pos_pix(last_cursor_pos_pix)
        }

        self.update();
    }

    pub fn update(&mut self) {
        self.update_scrollbars();
        self.validate_selection();
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

    pub fn image(&mut self) -> &mut FusedImage {
        self.image_hist.now_mut()
    }

    pub fn undo_id(&self) -> usize {
        self.image_hist.now_id()
    }

    pub fn image_ref(&self) -> &FusedImage {
        self.image_hist.now()
    }

    pub fn image_image_ref(&self) -> &Image {
        self.image_hist.now().image()
    }

    pub fn image_height(&self) -> i32 {
        self.image_hist.now().height()
    }

    pub fn image_width(&self) -> i32 {
        self.image_hist.now().width()
    }

    pub fn undo(&mut self) {
        self.image_hist.undo();
        self.update_after_undo_or_redo();
    }

    pub fn redo(&mut self) {
        self.image_hist.redo();
        self.update_after_undo_or_redo();
    }

    fn save_cursor_pos_after_history_commit(&mut self) {
        let history_id = self.image_hist.now_id();
        self.history_id_to_cursor_pos_pix.insert(
            history_id,
            self.cursor_pos_pix_f(),
        );
    }

    pub fn save_state_for_undo(&mut self, culprit: ActionName) {
        self.image_hist.push_current_state(culprit);
        self.save_cursor_pos_after_history_commit();
    }

    pub fn exec_doable_action<A>(&mut self, action: A)
    where
        A: DoableAction,
    {
        self.image_hist.exec_doable_action(action);
        self.save_cursor_pos_after_history_commit();
        self.update();
    }

    pub fn exec_undoable_action(&mut self, action: Box<dyn UndoableAction>) {
        self.image_hist.exec_undoable_action(action);
        self.save_cursor_pos_after_history_commit();
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

    pub fn delete_selection(&mut self) {
        let action = DeletePix::new(self.selection.iter());
        self.image_hist.exec_doable_action(action);
        self.selection = Selection::NoSelection;
        self.save_cursor_pos_after_history_commit();
        self.update();
    }

    /// If `self.selection` is out of bounds/invalid, unselect it
    fn validate_selection(&mut self) {
        let is_valid = match self.selection {
            Selection::NoSelection => true,
            Selection::Rectangle(x, y, w, h) => {
                x + w < self.image_width() as usize &&
                y + h < self.image_height() as usize
            },
            Selection::Bitmask(ref bitmask) => {
                bitmask.width() == self.image_width() as usize &&
                bitmask.height() == self.image_height() as usize
            },
        };

        if !is_valid {
            self.selection = Selection::NoSelection;
        }
    }

    /// Resize `self.pencil_mask` if it's become too small
    fn validate_pencil_mask(&mut self) {
        let image_net_size = self.image().width() * self.image().height();
        let deficit = image_net_size - self.pencil_mask.len() as i32;
        if deficit > 0 {
            self.pencil_mask.append(&mut vec![0; deficit as usize]);
        }
    }

    pub fn last_cursor_pos_pix(&self) -> (f64, f64) {
        let hist_id = self.image_hist.now_id();
        *self.history_id_to_cursor_pos_pix.get(&hist_id)
            .unwrap_or(&(0.0, 0.0))
    }

    fn test_pencil_mask_at(&mut self, r: usize, c: usize) -> bool {
        self.validate_pencil_mask();
        let w = self.image().width() as usize;

        self.pencil_mask[r * w + c] == self.pencil_mask_counter
    }

    fn set_pencil_mask_at(&mut self, r: usize, c: usize) {
        self.validate_pencil_mask();
        let w = self.image().width() as usize;
        self.pencil_mask[r * w + c] = self.pencil_mask_counter;
    }

    pub fn clear_pencil_mask(&mut self) {
        self.pencil_mask_counter += 1
    }

    /// Draws `other` onto self.image() at (x, y),
    /// setting the pencil mask at the drawn pixels,
    /// and not drawing any pixels that are aready
    /// set in the mask
    pub fn sample_image_respecting_pencil_mask(
        &mut self,
        other: &impl ImageLike,
        blending_mode: &BlendingMode,
        x: i32,
        y: i32
    ) {
        for i in 0..other.height() {
            for j in 0..other.width() {
                let ip = i as i32 + y;
                let jp = j as i32 + x;

                if ip < 0 || jp < 0 || ip >= self.image().height() || jp >= self.image().width() ||
                   self.test_pencil_mask_at(ip as usize, jp as usize) {
                    continue;
                }

                let p = self.image().pix_at_mut(ip, jp);
                let mut success = false;

                if let Some(op) = other.try_pix_at(i as usize, j as usize) {
                    *p = blending_mode.blend(op, &p);
                    success = true;
                }

                if success {
                    self.set_pencil_mask_at(ip as usize, jp as usize);
                }
            }
        }
    }
}
