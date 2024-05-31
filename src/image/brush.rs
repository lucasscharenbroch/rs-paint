use super::{Image, Pixel};
use super::blend::BlendingMode;

use gtk::gdk::RGBA;
use gtk::gio::ListStore;
use gtk::{prelude::*, Box as GBox, DropDown, Orientation, StringObject, SpinButton};

#[derive(Clone, Copy, PartialEq)]
pub enum BrushType {
    Square,
    Round,
    Dither,
    Pen, // faded round
    Crayon, // faded dither
}

#[derive(PartialEq)]
struct BrushProperties {
    brush_type: BrushType,
    radius: u8,
    color: RGBA,
}

pub struct Brush {
    props: BrushProperties,
    pub image: Image,
}

const TRANS: Pixel = Pixel::from_rgba(0, 0, 0, 0);

fn mk_square_brush_image(n: u8, color: RGBA) -> Image {
    let p = Pixel::from_rgba_struct(color);
    Image::from_pixels(vec![vec![p; n as usize]; n as usize])
}

fn mk_round_brush_image(n: u8, fade: bool, dither:bool, color: RGBA) -> Image {
    let p = Pixel::from_rgba_struct(color);
    let n = n as usize;
    let mut pix = vec![vec![TRANS; n]; n];

    const CIRC_THRESH: f64 = 0.3;

    for i in 0..n {
        for j in 0..n {
            let x = (n as f64 / 2.0) - (j as f64);
            let y = (n as f64 / 2.0) - (i as f64);
            let dist = (x * x + y * y).sqrt();
            let opacity = 1.0 - (dist / (n as f64 / 2.0));

            if opacity > CIRC_THRESH  && (!dither || i % 2 == j % 2) {
                if fade {
                    pix[i][j] = p.scale_alpha(opacity);
                } else {
                    pix[i][j] = p.clone();
                }
            }
        }
    }

    Image::from_pixels(pix)
}

impl Brush {
    fn image_from_props(props: &BrushProperties) -> Image {
        let r = props.radius;
        match props.brush_type {
            BrushType::Square => mk_square_brush_image(r, props.color),
            BrushType::Round => mk_round_brush_image(r, false, false, props.color),
            BrushType::Dither => mk_round_brush_image(r, false, true, props.color),
            BrushType::Pen => mk_round_brush_image(r, true, false, props.color),
            BrushType::Crayon => mk_round_brush_image(r, true, true, props.color),
        }
    }

    pub fn modify(&mut self, color: RGBA, brush_type: BrushType, radius: u8) {
        let new_props = BrushProperties {
            color,
            brush_type,
            radius,
        };

        if self.props == new_props {
            // no changes necessary
        } else {
            *self = Brush::from_props(new_props)
        }
    }

    pub fn new(color: RGBA, brush_type: BrushType, radius: u8) -> Self {
        let props = BrushProperties {
            color,
            brush_type,
            radius,
        };

        Self::from_props(props)
    }

    fn from_props(props: BrushProperties) -> Self {
        let image = Self::image_from_props(&props);

        Brush {
            props,
            image,
        }
    }
}

pub struct BrushToolbar {
    brush: Brush,
    brush_type: BrushType,
    type_dropdown: DropDown,
    blending_mode_dropdown: DropDown,
    radius_selector: SpinButton,
    widget: GBox,
}

const BRUSH_TYPES_AND_IDS: [(BrushType, &str); 5] = [
    (BrushType::Round, "Round"),
    (BrushType::Square, "Square"),
    (BrushType::Dither, "Dither"),
    (BrushType::Pen, "Pen"),
    (BrushType::Crayon, "Crayon"),
];

const BLENDING_MODES_AND_IDS: [(BlendingMode, &str); 3] = [
    (BlendingMode::Average, "Average"),
    (BlendingMode::Overwrite, "Overwrite"),
    (BlendingMode::Paint, "Paint"),
];

impl BrushToolbar {
    pub fn new(color: RGBA, brush_type: BrushType, radius: u8) -> Self {
        let props = BrushProperties {
            color,
            brush_type,
            radius,
        };

        let type_list = ListStore::new::<StringObject>();

        for (_, id) in BRUSH_TYPES_AND_IDS.iter() {
            type_list.append(&StringObject::new(id))
        }

        let type_dropdown = DropDown::builder()
            .model(&type_list)
            .build();

        let blending_mode_list = ListStore::new::<StringObject>();

        for (_, id) in BLENDING_MODES_AND_IDS.iter() {
            blending_mode_list.append(&StringObject::new(id))
        }

        let blending_mode_dropdown = DropDown::builder()
            .model(&blending_mode_list)
            .build();

        let radius_selector = SpinButton::with_range(1.0, 255.0, 1.0);
        radius_selector.set_value(5.0);

        let widget = GBox::new(Orientation::Horizontal, 5);

        widget.append(&type_dropdown);
        widget.append(&blending_mode_dropdown);
        widget.append(&radius_selector);

        BrushToolbar {
            brush: Brush::from_props(props),
            brush_type,
            type_dropdown,
            blending_mode_dropdown,
            radius_selector,
            widget
        }
    }

    pub fn brush_type(&self) -> BrushType {
        BRUSH_TYPES_AND_IDS[self.type_dropdown.selected() as usize].0
    }

    pub fn radius(&self) -> u8 {
        self.radius_selector.value() as u8
    }

    pub fn get_brush(&mut self, color: RGBA) -> &Brush {
        self.brush.modify(color, self.brush_type(), self.radius());
        &self.brush
    }

    pub fn widget(&self) -> &GBox {
        &self.widget
    }

    pub fn get_blending_mode(&self) -> BlendingMode {
        BLENDING_MODES_AND_IDS[self.blending_mode_dropdown.selected() as usize].0
    }
}
