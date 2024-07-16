use crate::image::undo::action::{AutoDiffAction, MultiLayerAction, SingleLayerAction};
use crate::image::{ImageLikeUnchecked, LayerIndex, Pixel};
use crate::transformable::{Transformable, SampleableCommit, TransformableImage};
use crate::geometry::matrix_width_height;

use super::super::image::{Image, FusedLayeredImage, TrackedLayeredImage, DrawableImage, mk_transparent_checkerboard};
use super::super::image::bitmask::DeletePix;
use super::super::image::undo::{ImageHistory, action::ActionName};
use super::super::image::resize::Crop;
use super::selection::Selection;
use super::tab::Tab;
use super::UiState;
use super::toolbar::Toolbar;
use super::toolbar::mode::{CursorState, FreeTransformState, MouseMode, TransformationSelection};
use crate::image::{ImageLike, blend::BlendingMode};
use super::layer_window::LayerWindow;
use super::dialog::modal_ok_dialog_str;

use gtk::prelude::*;
use gtk::gdk::{ModifierType, RGBA};
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
    drawing_area: gtk::DrawingArea,
    grid: gtk::Grid,
    selection: Selection,
    v_scrollbar: gtk::Scrollbar,
    h_scrollbar: gtk::Scrollbar,
    scrollbar_update_handlers: Option<(SignalHandlerId, SignalHandlerId)>,
    single_shot_draw_hooks: Vec<Box<dyn Fn(&cairo::Context)>>,
    draw_hook: Option<Box<dyn Fn(&cairo::Context)>>,
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
    layer_window_p: Rc<RefCell<LayerWindow>>,
    lock_dialog_open: Rc<RefCell<bool>>,
    tab_thumbnail_p: Option<Rc<RefCell<gtk::DrawingArea>>>,
    transformation_selection: RefCell<Option<TransformationSelection>>,
}

macro_rules! run_lockable_mouse_mode_hook {
    ($ui_p:expr, $canvas_p:expr, $controller:expr, $hook_name:ident) => {
        let ui = $ui_p.borrow();
        let mut toolbar = ui.toolbar_p.borrow_mut();
        let mut mouse_mode = toolbar.mouse_mode().clone();

        if mouse_mode.disable_when_locked() {
            if $canvas_p.borrow().active_layer_locked() {
                std::mem::drop(toolbar); // prevent a double borrow-mut (from the above)
                $canvas_p.borrow().alert_user_of_lock("Can't modify: active layer locked");
                return;
            }
        }

        mouse_mode.$hook_name(&$controller.current_event_state(), &mut $canvas_p.borrow_mut(), &mut toolbar);
        toolbar.set_mouse_mode(mouse_mode.updated_after_hook());
    };
}

macro_rules! run_non_lockable_mouse_mode_hook {
    ($ui_p:expr, $canvas_p:expr, $controller:expr, $hook_name:ident) => {
        let ui = $ui_p.borrow();
        let mut toolbar = ui.toolbar_p.borrow_mut();
        let mut mouse_mode = toolbar.mouse_mode().clone();
        mouse_mode.$hook_name(&$controller.current_event_state(), &mut $canvas_p.borrow_mut(), &mut toolbar);
        toolbar.set_mouse_mode(mouse_mode.updated_after_hook());
    };
}

impl Canvas {
    pub fn new_p(ui_p: &Rc<RefCell<UiState>>, layered_image: FusedLayeredImage) -> Rc<RefCell<Canvas>> {
        let grid = gtk::Grid::new();

        let drawing_area =  gtk::DrawingArea::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        grid.attach(&drawing_area, 0, 0, 1, 1);

        let v_scrollbar = gtk::Scrollbar::new(gtk::Orientation::Vertical, gtk::Adjustment::NONE);
        let h_scrollbar = gtk::Scrollbar::new(gtk::Orientation::Horizontal, gtk::Adjustment::NONE);

        grid.attach(&v_scrollbar, 1, 0, 1, 1);
        grid.attach(&h_scrollbar, 0, 1, 1, 1);

        let image_net_size = layered_image.height() as usize * layered_image.width() as usize;

        let canvas_p = Rc::new(RefCell::new(Canvas {
            image_hist: ImageHistory::new(layered_image),
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
            layer_window_p: Rc::new(RefCell::new(LayerWindow::new())),
            lock_dialog_open: Rc::new(RefCell::new(false)),
            tab_thumbnail_p: None,
            transformation_selection: RefCell::new(None),
        }));

        let mod_hist = Rc::new(clone!(@strong canvas_p => move |f: Box<dyn Fn(&mut ImageHistory)>| {
            f(&mut canvas_p.borrow_mut().image_hist);
        }));

        let update_canvas = Rc::new(clone!(@strong canvas_p => move || {
            canvas_p.borrow_mut().update_after_undo_or_redo();
        }));

        canvas_p.borrow_mut().image_hist.set_hooks(mod_hist, update_canvas);

        canvas_p.borrow().layer_window_p.borrow_mut().finish_init(canvas_p.clone());

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

        let scroll_controller = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
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

    pub fn layer_at_index_is_locked(&self, layer_index: LayerIndex) -> bool {
        self.image_hist.now().layer_at_index(layer_index).is_locked()
    }

    pub fn active_layer_locked(&self) -> bool {
        self.image_hist.now().active_layer().is_locked()
    }

    /// Opens a dialog, but only if there isn't already
    /// an open one
    fn alert_user_of_lock(&self, message: &str) {
        if *self.lock_dialog_open.borrow() {
            return;
        }

        *self.lock_dialog_open.borrow_mut() = true;

        let lock_dialog_open = self.lock_dialog_open.clone();

        modal_ok_dialog_str(
            self.ui_p.borrow().window(),
            "Active Layer Locked",
            message,
            clone!(@strong lock_dialog_open => move || {
                *lock_dialog_open.borrow_mut() = false;
                super::dialog::CloseDialog::Yes
            }),
            clone!(@strong lock_dialog_open => move || {
                *lock_dialog_open.borrow_mut() = false;
            })
        );
    }

    fn init_ui_state_connections(canvas_p: &Rc<RefCell<Self>>, ui_p: &Rc<RefCell<UiState>>) {
        // left click drag

        let left_drag_controller = gtk::GestureDrag::builder()
            .button(1) // left click
            .build();
        left_drag_controller.connect_begin(clone!(@strong ui_p, @strong canvas_p => move |dc, _| {
            run_lockable_mouse_mode_hook!(ui_p, canvas_p, dc, handle_drag_start);
        }));

        left_drag_controller.connect_drag_update(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            run_lockable_mouse_mode_hook!(ui_p, canvas_p, dc, handle_drag_update);
        }));

        left_drag_controller.connect_drag_end(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            run_lockable_mouse_mode_hook!(ui_p, canvas_p, dc, handle_drag_end);
        }));

        canvas_p.borrow().drawing_area().add_controller(left_drag_controller);

        // right click drag

        let right_drag_controller = gtk::GestureDrag::builder()
            .button(3) // right click
            .build();
        right_drag_controller.connect_begin(clone!(@strong ui_p, @strong canvas_p => move |dc, _| {
            run_lockable_mouse_mode_hook!(ui_p, canvas_p, dc, handle_right_drag_start);
        }));

        right_drag_controller.connect_drag_update(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            run_lockable_mouse_mode_hook!(ui_p, canvas_p, dc, handle_right_drag_update);
        }));

        right_drag_controller.connect_drag_end(clone!(@strong ui_p, @strong canvas_p => move |dc, _, _| {
            run_lockable_mouse_mode_hook!(ui_p, canvas_p, dc, handle_right_drag_end);
        }));

        canvas_p.borrow().drawing_area().add_controller(right_drag_controller);

        // mouse movement

        let motion_controller = gtk::EventControllerMotion::new();

        motion_controller.connect_motion(clone!(@strong ui_p, @strong canvas_p => move |ecm, x, y| {
            canvas_p.borrow_mut().update_cursor_pos(x, y);
            run_non_lockable_mouse_mode_hook!(ui_p, canvas_p, ecm, handle_motion);
        }));

        canvas_p.borrow().drawing_area().add_controller(motion_controller);

        // drawing

        canvas_p.borrow_mut().set_draw_hook(Box::new(clone!(@strong ui_p, @strong canvas_p => move |cr| {
            let ui = ui_p.borrow();
            let mut toolbar = ui.toolbar_p.borrow_mut();
            let mouse_mode = toolbar.mouse_mode().clone();
            mouse_mode.draw(&mut canvas_p.borrow(), cr, &mut toolbar);
        })));

        // mouse-mode-change

        ui_p.borrow_mut().toolbar_p.borrow_mut().set_mode_change_hook(Box::new(clone!(@strong ui_p, @strong canvas_p => move |_toolbar: &Toolbar| {
            canvas_p.borrow_mut().update();
        })));
    }

    pub fn widget(&self) -> &gtk::Grid {
        &self.grid
    }

    pub fn drawing_area(&self) -> &gtk::DrawingArea {
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

    pub fn cursor_pos_pix_f_rounded(&self) -> (f64, f64) {
        let (x, y) = self.cursor_pos_pix_f();
        (x.floor(), y.floor())
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

    pub fn transparent_checkerboard_pattern(&self) -> cairo::SurfacePattern {
        self.transparent_checkerboard.borrow_mut().to_repeated_surface_pattern()
    }

    fn draw(&mut self, _drawing_area: &gtk::DrawingArea, cr: &cairo::Context, area_width: i32, area_height: i32) {
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

    fn draw_thumbnail_helper(
        &mut self,
        scale: f64,
        cr: &cairo::Context,
        img_width: f64,
        img_height: f64,
        image_surface_pattern: gtk::cairo::SurfacePattern,
    ) {
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

    pub fn draw_thumbnail(&mut self, _drawing_area: &gtk::DrawingArea, cr: &cairo::Context, area_width: i32, _area_height: i32) {
        let img_width = self.image_width() as f64;
        let img_height = self.image_height() as f64;
        let scale = (area_width as f64 - 0.1) / img_width as f64;
        let image_surface_pattern = self.image_hist.now_mut().drawable().to_surface_pattern();

        self.draw_thumbnail_helper(scale, cr, img_width, img_height, image_surface_pattern);
    }

    pub fn draw_layer_thumbnail(
        &mut self,
        _drawing_area: &gtk::DrawingArea,
        cr: &cairo::Context,
        area_width: i32,
        _area_height: i32,
        layer_index: LayerIndex,
    ) {
        let img_width = self.image_width() as f64;
        let img_height = self.image_height() as f64;
        let scale = (area_width as f64 - 0.1) / img_width as f64;

        let image_surface_pattern = self.image_hist.now_mut()
            .layer_drawable(layer_index).to_surface_pattern();

        self.draw_thumbnail_helper(scale, cr, img_width, img_height, image_surface_pattern);
    }

    pub fn set_tab_thumbnail_p(&mut self, thumbnail_p: Rc<RefCell<gtk::DrawingArea>>) {
        self.tab_thumbnail_p = Some(thumbnail_p);
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

    fn update_tab_thumbnail_aspect_ratio(&self, aspect_ratio: f64) {
        if let Some(drawing_area_p) = self.tab_thumbnail_p.as_ref() {
            Tab::update_aspect_ratio(&drawing_area_p.borrow(), aspect_ratio)
        }
    }

    pub fn update(&mut self) {
        self.update_scrollbars();
        self.validate_selection();
        self.drawing_area.queue_draw();

        let aspect_ratio = self.image_width() as f64 /
            self.image_height() as f64;
        self.update_tab_thumbnail_aspect_ratio(aspect_ratio);
        self.layer_window_p.borrow().update(
            self.image_hist.now().num_layers(),
            self.image_hist.now().layer_propss(),
            *self.image_hist.now().active_layer_index(),
            aspect_ratio,
        );

        if let Ok(ui) = self.ui_p.try_borrow() {
            if let Some(tab) = ui.active_tab() {
                tab.redraw_thumbnail();
            }
        }
    }

    pub fn update_with(&mut self, draw_hook: Box<dyn Fn(&cairo::Context)>) {
        self.single_shot_draw_hooks.push(draw_hook);
        self.update();
    }

    pub fn set_draw_hook(&mut self, draw_hook: Box<dyn Fn(&cairo::Context)>) {
        self.draw_hook = Some(draw_hook);
    }

    fn handle_scroll(&mut self, event_controller: &gtk::EventControllerScroll, dx: f64, dy: f64) -> Propagation {
        match event_controller.current_event_state() {
            ModifierType::CONTROL_MASK => self.inc_zoom_around_cursor(-dy),
            ModifierType::SHIFT_MASK => self.inc_pan(-dy, -dx),
            _ => self.inc_pan(-dx, -dy),
        }

        self.update();

        Propagation::Stop
    }

    pub fn undo_id(&self) -> usize {
        self.image_hist.now_id()
    }

    pub fn layered_image(&self) -> &FusedLayeredImage {
        self.image_hist.now()
    }

    pub fn active_image(&self) -> &Image {
        self.image_hist.now().active_image()
    }

    pub fn active_image_mut(&mut self) -> &mut impl TrackedLayeredImage {
        self.image_hist.now_mut()
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

    pub fn commit_changes(&mut self, culprit: ActionName) {
        self.image_hist.push_current_state(culprit);
        self.save_cursor_pos_after_history_commit();
    }

    pub fn exec_auto_diff_action<A>(&mut self, action: A)
    where
        A: AutoDiffAction,
    {
        self.image_hist.exec_doable_action(action);
        self.save_cursor_pos_after_history_commit();
        self.update();
    }

    pub fn exec_undoable_action(&mut self, action: Box<dyn SingleLayerAction<Image>>) {
        self.image_hist.exec_undoable_action(action);
        self.save_cursor_pos_after_history_commit();
        self.update();
    }

    pub fn exec_multi_undoable_action<D: 'static>(&mut self, action: Box<dyn MultiLayerAction<LayerData = D>>) {
        self.image_hist.exec_multi_undoable_action(action);
        self.save_cursor_pos_after_history_commit();
        self.update();
    }


    pub fn history_widget(&self) -> &impl IsA<gtk::Widget> {
        self.image_hist.widget_scrolled_to_active_commit()
    }

    pub fn layers_widget(&self) -> impl IsA<gtk::Widget> {
        let x = self.layer_window_p.borrow();
        x.widget()
    }

    pub fn crop_to(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let img_width = self.image_width() as usize;
        let img_height = self.image_height() as usize;

        if w == 0 || h == 0 || x + w >= img_width || y + h >= img_height {
            panic!("Out of bounds crop: x={x} y={y} w={w} h={h} img_width={img_width} img_height={img_height}");
        }

        let crop = Crop::new(x, y, w, h);
        self.exec_multi_undoable_action(Box::new(crop));
    }

    pub fn delete_selection(&mut self) {
        if self.active_layer_locked() {
            self.alert_user_of_lock("Can't delete selection: active layer locked");
            return;
        }

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
        let image_net_size = self.image_width() * self.image_height();
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
        let w = self.image_width() as usize;

        self.pencil_mask[r * w + c] == self.pencil_mask_counter
    }

    fn set_pencil_mask_at(&mut self, r: usize, c: usize) {
        self.validate_pencil_mask();
        let w = self.image_width() as usize;
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

                if ip < 0 || jp < 0 || ip >= self.image_height() || jp >= self.image_width() ||
                   self.test_pencil_mask_at(ip as usize, jp as usize) {
                    continue;
                }

                let p = self.active_image_mut().pix_at_mut(ip, jp);
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

    pub fn append_layer(&mut self, fill_color: RGBA) -> LayerIndex {
        // insert at index above current layer
        let current_idx = self.image_hist.now().active_layer_index();
        let target_idx = LayerIndex::from_usize(current_idx.to_usize() + 1);
        self.image_hist.append_layer(fill_color, target_idx);
        self.update();
        target_idx
    }


    pub fn clone_active_layer(&mut self) -> LayerIndex {
        let active_layer_idx = self.layered_image().active_layer_index().clone();
        let target_idx = LayerIndex::from_usize(active_layer_idx.to_usize() + 1);
        self.image_hist.clone_layer(active_layer_idx, target_idx);
        self.update();
        target_idx
    }

    /// Set the layer at the given index to active
    pub fn focus_layer(&mut self, layer_index: LayerIndex) {
        self.image_hist.focus_layer(layer_index);
        self.update();
    }

    /// Attempts to move the active layer up (by one)
    pub fn try_move_active_layer_up(&mut self) -> Result<LayerIndex, ()> {
        let active_layer_idx = self.layered_image().active_layer_index().clone();
        let target_idx = LayerIndex::from_usize(active_layer_idx.to_usize() + 1);
        if target_idx >= self.layered_image().next_unused_layer_idx() {
            return Err(()); // can't move top layer up
        }

        self.image_hist.swap_layers(active_layer_idx, target_idx);
        self.update();
        Ok(target_idx)
    }

    /// Attempts to move the active layer down (by one)
    pub fn try_move_active_layer_down(&mut self) -> Result<LayerIndex, ()> {
        let active_layer_idx = self.layered_image().active_layer_index().clone();
        if active_layer_idx == LayerIndex::BaseLayer {
            return Err(()); // can't move base layer down
        }

        let target_idx = LayerIndex::from_usize(active_layer_idx.to_usize() - 1);

        self.image_hist.swap_layers(active_layer_idx, target_idx);
        self.update();
        Ok(target_idx)
    }

    pub fn try_merge_active_layer_down(&mut self) -> Result<LayerIndex, ()> {
        let active_layer_idx = self.layered_image().active_layer_index().clone();
        if active_layer_idx == LayerIndex::BaseLayer {
            return Err(()); // can't move base layer down
        }

        let target_idx = LayerIndex::from_usize(active_layer_idx.to_usize() - 1);

        if self.active_layer_locked() {
            self.alert_user_of_lock("Can't merge: active layer is locked");
            return Err(());
        } else if self.layer_at_index_is_locked(target_idx) {
            self.alert_user_of_lock("Can't merge: target layer is locked");
            return Err(());
        }

        self.image_hist.merge_layers(active_layer_idx, target_idx);
        self.update();
        Ok(target_idx)
    }

    pub fn remove_layer(&mut self, layer_index: LayerIndex) {
        self.image_hist.remove_layer(layer_index);
        self.update();
    }

    pub fn toggle_layer_lock(&mut self, layer_index: LayerIndex) {
        self.image_hist.now_mut().toggle_layer_lock(layer_index);
        self.update();
    }

    pub fn toggle_layer_visibility(&mut self, layer_index: LayerIndex) {
        self.image_hist.now_mut().toggle_layer_visibility(layer_index);
        self.update();
    }

    pub fn set_layer_name(&mut self, layer_index: LayerIndex, new_name: &str) {
        self.image_hist.now_mut().set_layer_name(layer_index, new_name);
    }

    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    pub fn transformation_selection(&self) -> &RefCell<Option<TransformationSelection>> {
        &self.transformation_selection
    }

    /// Deletes the current selection (both `self.selection`, and actually
    /// clears the pixels on the image, without committing that change),
    /// switching the mouse mode to free-transform
    pub fn try_consume_selection_to_transformable(&mut self) -> Result<(), ()> {
        fn xywh_to_matrix(x: usize, y: usize, w: usize, h: usize) -> cairo::Matrix {
            let mut matrix = cairo::Matrix::identity();
            matrix.translate(x as f64, y as f64);
            matrix.scale(w as f64, h as f64);

            matrix
        }

        let res = match self.selection {
            Selection::Rectangle(x, y, w, h) => {
                *self.transformation_selection.borrow_mut() = Some(TransformationSelection::new(
                    Box::new(TransformableImage::from_image(self.active_image().subimage(x, y, w, h))),
                    xywh_to_matrix(x, y, w, h),
                    ActionName::Transform,
                ));

                Ok(())
            },
            Selection::Bitmask(ref bitmask) => {
                let (x, y, w, h) = bitmask.bounding_box();

                if w == 0 || h == 0 {
                    Err(())
                } else {
                    let subimage = self.active_image().subimage(x, y, w, h);
                    let submask = bitmask.submask(x, y, w, h);
                    let transparent = Pixel::from_rgba(0, 0, 0, 0);

                    let pixels = (0..(w * h))
                        .map(|i| if submask.bit_at(i) { subimage.pix_at_flat(i) } else { &transparent })
                        .map(|p| p.clone())
                        .collect::<Vec<_>>();

                    let masked_image = Image::new(pixels, w, h);

                    *self.transformation_selection.borrow_mut() = Some(TransformationSelection::new(
                        Box::new(TransformableImage::from_image(masked_image)),
                        xywh_to_matrix(x, y, w, h),
                        ActionName::Transform,
                    ));

                    Ok(())
                }
            },
            _ => Err(()),
        };

        let selection = std::mem::replace(&mut self.selection, Selection::NoSelection);
        for (i, j) in selection.iter() {
            *self.active_image_mut().pix_at_mut(i as i32, j as i32) =
                crate::image::Pixel::from_rgba(0, 0, 0, 0); // transparent
        }

        res
    }

    pub fn scrap_transformable(&mut self) {
        if self.transformation_selection.borrow_mut().is_some() {
            *self.transformation_selection.borrow_mut() = None;
            let last_variant = self.ui_p.borrow().toolbar_p.borrow().last_two_mouse_mode_variants().0;
            let last_mode = MouseMode::from_variant(last_variant, self);
            self.ui_p.borrow().toolbar_p.borrow_mut().set_mouse_mode(last_mode);
            self.update();
            if self.layered_image().has_unsaved_changes() {
                self.commit_changes(ActionName::Delete);
            }
        }
    }

    fn commit_transformable_no_update(&mut self) {
        let mut transformable_option = self.transformation_selection.borrow_mut();

        if let Some(selection) = transformable_option.as_mut() {
            let (width, height) = matrix_width_height(&selection.matrix);
            let sampleable = selection.transformable.gen_sampleable(width, height);
            let commit_struct = SampleableCommit::new(&sampleable, selection.matrix, selection.culprit.clone());

            // self.exec_auto_diff_action(commit_struct);
            // can't call because of ownership: work-around:
            {
                self.image_hist.exec_doable_action_taking_blame(commit_struct);
                std::mem::drop(sampleable);
                std::mem::drop(transformable_option);
                self.save_cursor_pos_after_history_commit();
            }
        }
    }

    pub fn commit_transformable(&mut self) {
        self.commit_transformable_no_update();
        self.update();
    }

    pub fn commit_and_scrap_transformable(&mut self) {
        self.commit_transformable_no_update();
        let _ = self.scrap_transformable();
    }
}
