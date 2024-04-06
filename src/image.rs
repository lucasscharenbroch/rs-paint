extern crate image as image_lib;

use image_lib::io::Reader as ImageReader;
use image_lib::{DynamicImage, ImageError, RgbaImage, ImageFormat as ImgFmt};
use std::io::Error;
use std::path::Path;
use std::fs::File;

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

pub fn mk_transparent_checkerboard() -> Image {
    Image::from_pixels(vec![vec![GRAY, DARK_GRAY], vec![DARK_GRAY, GRAY]])
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
                                                                            4 * self.width as i32).unwrap();

            let surface_pattern = SurfacePattern::create(image_surface);
            surface_pattern.set_filter(Filter::Fast);

            surface_pattern
        }
    }

    pub fn to_repeated_surface_pattern(&mut self) -> SurfacePattern {
        let res = self.to_surface_pattern();
        res.set_extend(cairo::Extend::Repeat);
        res
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

// i/o
impl Image {
    pub fn from_file(path: &Path) -> Result<Image, Error> {
        match ImageReader::open(path)?.decode() {
            Ok(dyn_img) => {
                let rgba = dyn_img.to_rgba8();
                let pixels = rgba.enumerate_pixels().map(|(x, y, rgba)| {
                    Pixel {
                        r: rgba.0[0],
                        g: rgba.0[1],
                        b: rgba.0[2],
                        a: rgba.0[3],
                    }
                }).collect::<Vec<_>>();

                let img = Image {
                    height: dyn_img.height() as usize,
                    width: dyn_img.width() as usize,
                    pixels,
                };

                Ok(img)
            },
            Err(err) => {
                panic!("Error when loading image: {:?}", err);
            },
        }
    }

    pub fn to_file(&self, path: &Path) -> Result<(), ImageError> {
        let mut out_file = File::create(path).unwrap();

        let ext = path.extension()
            .and_then(|os| os.to_str())
            .map(|s| s.to_ascii_lowercase());

        let format = match ext.as_ref().map(|s| s.as_str()) {
            Some("png") => ImgFmt::Png,
            Some("jpg") | Some("jpeg") => ImgFmt::Jpeg,
            Some("gif") => ImgFmt::Gif,
            Some("webp") => ImgFmt::WebP,
            Some("bmp") => ImgFmt::Bmp,
            _ => panic!("Invalid file extension: {:?}", ext),
        };

        unsafe {
            let (_, u8_slice, _) = self.pixels.align_to::<u8>();
            let rgba = RgbaImage::from_raw(self.width as u32, self.height as u32, u8_slice.to_vec()).unwrap();
            match format {
                ImgFmt::Jpeg =>  {
                    // jpg doesn't support alpha
                    let rgb = DynamicImage::from(rgba).to_rgb8();
                    rgb.save_with_format(path, format)
                }
                _ => rgba.save_with_format(path, format)
            }
        }
    }
}
