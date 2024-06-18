pub mod mode;
mod palette;

use gtk::gdk::RGBA;
use mode::MouseMode;
use super::canvas::Canvas;
use super::UiState;
use palette::Palette;
use crate::image::brush::{Brush, BrushType};
use crate::image::blend::BlendingMode;
use super::toolbar::mode::ModeToolbar;

use gtk::prelude::*;
use gtk::{Box as GBox, Orientation, ToggleButton};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

pub struct Toolbar {
    widget: GBox,
    mode_button_box: GBox,
    palette_p: Rc<RefCell<Palette>>,
    mouse_mode: MouseMode,
    mouse_mode_buttons: Vec<MouseModeButton>,
    mode_change_hook: Option<Box<dyn Fn(&Toolbar)>>,
    mode_toolbar: ModeToolbar,
    brush: Brush,
}

struct MouseModeButton {
    mode: MouseMode,
    widget: ToggleButton,
}

const INITIAL_MODE: MouseMode = MouseMode::cursor_default();

impl Toolbar {
    pub fn new_p() -> Rc<RefCell<Toolbar>> {
        let default_palette_colors = vec![
            RGBA::new(0.0, 0.0, 0.0, 1.0),
            RGBA::new(1.0, 0.0, 0.0, 1.0),
            RGBA::new(0.0, 1.0, 0.0, 1.0),
            RGBA::new(0.0, 0.0, 1.0, 1.0),
            RGBA::new(0.0, 0.0, 0.0, 0.0),
        ];

        let default_color = default_palette_colors[0].clone();
        let widget =  GBox::new(Orientation::Horizontal, 10);
        let mode_button_box =  GBox::new(Orientation::Horizontal, 10);
        let palette_p = Palette::new_p(default_palette_colors);
        let mode_toolbar_wrapper = GBox::builder().build();
        let mode_toolbar = ModeToolbar::new(&mode_toolbar_wrapper, Some(INITIAL_MODE.variant()));
        let brush = Brush::new(default_color, BrushType::Round, 5);

        widget.append(&mode_button_box);
        widget.append(palette_p.borrow().widget());
        widget.append(&mode_toolbar_wrapper);

        let toolbar_p = Rc::new(RefCell::new(Toolbar {
            widget,
            mode_button_box,
            palette_p,
            mouse_mode: INITIAL_MODE,
            mouse_mode_buttons: vec![],
            mode_change_hook: None,
            mode_toolbar,
            brush,
        }));

        toolbar_p
    }

    pub fn init_ui_hooks(ui_p: &Rc<RefCell<UiState>>) {
        let toolbar_p = ui_p.borrow().toolbar_p.clone();

        let button_info: Vec<(&str, fn(&Canvas) -> MouseMode, fn() -> MouseMode)> = vec![
            ("cursor", MouseMode::cursor, MouseMode::cursor_default),
            ("pencil", MouseMode::pencil, MouseMode::pencil_default),
            ("eyedropper", MouseMode::eyedropper, MouseMode::eyedropper_default),
            ("rectangle-select", MouseMode::rectangle_select, MouseMode::rectangle_select_default),
            ("magic-wand", MouseMode::magic_wand, MouseMode::magic_wand_default),
            ("fill", MouseMode::fill, MouseMode::fill_default),
        ];

        toolbar_p.borrow_mut().mouse_mode_buttons = button_info.into_iter()
            .map(|(name, mode_constructor, mode_constructor_default)| {
                let icon_widget = gtk::Image::builder()
                    .file(format!("./icons/{name}.png"))
                    .build();

                let button = ToggleButton::builder()
                    .child(&icon_widget)
                    .width_request(75)
                    .height_request(75)
                    .build();

                button.connect_clicked(clone!(@strong toolbar_p, @strong ui_p => move |b| {
                    if b.is_active() {
                        let mode =
                            if let Some(canvas_p) = ui_p.borrow().active_canvas_p()  {
                                mode_constructor(&canvas_p.borrow())
                            } else {
                                mode_constructor_default()
                            };

                        toolbar_p.borrow_mut().set_mouse_mode(mode.clone());
                        for other_button in toolbar_p.borrow().mouse_mode_buttons.iter() {
                            if other_button.mode.variant() != mode.variant() {
                                other_button.widget.set_active(false);
                            }
                        }

                        if let Some(ref f) = toolbar_p.borrow().mode_change_hook {
                            f(&toolbar_p.borrow());
                        }
                    } else {
                        // the only way to deactivate is to activate a different modal button
                        b.set_active(true);
                    }
                }));

                toolbar_p.borrow_mut().mode_button_box.append(&button);

                MouseModeButton {
                    widget: button,
                    mode: mode_constructor_default(),
                }
            })
            .collect::<Vec<_>>();

        // activate INITIAL_MODE button
        toolbar_p.borrow_mut().mouse_mode_buttons.iter().for_each(|b| {
            if b.mode.variant() == INITIAL_MODE.variant() {
                b.widget.set_active(true);
            }
        });
    }

    pub fn mouse_mode(&self) -> &MouseMode {
        &self.mouse_mode
    }

    pub fn set_mouse_mode(&mut self, new_mouse_mode: MouseMode) {
        self.mouse_mode = new_mouse_mode;
        self.mode_toolbar.set_to_variant(new_mouse_mode.variant());
    }

    pub fn primary_color(&self) -> RGBA {
        self.palette_p.borrow().primary_color()
    }

    pub fn set_primary_color(&self, color: RGBA) {
        self.palette_p.borrow_mut().set_primary_color(color);
    }

    pub fn widget(&self) -> &GBox {
        &self.widget
    }

    pub fn set_mode_change_hook(&mut self, f: Box<dyn Fn(&Toolbar)>) {
        self.mode_change_hook = Some(f);
    }

    fn get_brush(&mut self) -> &Brush {
        let color = self.primary_color();
        let (brush_type, _blending_mode, radius) = self.mode_toolbar.get_pencil_settings();
        self.brush.modify(color, brush_type, radius);
        &self.brush
    }

    fn get_brush_mut(&mut self) -> &mut Brush {
        self.get_brush();
        &mut self.brush
    }

    fn get_blending_mode(&self) -> BlendingMode {
        let (_brush_type, blending_mode, _radius) = self.mode_toolbar.get_pencil_settings();
        blending_mode
    }

    fn get_magic_wand_tolerance(&self) -> f64 {
        self.mode_toolbar.get_magic_wand_settings()
    }

    fn get_fill_tolerance(&self) -> f64 {
        self.mode_toolbar.get_fill_settings()
    }
}
