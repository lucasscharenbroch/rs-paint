use crate::{composite_field, vertical_composite_field};

use super::*;

use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

/// A gadget is a wrapper of FormFeilds that can support
/// extra state and interconnected behavior
trait FormGadget {
    fn add_to_builder<T: FormBuilderIsh>(&self, builder: T) -> T;
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
        orientation: gtk::Orientation,
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
        // ensure the label-slider and number are aligned correctly
        // by binding the two into a composite
        if self.use_vertical_layout {
            builder
                .with_field(&vertical_composite_field!(&self.slider_field, &self.label_field))
        } else {
            builder
                .with_field(&composite_field!(&self.slider_field, &self.label_field))
        }
     }
}


/// Similar to RadioField, but each button gets its own widget
/// This needs to be a gadget because there needs to be a pointer
/// to the container to update the buttons' activity.
pub struct ToggleButtonsGadget<T> {
    buttons: Vec<gtk::ToggleButton>,
    wrapper: gtk::Box,
    variants: Vec<T>,
}

impl<T: 'static> ToggleButtonsGadget<T> {
    pub fn new_p(
        label: Option<&str>,
        variants: Vec<(&impl IsA<gtk::Widget>, T)>,
        default: usize,
        overall_orientation: gtk::Orientation,
        num_per_group: usize,
        group_orientation: gtk::Orientation,
    ) -> Rc<RefCell<Self>> {
        let buttons = variants.iter().enumerate()
            .map(|(idx, (child_widget, _x))| {
            gtk::ToggleButton::builder()
                .child(*child_widget)
                .active(idx == default)
                .build()
        }).collect::<Vec<_>>();

        let variants = variants.into_iter()
            .map(|(_, x)| x)
            .collect::<Vec<_>>();

        let wrapper = gtk::Box::builder()
            .orientation(overall_orientation)
            .spacing(4)
            .build();

        let num_groups = if buttons.len() % num_per_group != 0 {
            1 + (buttons.len() / num_per_group)
        } else {
            buttons.len() / num_per_group
        };

        let mut groups = Vec::with_capacity(num_groups);

        for _ in 0..num_groups { // can't use vec! macro (need a loop)
            groups.push(gtk::Box::new(group_orientation, 4));
        }

        for (i, b) in buttons.iter().enumerate() {
            groups[i / num_per_group].append(b);
        }

        for g in groups.iter() {
            wrapper.append(g);
        }

        label.map(|label_text| wrapper.prepend(&new_label(label_text)));

        buttons[0].set_active(true);

        let self_p = Rc::new(RefCell::new(
            ToggleButtonsGadget {
                buttons,
                wrapper,
                variants,
            }
        ));

        for (i, b) in self_p.borrow().buttons.iter().enumerate() {
            b.connect_clicked(clone!(@strong self_p => move |b| {
                for (ip, bp) in self_p.borrow().buttons.iter().enumerate() {
                    bp.set_active(i == ip);
                }
            }));
        }

        self_p
    }

    pub fn value(&self) -> Option<&T> {
        self.buttons.iter()
            .enumerate()
            .filter(|(_idx, b)| b.is_active())
            .next()
            .map(|(idx, _b)| &self.variants[idx]) // map over the Option
    }
}

impl<T> FormGadget for ToggleButtonsGadget<T> {
    fn add_to_builder<B: FormBuilderIsh>(&self, builder: B) -> B {
        builder.with_field(&self.wrapper)
    }
}
