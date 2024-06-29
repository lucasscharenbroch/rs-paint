pub mod gadget;

use crate::image::resize::ExpandJustification;

use gtk::{prelude::*, Box as GBox, CheckButton, ColorDialog, ColorDialogButton, Entry, Label, Orientation, SpinButton, Widget};
use gtk::gdk::RGBA;
use gtk::glib::object::IsA;
use gtk::{glib, gio};
use std::rc::Rc;
use std::cell::RefCell;
use glib_macros::clone;

fn new_label(text: &str) -> Label {
    Label::builder()
        .label(text)
        .valign(gtk::Align::Center)
        .build()
}

pub trait FormField {
    fn outer_widget(&self) -> &impl IsA<Widget>;
}

pub struct TextField {
    text_box: Entry,
    wrapper: GBox,
}

impl TextField {
    pub fn new(label: Option<&str>, default_text: &str, phantom_text: &str) -> Self {
        let text_box = Entry::builder()
            .placeholder_text(phantom_text)
            .text(default_text)
            .build();

        let wrapper = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        wrapper.append(&text_box);
        label.map(|label_text| wrapper.prepend(&new_label(label_text)));

        TextField {
            text_box,
            wrapper,
        }
    }

    pub fn value(&self) -> String {
        self.text_box.text().to_string()
    }
}

impl FormField for TextField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.wrapper
    }
}

pub struct NaturalField {
    num_entry: SpinButton,
    wrapper: GBox,
}

impl NaturalField {
    pub fn new(label: Option<&str>, min: usize, max: usize, step: usize, default_value: usize) -> Self {
        let num_entry = SpinButton::with_range(min as f64, max as f64, step as f64);
        num_entry.set_valign(gtk::Align::Center);
        num_entry.set_value(default_value as f64);

        let wrapper = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        wrapper.append(&num_entry);
        label.map(|label_text| wrapper.prepend(&new_label(label_text)));

        NaturalField {
            num_entry,
            wrapper,
        }
    }

    pub fn value(&self) -> usize {
        self.num_entry.value() as usize
    }

    pub fn set_value(&self, new_value: usize) {
        self.num_entry.set_value(new_value as f64);
    }

    pub fn set_changed_hook<F: Fn(usize) + 'static>(&self, f: F)
    {
        self.num_entry.connect_value_changed(move |b| {
            f(b.value() as usize)
        });
    }

    pub fn set_orientation(&self, orientation: gtk::Orientation) {
        self.wrapper.set_orientation(orientation);
    }
}

impl FormField for NaturalField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.wrapper
    }
}

pub struct ColorField {
    button: ColorDialogButton,
    wrapper: GBox,
}

impl ColorField {
    pub fn new(label: Option<&str>, default_color: RGBA) -> Self {
        let dialog_props = ColorDialog::builder()
            .with_alpha(true)
            .build();

        let button = ColorDialogButton::builder()
            .dialog(&dialog_props)
            .rgba(&default_color)
            .build();

        let wrapper = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        wrapper.append(&button);
        label.map(|label_text| wrapper.prepend(&new_label(label_text)));

        ColorField {
            button,
            wrapper,
        }
    }

    pub fn value(&self) -> RGBA {
        self.button.rgba()
    }
}

impl FormField for ColorField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.wrapper
    }
}

pub struct CheckboxField {
    button: CheckButton,
}

impl CheckboxField {
    pub fn new(label: Option<&str>, is_checked: bool) -> Self {
        let button = CheckButton::builder()
            .active(is_checked)
            .build();

        button.set_label(label);

        CheckboxField {
            button,
        }
    }

    pub fn value(&self) -> bool {
        self.button.is_active()
    }

    pub fn set_toggled_hook<F: Fn(bool) + 'static>(&self, f: F)
    {
        self.button.connect_toggled(move |b| {
            f(b.is_active())
        });
    }
}

impl FormField for CheckboxField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.button
    }
}

pub struct RadioField<T> {
    buttons: Vec<CheckButton>,
    wrapper: GBox,
    variants: Vec<T>,
}

impl<T> RadioField<T> {
    pub fn new(label: Option<&str>, variants: Vec<(&str, T)>, default: usize) -> Self {
        let buttons = variants.iter().enumerate()
            .map(|(idx, (label_text, _x))| {
            CheckButton::builder()
                .label(*label_text)
                .active(idx == default)
                .build()
        }).collect::<Vec<_>>();

        // group buttons together
        assert!(buttons.len() >= 1);
        let first_button = &buttons[0];
        buttons.iter().skip(1).for_each(|b| b.set_group(Some(first_button)));

        let variants = variants.into_iter()
            .map(|(_, x)| x)
            .collect::<Vec<_>>();

        let wrapper = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        for b in buttons.iter() {
            wrapper.append(b);
        }

        label.map(|label_text| wrapper.prepend(&new_label(label_text)));

        RadioField {
            buttons,
            wrapper,
            variants,
        }
    }

    pub fn value(&self) -> Option<&T> {
        self.buttons.iter()
            .enumerate()
            .filter(|(_idx, b)| b.is_active())
            .next()
            .map(|(idx, _b)| &self.variants[idx]) // map over the Option
    }
}

impl<T> FormField for RadioField<T> {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.wrapper
    }
}

pub struct DropdownField<T> {
    dropdown: gtk::DropDown,
    wrapper: GBox,
    variants: Vec<T>,
}

impl<T> DropdownField<T> {
    pub fn new(label: Option<&str>, variants: Vec<(&str, T)>, default: usize) -> Self {
        let variant_str_list = gio::ListStore::new::<gtk::StringObject>();

        for (name, _val) in variants.iter() {
            variant_str_list.append(&gtk::StringObject::new(name))
        }

        let dropdown = gtk::DropDown::builder()
            .model(&variant_str_list)
            .valign(gtk::Align::Center)
            .build();

        let variants = variants.into_iter()
            .map(|(_, x)| x)
            .collect::<Vec<_>>();

        let wrapper = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        wrapper.append(&dropdown);

        label.map(|label_text| wrapper.prepend(&new_label(label_text)));

        DropdownField {
            dropdown,
            wrapper,
            variants,
        }
    }

    pub fn value(&self) -> &T {
        &self.variants[self.dropdown.selected() as usize]
    }

    pub fn set_orientation(&self, orientation: gtk::Orientation) {
        self.wrapper.set_orientation(orientation);
    }
}

impl<T> FormField for DropdownField<T> {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.wrapper
    }
}

pub struct SliderField {
    scale: gtk::Scale,
    wrapper: GBox,
}

impl SliderField {
    pub fn new(
        label: Option<&str>,
        orientation: Orientation,
        min: usize,
        max: usize,
        step: usize,
        default_value: usize
    ) -> Self {
        let scale = gtk::Scale::with_range(orientation, min as f64, max as f64, step as f64);
        scale.set_value(default_value as f64);
        scale.set_width_request(100);

        let wrapper = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        wrapper.append(&scale);
        label.map(|label_text| wrapper.prepend(&new_label(label_text)));

        SliderField {
            scale,
            wrapper,
        }
    }

    pub fn value(&self) -> usize {
        self.scale.value() as usize
    }

    pub fn set_value(&self, new_value: usize) {
        self.scale.set_value(new_value as f64);
    }

    pub fn set_changed_hook<F: Fn(usize) + 'static>(&self, f: F)
    {
        self.scale.connect_value_changed(move |b| {
            f(b.value() as usize)
        });
    }

    pub fn set_population_orientation(&self, orientation: gtk::Orientation) {
        self.wrapper.set_orientation(orientation);
    }
}

impl FormField for SliderField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.wrapper
    }
}

/// Wrapper for `gtk::Label` that implements `FormField`
pub struct LabelField {
    label: gtk::Label,
}

impl LabelField {
    fn new(initial_text: &str) -> Self {
        LabelField {
            label: gtk::Label::new(Some(initial_text)),
        }
    }

    fn set_text(&self, new_text: &str) {
        self.label.set_label(new_text);
    }
}

impl FormField for LabelField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.label
    }
}

pub struct ExpandJustificationField {
    buttons: Rc<RefCell<Vec<gtk::ToggleButton>>>,
    wrapper: GBox,
}

impl ExpandJustificationField {
    const VALS: [ExpandJustification; 9] = [
        ExpandJustification::TopLeft,
        ExpandJustification::TopCenter,
        ExpandJustification::TopRight,
        ExpandJustification::MiddleLeft,
        ExpandJustification::MiddleCenter,
        ExpandJustification::MiddleRight,
        ExpandJustification::BottomLeft,
        ExpandJustification::BottomCenter,
        ExpandJustification::BottomRight,
    ];

    pub fn new(initial_value: ExpandJustification) -> Self {
        let wrapper = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        let inner_wrapper = GBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();

        wrapper.append(&gtk::Label::new(Some("Justification:")));
        wrapper.append(&inner_wrapper);

        let mut buttons: Rc<RefCell<Vec<gtk::ToggleButton>>> = Rc::new(RefCell::new(vec![]));

        for r in 0..3 {
            let row_widget = GBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(4)
                .build();

            for c in 0..3 {
                let b = gtk::ToggleButton::builder()
                    .active(Self::VALS[r * 3 + c] == initial_value)
                    .build();

                b.connect_clicked(clone!(@strong buttons => move |b| {
                    for other in buttons.borrow().iter() {
                        if other != b {
                            other.set_active(false);
                        }
                    }

                    b.set_active(true);
                }));

                row_widget.append(&b);
                buttons.borrow_mut().push(b);
            }

            inner_wrapper.append(&row_widget);
        }

        ExpandJustificationField {
            buttons,
            wrapper,
        }
    }

    pub fn value(&self) -> ExpandJustification {
        for (idx, b) in self.buttons.borrow().iter().enumerate() {
            if b.is_active() {
                return Self::VALS[idx].clone();
            }
        }

        panic!("No button in ExpandJustificationField is toggled");
    }
}

impl FormField for ExpandJustificationField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.wrapper
    }
}

/// A field composed of multiple fields
/// (used solely for avoiding line-breaks).
/// This struct uses linked-list-like construction
/// to allow for compile-time composition.
/// Use the `composite_field!` macro to construct it.
pub struct CompositeField {
    widget: gtk::Box,
}

impl CompositeField {
    /// Combine two fields into one (base-case)
    pub fn from2<L, R>(left: &L, right: &R) -> Self
    where
        L: FormField,
        R: FormField,
    {
        let widget = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .build();

        widget.append(left.outer_widget());
        widget.append(right.outer_widget());

        Self {
            widget
        }
    }

    pub fn append<R: FormField>(self, right: &R) -> Self {
        let widget = self.widget;
        widget.append(right.outer_widget());

        Self {
            widget
        }
    }

    pub fn made_vertical(self) -> Self {
        self.widget.set_orientation(Orientation::Vertical);
        self
    }
}

impl FormField for CompositeField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.widget
    }
}

/// Combines the provided `FormField`s into a single `FormField`
/// (this is useless except for dislpay purposes)
#[macro_export]
macro_rules! composite_field {
    ($first:expr, $second:expr $(, $another:expr)*) => {
        crate::ui::form::CompositeField::from2($first, $second)
        $(
            .append($another);
        )*
    };
}


#[macro_export]
macro_rules! vertical_composite_field {
    ($($args:expr),*) => {
        crate::composite_field!($($args),*)
            .made_vertical()
    };
}

pub struct Form {
    widget: gtk::Box,
}

impl Form {
    pub fn builder() -> FormBuilder {
        FormBuilder::new()
    }

    pub fn widget(&self) -> &impl IsA<Widget> {
        &self.widget
    }
}

pub trait FormBuilderIsh {
    fn with_field(self, new_field: &impl FormField) -> Self;
}

pub struct FormBuilder {
    title: Option<String>,
    widget: gtk::Box,

}

impl FormBuilderIsh for FormBuilder {
    fn with_field(self, new_field: &impl FormField) -> Self {
        self.widget.append(new_field.outer_widget());
        self
    }
}

impl FormBuilder {
    fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();

        FormBuilder {
            title: None,
            widget,
        }
    }

    pub fn build(self) -> Form {
        if let Some(title_str) = self.title {
            let title_label = Label::builder()
                .label(title_str)
                .build();

            self.widget.prepend(&title_label);
        }

        Form {
            widget: self.widget,
        }
    }

    pub fn title(mut self, new_title: &str) -> Self {
        self.title = Some(String::from(new_title));
        self
    }

    pub fn orientation(self, orientation: gtk::Orientation) -> Self {
        self.widget.set_orientation(orientation);
        self
    }

    pub fn spacing(self, n: i32) -> Self {
        self.widget.set_spacing(n);
        self
    }
}

pub struct FlowForm {
    widget: gtk::FlowBox,
}

impl FlowForm {
    pub fn builder() -> FlowFormBuilder {
        FlowFormBuilder::new()
    }

    pub fn widget(&self) -> &impl IsA<Widget> {
        &self.widget
    }
}

pub struct FlowFormBuilder {
    title: Option<String>,
    widget: gtk::FlowBox,
}


impl FormBuilderIsh for FlowFormBuilder {
    fn with_field(self, new_field: &impl FormField) -> Self {
        self.widget.append(new_field.outer_widget());
        self
    }
}

impl FlowFormBuilder {
    fn new() -> Self {
        let widget = gtk::FlowBox::builder()
            .orientation(Orientation::Vertical)
            .row_spacing(4)
            .column_spacing(4)
            .selection_mode(gtk::SelectionMode::None)
            .build();

        FlowFormBuilder {
            title: None,
            widget,
        }
    }

    pub fn build(self) -> FlowForm {
        if let Some(title_str) = self.title {
            let title_label = Label::builder()
                .label(title_str)
                .build();

            self.widget.prepend(&title_label);
        }

        FlowForm {
            widget: self.widget,
        }
    }

    pub fn title(mut self, new_title: &str) -> Self {
        self.title = Some(String::from(new_title));
        self
    }

    pub fn orientation(self, orientation: gtk::Orientation) -> Self {
        self.widget.set_orientation(orientation);
        self
    }
}
