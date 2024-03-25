use gtk::cairo::{ImageSurface, SurfacePattern, Format, Filter};
use gtk::cairo;

#[derive(Clone)]
pub struct Pixel {
    b: u8,
    g: u8,
    r: u8,
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
        }
    }
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

#[derive(Clone)]
pub struct Image {
    pixels: Vec<Pixel>,
    width: usize,
    height: usize,
}

pub fn mk_test_image() -> Image {
    let mut pixels = vec![vec![BLUE; 400]; 400];

    for i in 0..400 {
        for j in 0..400 {
            if i % 2 == 0 || j % 2 == 0 {
                pixels[i][j] = GREEN;
            }
        }
    }

    Image::from_pixels(pixels)
}

pub fn mk_test_brush() -> Image {
    Image::from_pixels(vec![
            vec![TRANS, BLACK, BLACK, BLACK, TRANS],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![BLACK, BLACK, BLACK, BLACK, BLACK],
            vec![TRANS, BLACK, BLACK, BLACK, TRANS],
        ])
}

pub fn mk_transparent_pattern() -> SurfacePattern {
    let mut img = Image::from_pixels(vec![vec![GRAY, DARK_GRAY], vec![DARK_GRAY, GRAY]]);

    let res = img.to_surface_pattern();
    res.set_extend(cairo::Extend::Repeat);
    res
}

impl Image {
    fn from_pixels(pixels: Vec<Vec<Pixel>>) -> Image {
        Image {
            width: pixels[0].len(),
            height: pixels.len(),
            pixels: pixels.into_iter().flatten().collect::<Vec<_>>(),
        }
    }

    pub fn to_surface_pattern(&mut self) -> SurfacePattern {
        unsafe {
            let (_, u8_slice, _) = self.pixels.align_to_mut::<u8>();

            let image_surface = ImageSurface::create_for_data_unsafe(u8_slice.as_mut_ptr(),
                                                                            Format::ARgb32,
                                                                            self.width as i32,
                                                                            self.height as i32,
                                                                            4 * self.width as i32) .unwrap();

            let surface_pattern = SurfacePattern::create(image_surface);
            surface_pattern.set_filter(Filter::Fast);

            surface_pattern
        }
    }

    // draw `other` at (x, y)
    pub fn sample(&mut self, other: &Image, x: i32, y: i32) {
        for i in 0..other.height() {
            for j in 0..other.width() {
                let ip = i as i32 + y;
                let jp = j as i32 + x;

                if let Some(p) = self.pix_at(ip, jp) {
                    *p = Pixel::blend_onto(&other.pixels[(i * other.width() + j) as usize], &p);
                }
            }
        }
    }

    pub fn pix_at(&mut self, r: i32, c: i32) -> Option<&mut Pixel> {
        if r < 0 || c < 0 || r as usize >= self.height || c as usize >= self.width {
            None
        } else {
            Some(&mut self.pixels[r as usize * self.width + c as usize])
        }
    }

    pub fn width(&self) -> i32 {
        self.width as i32
    }

    pub fn height(&self) -> i32 {
        self.height as i32
    }
}
