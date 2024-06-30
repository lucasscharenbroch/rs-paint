use crate::composite_field;

use super::*;

use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

/// A gadget is a wrapper of FormFeilds that can support
/// extra state and interconnected behavior
trait FormGadget {
    fn add_to_builder<T: FormBuilderIsh>(&self, buider: T) -> T;
}

impl FormBuilder {
    pub fn with_gadget(self, new_gadget: &impl FormGadget) -> Self {
        new_gadget.add_to_builder(self)
    }
}

impl FlowFormBuilder {
    pub fn with_gadget(self, new_gadget: &impl FormGadget) -> Self {
        new_gadget.add_to_builder(self)
    }
}

pub struct AspectRatioGadget {
    old_width: usize,
    old_height: usize,
    enforce: bool,
    width_field: NaturalField,
    height_field: NaturalField,
    ratio_button: CheckboxField,
}

impl FormGadget for AspectRatioGadget {
     fn add_to_builder<T: FormBuilderIsh>(&self, builder: T) -> T {
        builder
            .with_field(&self.width_field)
            .with_field(&self.height_field)
            .with_field(&self.ratio_button)
     }
}

impl AspectRatioGadget {
    fn update_ratio(&mut self) {
        self.old_width = self.width_field.value();
        self.old_height = self.height_field.value();
    }

    pub fn new_p(
        width_label: &str,
        height_label: &str,
        default_width: usize,
        default_height: usize,
        allow_zero: bool,
    ) -> Rc<RefCell<Self>> {
        let min = if allow_zero { 0 } else { 1 };
        let width_field = NaturalField::new(Some(width_label), min, usize::MAX, 1, default_width);
        let height_field = NaturalField::new(Some(height_label), min, usize::MAX, 1, default_height);
        let ratio_button = CheckboxField::new(Some("Maintain Aspect Ratio"), true);

        let state_p = Rc::new(RefCell::new(AspectRatioGadget {
            old_width: width_field.value(),
            old_height: height_field.value(),
            enforce: ratio_button.value(),
            width_field,
            height_field,
            ratio_button,
        }));

        state_p.borrow().ratio_button.set_toggled_hook(clone!(@strong state_p => move |now_active| {
            state_p.borrow_mut().enforce = now_active;
            if now_active {
                state_p.borrow_mut().update_ratio();
            }
        }));

        state_p.borrow().width_field.set_changed_hook(clone!(@strong state_p => move |new_width| {
            if let Ok(state) = state_p.try_borrow_mut() {
                if !state.enforce {
                    return
                }

                let width_change = new_width as f64 / (state.old_width as f64);
                state.height_field.set_value((state.old_height as f64 * width_change).ceil() as usize);
            }
        }));

        state_p.borrow().height_field.set_changed_hook(clone!(@strong state_p => move |new_height| {
            if let Ok(state) = state_p.try_borrow_mut() {
                if !state.enforce {
                    return
                }

                let height_change = new_height as f64 / (state.old_height as f64);
                state.width_field.set_value((state.old_width as f64 * height_change).ceil() as usize);
            }
        }));

        state_p
    }

    pub fn width(&self) -> usize {
        self.width_field.value()
    }

    pub fn height(&self) -> usize {
        self.height_field.value()
    }
}

pub struct NumberedSliderGadget {
    slider_field: SliderField,
    label_field: LabelField,
    /// Place the label above (and the number below) the slider?
    use_vertical_layout: bool,
}

impl NumberedSliderGadget {
    pub fn new_p(
        label: Option<&str>,
        orientation: Orientation,
        use_vertical_layout: bool,
        min: usize,
        max: usize,
        step: usize,
        default_value: usize,
        suffix: String,
    ) -> Rc<RefCell<Self>> {
        let gen_label = move |new_val: usize| {
            format!("{new_val}{suffix}")
        };

        let slider_field = SliderField::new(label, orientation, min, max, step, default_value);
        let label_field = LabelField::new(&gen_label(default_value));

        let state_p = Rc::new(RefCell::new(NumberedSliderGadget {
            slider_field,
            label_field,
            use_vertical_layout,
        }));

        state_p.borrow().slider_field.set_changed_hook(clone!(@strong state_p => move |new_val| {
            state_p.borrow().label_field.set_text(&gen_label(new_val))
        }));

        if use_vertical_layout {
            state_p.borrow().slider_field.set_population_orientation(gtk::Orientation::Vertical);
        }

        state_p
    }

    pub fn value(&self) -> usize {
        self.slider_field.value()
    }
}

impl FormGadget for NumberedSliderGadget {
     fn add_to_builder<T: FormBuilderIsh>(&self, builder: T) -> T {
        if self.use_vertical_layout {
            builder
                .with_field(&self.slider_field)
                .with_field(&self.label_field)
        } else {
            // ensure the label-slider and number are all on the same line by binding
            // the two into a composite
            builder
                .with_field(&composite_field!(&self.slider_field, &self.label_field))
        }
     }
}
