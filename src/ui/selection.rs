use gtk::cairo::Context;

pub enum Selection {
    Rectangle(usize, usize, usize, usize), // x, y, w, h
    NoSelection
}

impl Selection {
    pub fn draw_outline(&self, cr: &Context) {
        match self {
            Self::Rectangle(x, y, w, h) => {
                println!("draw select {x} {y} {w} {h}"); // TODO
            },
            Self::NoSelection => (),
        }
    }
}
