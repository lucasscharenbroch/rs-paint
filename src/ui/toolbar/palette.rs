use crate::image::{DrawableImage, mk_transparent_checkerboard};
use crate::ui::{dialog::choose_color_dialog, get_parent_window};

use gtk::{prelude::*, Orientation, DrawingArea, ToggleButton, Box as GBox, GestureClick};
use std::rc::Rc;
use std::cell::RefCell;
use gtk::gdk::RGBA;
use glib_macros::clone;

struct ColorButton {
    widget: ToggleButton,
    drawing_area: DrawingArea,
    checkerboard: DrawableImage,
    color: RGBA,
}

impl ColorButton {
    fn new_p(color: RGBA) -> Rc<RefCell<Self>> {
        let widget = ToggleButton::new();
        let drawing_area =  DrawingArea::builder()
            .content_height(30)
            .content_width(30)
            .build();

        widget.set_child(Some(&drawing_area));

        let checkerboard = mk_transparent_checkerboard();

        let cb_p = Rc::new(RefCell::new(ColorButton {
            widget,
            drawing_area,
            checkerboard,
            color,
        }));

        cb_p.borrow().drawing_area.set_draw_func(clone!(@strong cb_p => move |_drawing_area, cr, width, height| {
            cr.scale((width / 2).into(), (height / 2).into());

            let transparent_pattern = cb_p.borrow_mut().checkerboard.to_repeated_surface_pattern();
            let _ = cr.set_source(&transparent_pattern);
            cr.rectangle(0.0, 0.0, 2.0, 2.0);
            let _ = cr.fill();

            let color = cb_p.borrow().color;
            cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
            cr.rectangle(0.0, 0.0, 2.0, 2.0);
            let _ = cr.fill();
        }));

        let click_controller = GestureClick::builder()
            .button(0)
            .build();

        const RIGHT_CLICK_BUTTON: u32 = 3;

        // Scuffed use of click_controller's handlers: I don't know
        // why this works, and why the alternatives don't.

        click_controller.connect_end(|controller, _| {
            controller.reset();
        });

        click_controller.connect_stopped(clone!(@strong cb_p => move |controller| {
            if controller.current_button() == RIGHT_CLICK_BUTTON {
                Self::select_new_color(cb_p.clone());
            }
        }));

        cb_p.borrow().widget.add_controller(click_controller);

        cb_p
    }

    fn select_new_color(self_p: Rc<RefCell<Self>>) {
        let parent = get_parent_window(&self_p.borrow().widget);
        let parent_ref = if let Some(ref w) = parent {
            Some(w)
        } else {
            None
        };

        choose_color_dialog(parent_ref, move |res_color| {
            if let Ok(rgba) = res_color {
                self_p.borrow_mut().color = rgba;
                self_p.borrow_mut().drawing_area.queue_draw();
            }
        });
    }
}

pub struct Palette {
    widget: GBox,
    color_buttons: Vec<Rc<RefCell<ColorButton>>>,
    active_idx: usize,
}

impl Palette {
    pub fn new_p(colors: Vec<RGBA>) -> Rc<RefCell<Self>> {
        let widget = GBox::new(Orientation::Horizontal, 10);

        let color_buttons = colors.iter()
            .map(|rgba| ColorButton::new_p(*rgba))
            .collect::<Vec<_>>();

        color_buttons[0].borrow().widget.set_active(true);

        let palette_p = Rc::new(RefCell::new(Palette {
            widget,
            color_buttons,
            active_idx: 0,
        }));

        for (i, cb_p) in palette_p.borrow().color_buttons.iter().enumerate() {
            palette_p.borrow().widget.append(&cb_p.borrow().widget);
            cb_p.borrow().widget.connect_clicked(clone!(@strong palette_p => move |_button| {
                palette_p.borrow_mut().active_idx = i;
                for (j, cb) in palette_p.borrow_mut().color_buttons.iter().enumerate() {
                    cb.borrow().widget.set_active(i == j);
                }
            }));
        }

        palette_p
    }

    pub fn widget(&self) -> &GBox {
        &self.widget
    }

    pub fn primary_color(&self) -> RGBA {
        self.color_buttons[self.active_idx].borrow().color
    }

    pub fn set_primary_color(&mut self, color: RGBA) {
        self.color_buttons[self.active_idx].borrow_mut().color = color;
        self.color_buttons[self.active_idx].borrow_mut().drawing_area.queue_draw();
    }
}
