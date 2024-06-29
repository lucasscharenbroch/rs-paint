use crate::image::{DrawableImage, mk_transparent_checkerboard};
use crate::ui::{dialog::choose_color_dialog, get_parent_window};

use gtk::{prelude::*, Orientation, DrawingArea, Box as GBox, GestureClick};
use std::rc::Rc;
use std::cell::RefCell;
use gtk::gdk::RGBA;
use glib_macros::clone;

struct PaletteColorButton {
    widget: gtk::Button,
    drawing_area: DrawingArea,
    checkerboard: DrawableImage,
    color: RGBA,
}

impl PaletteColorButton {
    fn new_p(color: RGBA) -> Rc<RefCell<Self>> {
        let widget = gtk::Button::new();
        let drawing_area =  DrawingArea::builder()
            .content_height(30)
            .content_width(30)
            .build();

        widget.set_child(Some(&drawing_area));

        let checkerboard = mk_transparent_checkerboard();

        let cb_p = Rc::new(RefCell::new(PaletteColorButton {
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

/// A toggle-button and color wrapper that's used for
/// both the primary and secondary color buttons
struct PrimarySecondaryButton {
    widget: gtk::ToggleButton,
    drawing_area: DrawingArea,
    checkerboard: DrawableImage,
    color: RGBA,
    kind: PrimaryOrSecondary,
}

impl PrimarySecondaryButton {
    fn new_p(color: RGBA, kind: PrimaryOrSecondary) -> Rc<RefCell<Self>> {
        let widget = gtk::ToggleButton::builder()
            .active(kind == PrimaryOrSecondary::Primary)
            .build();
        let drawing_area =  DrawingArea::builder()
            .content_height(30)
            .content_width(30)
            .build();

        widget.set_child(Some(&drawing_area));

        let checkerboard = mk_transparent_checkerboard();

        let cb_p = Rc::new(RefCell::new(PrimarySecondaryButton {
            widget,
            drawing_area,
            checkerboard,
            color,
            kind,
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
    color_buttons: Vec<Rc<RefCell<PaletteColorButton>>>,
    active: PrimaryOrSecondary,
    primary_button_p: Rc<RefCell<PrimarySecondaryButton>>,
    secondary_button_p: Rc<RefCell<PrimarySecondaryButton>>,
}

#[derive(PartialEq)]
enum PrimaryOrSecondary {
    Primary,
    Secondary,
}

impl Palette {
    pub fn new_p(default_primary: RGBA, default_secondary: RGBA, colors: Vec<RGBA>) -> Rc<RefCell<Self>> {
        let widget = GBox::new(Orientation::Horizontal, 10);

        let color_buttons = colors.iter()
            .map(|rgba| PaletteColorButton::new_p(*rgba))
            .collect::<Vec<_>>();

        let primary_button_p = PrimarySecondaryButton::new_p(default_primary, PrimaryOrSecondary::Primary);
        let secondary_button_p = PrimarySecondaryButton::new_p(default_secondary, PrimaryOrSecondary::Secondary);

        let palette_p = Rc::new(RefCell::new(Palette {
            widget,
            color_buttons,
            active: PrimaryOrSecondary::Primary,
            primary_button_p,
            secondary_button_p,
        }));

        for cb_p in palette_p.borrow().color_buttons.iter() {
            palette_p.borrow().widget.append(&cb_p.borrow().widget);
            cb_p.borrow().widget.connect_clicked(clone!(@strong palette_p, @strong cb_p => move |_button| {
                palette_p.borrow_mut().set_active_color(cb_p.borrow().color);
            }));
        }

        let primary_secondary_wrapper = gtk::Box::new(gtk::Orientation::Vertical, 4);

        primary_secondary_wrapper.append(&palette_p.borrow().primary_button_p.borrow().widget);
        primary_secondary_wrapper.append(&palette_p.borrow().secondary_button_p.borrow().widget);

        palette_p.borrow().widget.prepend(&primary_secondary_wrapper);

        palette_p.borrow()
            .primary_button_p.borrow().widget
            .connect_clicked(clone!(@strong palette_p => move |button| {
                button.set_active(true);
                palette_p.borrow().secondary_button_p.borrow().widget.set_active(false);
                palette_p.borrow_mut().active = PrimaryOrSecondary::Primary;
            }));

        palette_p.borrow()
            .secondary_button_p.borrow().widget
            .connect_clicked(clone!(@strong palette_p => move |button| {
                button.set_active(true);
                palette_p.borrow().primary_button_p.borrow().widget.set_active(false);
                palette_p.borrow_mut().active = PrimaryOrSecondary::Secondary;
            }));

        palette_p
    }

    pub fn widget(&self) -> &GBox {
        &self.widget
    }

    pub fn primary_color(&self) -> RGBA {
        self.primary_button_p.borrow().color
    }

    pub fn secondary_color(&self) -> RGBA {
        self.secondary_button_p.borrow().color
    }

    /// Sets the primary or secondary color (whichever
    /// is currently active) to the argument
    pub fn set_active_color(&mut self, color: RGBA) {
        let active_button_p = match self.active {
            PrimaryOrSecondary::Primary => &self.primary_button_p,
            PrimaryOrSecondary::Secondary => &self.secondary_button_p,
        };

        active_button_p.borrow_mut().color = color;
        active_button_p.borrow().drawing_area.queue_draw();
    }
}
