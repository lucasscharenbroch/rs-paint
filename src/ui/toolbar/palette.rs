use crate::image::{DrawableImage, mk_transparent_checkerboard};
use crate::ui::{dialog::choose_color_dialog, get_parent_window};

use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::gdk::RGBA;
use glib_macros::clone;

struct PaletteColorButton {
    widget: gtk::Button,
    drawing_area: gtk::DrawingArea,
    checkerboard: DrawableImage,
    color: Option<RGBA>,
}

impl PaletteColorButton {
    fn new_p(color: Option<RGBA>) -> Rc<RefCell<Self>> {
        const SIZE: i32 = 30;

        let widget = gtk::Button::builder()
            .height_request(SIZE)
            .width_request(SIZE)
            .css_classes(["no-padding", "palette-button"])
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();

        let drawing_area =  gtk::DrawingArea::builder()
            .content_height(SIZE)
            .content_width(SIZE)
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
            cr.rectangle(0.0, 0.0, 2.1, 2.1);
            let _ = cr.fill();

            if let Some(color) = cb_p.borrow().color {
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
                cr.rectangle(0.0, 0.0, 2.1, 2.1);
                let _ = cr.fill();
            } else {
                // color is empty: button disabled: draw x
                cr.set_line_width(0.1);
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.move_to(0.0, 0.0);
                cr.line_to(2.1, 2.1);
                let _ = cr.stroke();
                cr.move_to(2.1, 0.0);
                cr.line_to(0.0, 2.1);
                let _ = cr.stroke();
            }
        }));

        let click_controller = gtk::GestureClick::builder()
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
                if controller.current_event_state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                    cb_p.borrow_mut().color = None;
                    cb_p.borrow().drawing_area.queue_draw();
                } else {
                    Self::select_new_color(cb_p.clone());
                }
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
                self_p.borrow_mut().color = Some(rgba);
                self_p.borrow_mut().drawing_area.queue_draw();
            }
        });
    }
}

/// A toggle-button and color wrapper that's used for
/// both the primary and secondary color buttons
struct PrimarySecondaryButton {
    widget: gtk::ToggleButton,
    drawing_area: gtk::DrawingArea,
    checkerboard: DrawableImage,
    color: RGBA,
    kind: PrimaryOrSecondary,
}

impl PrimarySecondaryButton {
    fn new_p(color: RGBA, kind: PrimaryOrSecondary) -> Rc<RefCell<Self>> {
        let size = match kind {
            PrimaryOrSecondary::Primary => 40,
            PrimaryOrSecondary::Secondary => 25,
        };

        let widget = gtk::ToggleButton::builder()
            .active(kind == PrimaryOrSecondary::Primary)
            .height_request(size)
            .width_request(size)
            .css_classes(["no-padding", "primary-secondary-border"])
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();

        let drawing_area =  gtk::DrawingArea::builder()
            .content_height(size)
            .content_width(size)
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
            cr.rectangle(0.0, 0.0, 2.1, 2.1);
            let _ = cr.fill();

            let color = cb_p.borrow().color;
            cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
            cr.rectangle(0.0, 0.0, 2.1, 2.1);
            let _ = cr.fill();
        }));

        let click_controller = gtk::GestureClick::builder()
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
    widget: gtk::Box,
    color_buttons: Vec<Vec<Rc<RefCell<PaletteColorButton>>>>,
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
    pub fn new_p(default_primary: RGBA, default_secondary: RGBA, colors: Vec<Vec<Option<RGBA>>>) -> Rc<RefCell<Self>> {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();

        let color_buttons = colors.iter()
            .map(|inner_vec| {
                inner_vec.iter()
                    .map(|color| PaletteColorButton::new_p(*color))
                    .collect::<Vec<_>>()
            })
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

        let color_array_wrapper_widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .valign(gtk::Align::Center)
            .build();

        for row in palette_p.borrow().color_buttons.iter() {
            let row_widget = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(4)
                .build();

            for cb_p in row {
                row_widget.append(&cb_p.borrow().widget);
                cb_p.borrow().widget.connect_clicked(clone!(@strong palette_p, @strong cb_p => move |_button| {
                    if let Some(color) = cb_p.borrow().color {
                        palette_p.borrow_mut().set_active_color(color);
                    }
                }));
            }

            color_array_wrapper_widget.append(&row_widget);
        }

        Self::set_up_util_buttons(&palette_p);

        palette_p.borrow().widget.append(&color_array_wrapper_widget);

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

    fn set_up_util_buttons(palette_p: &Rc<RefCell<Self>>) {
        let util_button_wrapper = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();

        use crate::icon_file;

        let arrow_icon1 = gtk::Image::builder()
            .file(icon_file!("right-arrow"))
            .build();

        let arrow_icon2 = gtk::Image::builder()
            .file(icon_file!("right-arrow"))
            .build();

        let swap_icon = gtk::Image::builder()
            .file(icon_file!("swap"))
            .build();

        let primary_to_palette = gtk::Button::builder()
            .child(&arrow_icon1)
            .css_classes(["no-padding"])
            .build();

        let swap_primary_and_secondary = gtk::Button::builder()
            .child(&swap_icon)
            .css_classes(["no-padding"])
            .build();

        let secondary_to_palette = gtk::Button::builder()
            .child(&arrow_icon2)
            .css_classes(["no-padding"])
            .build();

        primary_to_palette.connect_clicked(clone!(@strong palette_p => move |_| {
            let color = palette_p.borrow().primary_color();
            let _ = palette_p.borrow_mut().add_color(color);
        }));

        swap_primary_and_secondary.connect_clicked(clone!(@strong palette_p => move |_| {
            palette_p.borrow_mut().swap_primary_and_secondary();
        }));

        secondary_to_palette.connect_clicked(clone!(@strong palette_p => move |_| {
            let color = palette_p.borrow().secondary_color();
            let _ = palette_p.borrow_mut().add_color(color);
        }));

        util_button_wrapper.append(&primary_to_palette);
        util_button_wrapper.append(&swap_primary_and_secondary);
        util_button_wrapper.append(&secondary_to_palette);

        palette_p.borrow().widget.append(&util_button_wrapper);
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn primary_color(&self) -> RGBA {
        self.primary_button_p.borrow().color
    }

    pub fn secondary_color(&self) -> RGBA {
        self.secondary_button_p.borrow().color
    }

    pub fn set_primary_color(&self, color: RGBA) {
        self.primary_button_p.borrow_mut().color = color;
        self.primary_button_p.borrow().drawing_area.queue_draw();
    }

    pub fn set_secondary_color(&self, color: RGBA) {
        self.secondary_button_p.borrow_mut().color = color;
        self.secondary_button_p.borrow().drawing_area.queue_draw();
    }

    /// Sets the primary or secondary color (whichever
    /// is currently active) to the argument
    fn set_active_color(&mut self, color: RGBA) {
        let active_button_p = match self.active {
            PrimaryOrSecondary::Primary => &self.primary_button_p,
            PrimaryOrSecondary::Secondary => &self.secondary_button_p,
        };

        active_button_p.borrow_mut().color = color;
        active_button_p.borrow().drawing_area.queue_draw();
    }

    /// Looks for an empty color slot in the palette,
    /// inserting the given color if found
    pub fn add_color(&mut self, color: RGBA) -> Result<(), ()> {
        for row in self.color_buttons.iter() {
            for cb_p in row.iter() {
                if cb_p.borrow().color.is_none() {
                    cb_p.borrow_mut().color = Some(color);
                    cb_p.borrow().drawing_area.queue_draw();
                    return Ok(());
                }
            }
        }

        Err(())
    }

    fn swap_primary_and_secondary(&mut self) {
        let primary = self.primary_color();
        let secondary = self.secondary_color();
        self.set_primary_color(secondary);
        self.set_secondary_color(primary);
    }
}
