pub mod undo;

extern crate image as image_lib;

use image_lib::io::Reader as ImageReader;
use image_lib::{DynamicImage, ImageError, RgbaImage, ImageFormat as ImgFmt};
use std::hash::Hash;
use std::io::Error;
use std::path::Path;
use std::collections::HashMap;

use gtk::cairo::{ImageSurface, SurfacePattern, Format, Filter};
use gtk::cairo;

#[derive(Clone)]
pub struct Pixel {
    // the order of the fields is in the unsafe cast in Image::to_file
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Pixel { r, g, b, a: 255, }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self{
        Pixel { r, g, b, a, }
    }

    pub fn to_drawable(&self) -> DrawablePixel {
        DrawablePixel::from_rgba(self.r, self.g, self.b, self.a)
    }

    fn blend_onto(above: &Pixel, below: &Pixel) -> Pixel {
        let o = above.a as f64 / 255.0;
        let t = 1.0 - o;
        Pixel::from_rgba((above.r as f64 * o + below.r as f64 * t) as u8,
                         (above.g as f64 * o + below.g as f64 * t) as u8,
                         (above.b as f64 * o + below.b as f64 * t) as u8,
                         std::cmp::max(above.a, below.a))
    }
}

const TRANS: Pixel = Pixel::from_rgba(0, 0, 0, 0);
const BLACK: Pixel = Pixel::from_rgb(0, 0, 0);
const BLUE: Pixel = Pixel::from_rgb(0, 0, 255);
const GREEN: Pixel = Pixel::from_rgb(0, 255, 0);
const GRAY: Pixel = Pixel::from_rgb(211, 211, 211);
const DARK_GRAY: Pixel = Pixel::from_rgb(229, 229, 229);

#[derive(Clone)]
pub struct Image {
    pixels: Vec<Pixel>,
    width: usize,
    height: usize,
}

pub fn mk_test_image() -> UnifiedImage {
    let mut pixels = vec![vec![BLUE; 400]; 400];

    for i in 0..400 {
        for j in 0..400 {
            if i % 2 == 0 || j % 2 == 0 {
                pixels[i][j] = GREEN;
            }
        }
    }

    UnifiedImage::from_image(Image::from_pixels(pixels))
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

pub fn mk_transparent_checkerboard() -> DrawableImage {
    DrawableImage::from_image(&Image::from_pixels(vec![vec![GRAY, DARK_GRAY], vec![DARK_GRAY, GRAY]]))
}

impl Image {
    fn from_pixels(pixels: Vec<Vec<Pixel>>) -> Image {
        Image {
            width: pixels[0].len(),
            height: pixels.len(),
            pixels: pixels.into_iter().flatten().collect::<Vec<_>>(),
        }
    }

    pub fn pixels(&self) -> &Vec<Pixel> {
        &self.pixels
    }

    pub fn into_pixels(self) -> Vec<Pixel> {
        self.pixels
    }
}

// i/o
impl Image {
    pub fn from_file(path: &Path) -> Result<Image, Error> {
        match ImageReader::open(path)?.decode() {
            Ok(dyn_img) => {
                let rgba = dyn_img.to_rgba8();
                let pixels = rgba.enumerate_pixels().map(|(x, y, rgba)| {
                    Pixel::from_rgba(rgba.0[0], rgba.0[1], rgba.0[2], rgba.0[3])
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

// DrawablePixel / DrawableImage
// same as Pixel/Image, but with pre-multiplied-alpha;
// this is necessary for drawing in cairo

#[derive(Clone)]
struct DrawablePixel {
    // order of the fields corresponds to cairo::Format::ARgb32
    // (this struct is used for directly rendering the cairo pattern)
    b: u8,
    g: u8,
    r: u8,
    a: u8,
}

impl DrawablePixel {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self{
        let af = a as f64 / 255.0;
        DrawablePixel {
            r: (r as f64 * af) as u8,
            g: (g as f64 * af) as u8,
            b: (b as f64 * af) as u8,
            a,
        }
    }
}

#[derive(Clone)]
pub struct DrawableImage {
    pixels: Vec<DrawablePixel>,
    width: usize,
    height: usize,
}

impl DrawableImage {
    pub fn from_image(image: &Image) -> Self {
        DrawableImage {
            width: image.width,
            height: image.height,
            pixels: image.pixels.iter().map(|p| p.to_drawable()).collect::<Vec<_>>(),
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
}

// UnifiedImage = Image + DrawableImage
// Image has all the necessary information, but a DrawableImage
// is kept to avoid re-computation on each draw.
// All data is read from the Image, but writes are applied to both
#[derive(Clone)]
pub struct UnifiedImage {
    image: Image,
    drawable: DrawableImage,
    pix_modified_since_draw: HashMap<usize, (Pixel)>,
    pix_modified_since_save: HashMap<usize, (Pixel, Pixel)>,
    save_image_before_overwritten: Option<Image>,
}

impl UnifiedImage {
    pub fn new(image: Image, drawable: DrawableImage) -> Self {
        assert!(image.width == drawable.width);
        assert!(image.height == drawable.height);

        UnifiedImage {
            image,
            drawable,
            pix_modified_since_draw: HashMap::new(),
            pix_modified_since_save: HashMap::new(),
            save_image_before_overwritten: None,
        }
    }

    pub fn from_image(image: Image) -> Self {
        UnifiedImage {
            drawable: DrawableImage::from_image(&image),
            image,
            pix_modified_since_draw: HashMap::new(),
            pix_modified_since_save: HashMap::new(),
            save_image_before_overwritten: None,
        }
    }

    // draw `other` at (x, y)
    pub fn sample(&mut self, other: &Image, x: i32, y: i32) {
        for i in 0..other.height {
            for j in 0..other.width {
                let ip = i as i32 + y;
                let jp = j as i32 + x;

                if let Some(p) = self.try_pix_at(ip, jp) {
                    *p = Pixel::blend_onto(&other.pixels[(i * other.width + j) as usize], &p);
                }
            }
        }
    }

    pub fn pix_at(&mut self, r: i32, c: i32) -> &mut Pixel {
        let i = (r * self.width() + c) as usize;
        // only bother recording modified pixel if image hasn't been overwritten
        if let None = self.save_image_before_overwritten {
            self.pix_modified_since_draw.entry(i).or_insert(self.image.pixels[i].clone());
        }
        &mut self.image.pixels[i]
    }

    pub fn try_pix_at(&mut self, r: i32, c: i32) -> Option<&mut Pixel> {
        if r < 0 || c < 0 || r as usize >= self.image.height || c as usize >= self.image.width {
            None
        } else {
            let i = (r * self.width() + c) as usize;
            if let None = self.save_image_before_overwritten {
                self.pix_modified_since_draw.entry(i).or_insert(self.image.pixels[i].clone());
            }
            Some(&mut self.image.pixels[i])
        }
    }

    pub fn width(&self) -> i32 {
        self.image.width as i32
    }

    pub fn height(&self) -> i32 {
        self.image.height as i32
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn set_image(&mut self, image: Image)  {
        self.drawable = DrawableImage::from_image(&image);

        if let None = self.save_image_before_overwritten {
            self.save_image_before_overwritten = Some(std::mem::replace(&mut self.image, image));
        } else {
            self.image = image;
        }

        self.pix_modified_since_save.clear();
        self.pix_modified_since_draw.clear();
    }

    fn update_pix_modified_dict(dict: &mut HashMap<usize, (Pixel, Pixel)>, i: usize, before: &Pixel, after: &Pixel) {
        let entry = dict.entry(i);
        if let std::collections::hash_map::Entry::Occupied(mut oe) = entry {
            oe.insert((oe.get().0.clone(), after.clone()));
        } else {
            dict.insert(i, (before.clone(), after.clone()));
        }
    }

    pub fn drawable(&mut self) -> &mut DrawableImage {
        for (i, p_before) in self.pix_modified_since_draw.iter() {
            self.drawable.pixels[*i] = self.image.pixels[*i].to_drawable();
            Self::update_pix_modified_dict(&mut self.pix_modified_since_save, *i, p_before, &self.image.pixels[*i]);
        }

        self.pix_modified_since_draw.clear();
        &mut self.drawable
    }

    pub fn get_and_reset_modified(&mut self) -> (HashMap<usize, (Pixel, Pixel)>, Option<Image>) {
        for (i, p_before) in self.pix_modified_since_draw.iter() {
            // Self::update_pix_modified_since_save(&mut self.pix_modified_since_save, *i, p_before, &self.image.pixels[*i]);
            self.pix_modified_since_save.entry(*i).or_insert((p_before.clone(), self.image.pixels[*i].clone()));
        }

        let mut mod_pix = HashMap::new();
        std::mem::swap(&mut mod_pix, &mut self.pix_modified_since_save);
        let mut save_img = None;
        std::mem::swap(&mut save_img, &mut self.save_image_before_overwritten);

        (mod_pix, save_img)
    }
}
