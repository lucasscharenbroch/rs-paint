use super::*;

extern crate image as image_lib;
use image_lib::io::Reader as ImageReader;
use image_lib::{DynamicImage, RgbaImage, ImageFormat as ImgFmt};
use std::mem;
use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use serde_derive::{Serialize, Deserialize};

impl Image {
    pub fn from_path(path: &Path) -> Result<Image, String> {
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

/// A `FusedLayeredImage`, without all the drawables (and the
/// machinery to support them). Used solely for i/o
#[derive(Serialize, Deserialize)]
pub struct LayeredImage {
    base_layer: Layer,
    other_layers: Vec<Layer>,
}

impl LayeredImage {
    pub fn gen_entire_blended_image(&self) -> Image {
        let mut res = self.base_layer.image.clone();
        for layer in self.other_layers.iter() {
            res.blend_under(&layer.image)
        }

        res
    }

    pub fn from_path(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|err| err.to_string())?;
        let reader = BufReader::new(file);

        serde_cbor::from_reader(reader).map_err(|err| err.to_string())
    }

    pub fn to_file(&self, path: &Path) -> Result<(), String> {
        let file = File::create(path).map_err(|err| err.to_string())?;
        let writer = BufWriter::new(file);

        serde_cbor::to_writer(writer, self).map_err(|err| err.to_string())
    }
}

impl FusedLayeredImage {
    pub fn unfused(&self) -> LayeredImage {
        LayeredImage {
            base_layer: self.base_layer.unfused(),
            other_layers: self.other_layers.iter()
                .map(|layer| layer.unfused())
                .collect::<Vec<_>>()
        }
    }

    pub fn from_layered_image(layered_imge: LayeredImage) -> Self {
        FusedLayeredImage {
            drawable: DrawableImage::from_image(&layered_imge.gen_entire_blended_image()),
            base_layer: FusedLayer::from_layer(layered_imge.base_layer),
            other_layers: layered_imge.other_layers.into_iter()
                .map(|layer| FusedLayer::from_layer(layer))
                .collect::<Vec<_>>(),
            active_layer_index: LayerIndex::BaseLayer,
            pix_modified_since_draw: HashMap::new(),
            pix_modified_since_save: HashMap::new(),
        }
    }
}
