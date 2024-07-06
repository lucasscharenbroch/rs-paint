use super::*;

extern crate image as image_lib;
use image_lib::io::Reader as ImageReader;
use image_lib::{DynamicImage, RgbaImage, ImageFormat as ImgFmt};
use std::mem;
use std::path::Path;

// i/o
impl Image {
    pub fn from_file(path: &Path) -> Result<Image, String> {
        match ImageReader::open(path).map_err(|e| e.to_string())?.decode() {
            Ok(dyn_img) => {
                let rgba = dyn_img.into_rgba8();
                let (width, height) = rgba.dimensions();
                let (width, height) = (width as usize, height as usize);
                let n_pix = rgba.len() / 4;

                let pixels: Vec<Pixel> = unsafe {
                    let mut rgba = mem::ManuallyDrop::new(rgba);
                    Vec::from_raw_parts(rgba.as_mut_ptr() as *mut Pixel, n_pix, n_pix)
                };

                Ok(Image {
                    height,
                    width,
                    pixels,
                })
            },
            Err(img_err) => Err(img_err.to_string()),
        }
    }

    pub fn to_file(&self, path: &Path) -> Result<(), String> {
        let ext = path.extension()
            .and_then(|os| os.to_str())
            .map(|s| s.to_ascii_lowercase());

        let format = if let Some(s) = ext {
            match s.as_str() {
                "png" => ImgFmt::Png,
                "jpg" | "jpeg" => ImgFmt::Jpeg,
                "gif" => ImgFmt::Gif,
                "webp" => ImgFmt::WebP,
                "bmp" => ImgFmt::Bmp,
                _ => return Err(format!("Invalid file extension: `.{}`", s)),
            }
        } else {
            return Err(String::from("Can't determine image type (no extension)"));
        };

        unsafe {
            let (_, u8_slice, _) = self.pixels.align_to::<u8>();
            let rgba = RgbaImage::from_raw(self.width as u32, self.height as u32, u8_slice.to_vec())
                .ok_or("Failed to make RgbaImage from image buffer")?;
            match format {
                ImgFmt::Jpeg =>  {
                    // jpg doesn't support alpha
                    let rgb = DynamicImage::from(rgba).to_rgb8();
                    rgb.save_with_format(path, format).map_err(|e| e.to_string())
                }
                _ => rgba.save_with_format(path, format).map_err(|e| e.to_string())
            }
        }
    }
}
