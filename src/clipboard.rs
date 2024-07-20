use crate::image::{Image, Pixel};

/// Wrapper for arboard::Clipboard; using the wrapper
/// to make it easiser to tweak the api/ add side-effects
/// (facade pattern)
/// Fails gracefully when the clipboard is unavailable.
pub struct Clipboard {
    clipboard: Option<arboard::Clipboard>,
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
            clipboard
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

    pub fn set_image(&mut self, image: &Image) {
    }
}