use gtk::gdk::RGBA;

pub enum FieldType {
    Text(String, String), // (default text, phantom text)
    Natural(usize),
    Integral(i32),
    Numeric(f64),
    Color(RGBA),
    Radio(Vec<String>, usize), // (variants, index of default (out of bounds for none))
    Check(bool), // is checked?
}

pub struct Field {
    field_type: FieldType,
    label: Option<String>,
}

pub struct Form {
    title: Option<String>,
    fields: Vec<Field>,
}

impl Form {
    pub fn builder() -> FormBuilder {
        FormBuilder::new()
    }
}

pub struct FormBuilder {
    form: Form,
}

impl FormBuilder {
    pub fn new() -> Self {
        let form = Form {
            title: None,
            fields: vec![],
        };

        FormBuilder {
            form,
        }
    }

    pub fn build(self) -> Form {
        self.form
    }

    pub fn title(mut self, new_title: String) -> Self {
        self.form.title = Some(new_title);
        self
    }

    pub fn with_field(mut self, new_field: Field) -> Self {
        self.form.fields.push(new_field);
        self
    }
}
