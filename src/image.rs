use gtk::cairo::{ImageSurface, SurfacePattern, Format, Filter, Mesh, MeshCorner};
use gtk::cairo;
use gtk::glib::translate::ToGlibPtr;

#[derive(Clone)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel {
    fn blend_onto(above: &Pixel, below: &Pixel) -> Pixel {
        let o = above.a as f64 / 255.0;
        let t = 1.0 - o;
        Pixel {
            r: (above.r as f64 * o + below.r as f64 * t) as u8,
            g: (above.g as f64 * o + below.g as f64 * t) as u8,
            b: (above.b as f64 * o + below.b as f64 * t) as u8,
            a: std::cmp::max(above.a, below.a),
        };
        BLACK
    }
}

#[derive(Clone)]
pub struct Image {
    pub pixels: Vec<Vec<Pixel>>,
}

const TRANS: Pixel = Pixel {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

const BLACK: Pixel = Pixel {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};

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

const GRAY: Pixel = Pixel {
    r: 211,
    g: 211,
    b: 211,
    a: 255,
};

const DARK_GRAY: Pixel = Pixel {
    r: 229,
    g: 229,
    b: 229,
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

pub fn mk_test_brush() -> Image {
    Image {
        pixels: vec![
            vec![TRANS, TRANS, BLACK, TRANS, TRANS],
            vec![TRANS, BLACK, BLACK, BLACK, TRANS],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![TRANS, BLACK, BLACK, BLACK, TRANS],
            vec![TRANS, TRANS, BLACK, TRANS, TRANS],
        ],
    }
}

pub fn mk_transparent_pattern() -> SurfacePattern {
    let img = Image {
        pixels: vec![vec![GRAY, DARK_GRAY], vec![DARK_GRAY, GRAY]],
    };

    let res = img.to_surface_pattern();
    res.set_extend(cairo::Extend::Repeat);
    res
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

    // draw `other` at (x, y)
    pub fn sample(&mut self, other: &Image, x: i32, y: i32) {
        for i in 0..other.pixels.len() {
            for j in 0..other.pixels[0].len() {
                let ip = i as i32 + y;
                let jp = j as i32 + x;

                if ip < 0 || jp < 0 || ip >= self.pixels.len() as i32 || jp >= self.pixels[0].len() as i32 {
                    continue;
                }

                let ip = ip as usize;
                let jp = jp as usize;

                self.pixels[ip][jp] = Pixel::blend_onto(&self.pixels[ip][jp], &other.pixels[i][j]);
            }
        }
    }

    pub fn width(&self) -> i32 {
        self.pixels[0].len() as i32
    }

    pub fn height(&self) -> i32 {
        self.pixels.len() as i32
    }
}
