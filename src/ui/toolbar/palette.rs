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

        let state_p = Rc::new(RefCell::new(ColorButton {
            widget,
            drawing_area,
            checkerboard,
            color,
        }));

        state_p.borrow().drawing_area.set_draw_func(clone!(@strong state_p => move |_drawing_area, cr, width, height| {
            cr.scale((width / 2).into(), (height / 2).into());

            let transparent_pattern = state_p.borrow_mut().checkerboard.to_repeated_surface_pattern();
            let _ = cr.set_source(&transparent_pattern);
            cr.rectangle(0.0, 0.0, 2.0, 2.0);
            let _ = cr.fill();

            let color = state_p.borrow().color;
            cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
            cr.rectangle(0.0, 0.0, 2.0, 2.0);
            let _ = cr.fill();
        }));

        let click_controller = GestureClick::builder()
            .button(0)
            .build();

        const RIGHT_CLICK_BUTTON: u32 = 3;

        click_controller.connect_released(clone!(@strong state_p => move |controller, _n_press, _x, _y| {
            if controller.current_button() == RIGHT_CLICK_BUTTON {
                Self::select_new_color(state_p.clone());
            }
        }));

        state_p.borrow().widget.add_controller(click_controller);

        state_p
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
}

impl Palette {
    pub fn new(colors: Vec<RGBA>) -> Self {
        let widget = GBox::new(Orientation::Horizontal, 10);

        let color_buttons = colors.iter()
            .map(|rgba| ColorButton::new_p(*rgba))
            .collect::<Vec<_>>();

        for button in color_buttons.iter() {
            widget.append(&button.borrow().widget);
        }

        Palette {
            widget,
            color_buttons,
        }
    }

    pub fn widget(&self) -> &GBox {
        &self.widget
    }

    pub fn primary_color(&self) -> RGBA {
        todo!("primary color")
    }
}
