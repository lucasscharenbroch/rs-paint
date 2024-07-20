use crate::image::{Image, ImageLike, Pixel};

use std::borrow::Cow;

/// Wrapper for arboard::Clipboard; using the wrapper
/// to make it easiser to tweak the api/ add side-effects
/// (facade pattern)
/// Fails gracefully when the clipboard is unavailable.
pub struct Clipboard {
    clipboard: Option<arboard::Clipboard>,
    copied_image: Option<Image>,
}

impl Clipboard {
    pub fn new() -> Self {
        let clipboard = arboard::Clipboard::new()
            .map(|c| Some(c))
            .unwrap_or(None);

        if clipboard.is_none() {
            eprintln!("Failed to load clipboard");
        }

        Clipboard {
            clipboard,
            copied_image: None,
        }
    }

    pub fn get_image(&mut self) -> Option<Image> {
        self.clipboard.as_mut().and_then(|clipboard| {
            if let Ok(image_data) = clipboard.get_image() {
                let length = image_data.width * image_data.height;
                let capacity = length;

                let pixels = unsafe {
                    let bytes = image_data.bytes.into_owned();
                    let mut bytes = std::mem::ManuallyDrop::new(bytes);
                    Vec::from_raw_parts(bytes.as_mut_ptr() as *mut Pixel, length, capacity)
                };

                Some(Image::new(pixels, image_data.width, image_data.height))
            } else {
                None
            }
        })
    }

    pub fn set_image(&mut self, image: Image) {
        self.clipboard.as_mut().map(|clipboard| {
            let width = image.width();
            let height = image.height();
            self.copied_image = Some(image);

            let bytes = unsafe {
                let ptr = self.copied_image.as_ref().unwrap().pixels().as_slice().as_ptr() as *const u8;
                let slice = std::slice::from_raw_parts(ptr, width * height * 4);
                Cow::from(slice)
            };

            let image_data = arboard::ImageData {
                width,
                height,
                bytes,
            };

            let _ = clipboard.set_image(image_data);
        });
    }
}