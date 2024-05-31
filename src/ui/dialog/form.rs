use gtk::{prelude::*, Box as GBox, CheckButton, ColorDialog, ColorDialogButton, Entry, Label, Orientation, SpinButton, Widget};
use gtk::gdk::RGBA;
use gtk::glib::object::IsA;

trait FormField {
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

        if let Some(label_text) = label {
            let label = Label::builder()
                .label(label_text)
                .build();

            wrapper.prepend(&label);
        }

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
        num_entry.set_value(default_value as f64);

        let wrapper = GBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        wrapper.append(&num_entry);

        if let Some(label_text) = label {
            let label = Label::builder()
                .label(label_text)
                .build();

            wrapper.prepend(&label);
        }

        NaturalField {
            num_entry,
            wrapper,
        }
    }

    pub fn value(&self) -> usize {
        self.num_entry.value() as usize
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

        if let Some(label_text) = label {
            let label = Label::builder()
                .label(label_text)
                .build();

            wrapper.prepend(&label);
        }

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
}

impl FormField for CheckboxField {
    fn outer_widget(&self) -> &impl IsA<Widget> {
        &self.button
    }
}

pub struct RadioField {
    variants: Vec<String>,
    default_variant_idx: usize,
}


pub struct Form {
    title: Option<String>,
    widget: GBox,
}

impl Form {
    pub fn builder() -> FormBuilder {
        FormBuilder::new()
    }

    pub fn widget(&self) -> &impl IsA<Widget> {
        &self.widget
    }
}

pub struct FormBuilder {
    form: Form,
}

impl FormBuilder {
    fn new() -> Self {
        let widget = GBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();

        let form = Form {
            title: None,
            widget,
        };

        FormBuilder {
            form,
        }
    }

    pub fn build(self) -> Form {
        self.form
    }

    pub fn title(mut self, new_title: &str) -> Self {
        self.form.title = Some(String::from(new_title));
        self
    }

    pub fn with_field(mut self, new_field: &impl FormField) -> Self {
        self.form.widget.append(new_field.outer_widget());
        self
    }
}
