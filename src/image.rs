use gtk::cairo::ImageSurface;
use gtk::cairo::Format;

#[derive(Clone)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Clone)]
pub struct Image {
    pub pixels: Vec<Vec<Pixel>>,
}

const blue: Pixel = Pixel {
    r: 0,
    g: 0,
    b: 255,
    a: 255,
};

pub fn mk_test_image() -> Image {
    return Image {
        pixels: vec![vec![blue; 400]; 400],
    };
}

impl Image {
    fn to_u8_vec(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|row| row
                            .iter()
                            .flat_map(|pix| vec![pix.b, pix.g, pix.r, pix.a])
                            .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    }

    pub fn to_surface(&self) -> ImageSurface {
        let height = self.pixels.len();
        let width = self.pixels[0].len();

        ImageSurface::create_for_data(self.to_u8_vec(), Format::ARgb32, width as i32, height as i32, 4 * width as i32).unwrap()
    }

    pub fn width(&self) -> i32 {
        self.pixels[0].len() as i32
    }

    pub fn height(&self) -> i32 {
        self.pixels.len() as i32
    }
}
