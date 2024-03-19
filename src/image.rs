use gtk::cairo::{ImageSurface, SurfacePattern, Format, Filter};
use gtk::cairo;
use gtk::glib::translate::ToGlibPtr;

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

const BLUE: Pixel = Pixel {
    r: 0,
    g: 0,
    b: 255,
    a: 255,
};

const GREEN: Pixel = Pixel {
    r: 0,
    g: 255,
    b: 0,
    a: 255,
};

pub fn mk_test_image() -> Image {
    let mut pixels = vec![vec![BLUE; 400]; 400];
        for i in 0..400 {
            for j in 0..400 {
                if i % 2 == 0 || j % 2 == 0 {
                    pixels[i][j] = GREEN;
                }
            }
        }

    return Image {
        pixels
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

    pub fn to_surface_pattern(&self) -> SurfacePattern {
        let height = self.pixels.len();
        let width = self.pixels[0].len();
        let image_surface = ImageSurface::create_for_data(self.to_u8_vec(), Format::ARgb32, width as i32, height as i32, 4 * width as i32).unwrap();

        let surface_pattern = SurfacePattern::create(image_surface);
        surface_pattern.set_filter(Filter::Fast);
        surface_pattern
    }

    pub fn width(&self) -> i32 {
        self.pixels[0].len() as i32
    }

    pub fn height(&self) -> i32 {
        self.pixels.len() as i32
    }
}
