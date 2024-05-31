use gtk::{prelude::*, Box as GBox, Orientation, Widget, Entry, Label};
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
    pub fn new(default_text: &str, phantom_text: &str, label: Option<&str>) -> Self {
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
    default_value: usize,
    label: Option<String>,
}

pub struct ColorField {
    default_value: RGBA,
    label: Option<String>,
}

pub struct RadioField {
    variants: Vec<String>,
    default_variant_idx: usize,
}

pub struct CheckboxField {
    is_checked: bool,
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
