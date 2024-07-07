pub mod undo;
pub mod brush;
pub mod generate;
pub mod blend;
pub mod transform;
pub mod resize;
pub mod bitmask;
pub mod io;

use std::collections::HashMap;

use gtk::cairo;
use gtk::gdk::RGBA;

use serde_derive::{Serialize, Deserialize};

/// The ambivalent (r, g, b, a) pixel type, used for
/// importing and drawing (it cannot be directly displayed to cairo,
/// though: use `DrawablePixel` (and `DrawableImage`) instead)
#[derive(Clone, Serialize, Deserialize)]
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

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Pixel { r, g, b, a, }
    }

    pub fn from_rgba_struct(color: RGBA) -> Self {
        let r = (color.red() * 255.0) as u8;
        let g = (color.green() * 255.0) as u8;
        let b = (color.blue() * 255.0) as u8;
        let a = (color.alpha() * 255.0) as u8;

        Pixel { r, g, b, a, }
    }

    pub fn to_rgba_struct(&self) -> RGBA {
        RGBA::new(self.r as f32 / 255.0,
                  self.g as f32 / 255.0,
                  self.b as f32 / 255.0,
                  self.a as f32 / 255.0)
    }

    fn to_drawable(&self) -> DrawablePixel {
        DrawablePixel::from_rgba(self.r, self.g, self.b, self.a)
    }

    fn scale_alpha(&self, amount: f64) -> Pixel {
        Pixel::from_rgba(self.r, self.g, self.b, (self.a as f64 * amount) as u8)
    }

    fn blend(above: &Pixel, below: &Pixel) -> Pixel {
        let o = above.a as f64 / 255.0;
        let t = 1.0 - o;
        Pixel::from_rgba((above.r as f64 * o + below.r as f64 * t) as u8,
                         (above.g as f64 * o + below.g as f64 * t) as u8,
                         (above.b as f64 * o + below.b as f64 * t) as u8,
                         std::cmp::max(above.a, below.a))
    }
}

const GRAY: Pixel = Pixel::from_rgb(211, 211, 211);
const DARK_GRAY: Pixel = Pixel::from_rgb(229, 229, 229);

#[derive(Clone, Serialize, Deserialize)]
pub struct Image {
    pixels: Vec<Pixel>,
    width: usize,
    height: usize,
}

pub fn mk_transparent_checkerboard() -> DrawableImage {
    DrawableImage::from_image(&Image::from_pixels(vec![vec![GRAY, DARK_GRAY], vec![DARK_GRAY, GRAY]]))
}

impl Image {
    pub fn new(pixels: Vec<Pixel>, width: usize, height: usize) -> Image {
        assert!(width * height == pixels.len());

        Image {
            pixels,
            width,
            height,
        }
    }

    fn from_pixels(pixels: Vec<Vec<Pixel>>) -> Image {
        Image {
            width: pixels[0].len(),
            height: pixels.len(),
            pixels: pixels.into_iter().flatten().collect::<Vec<_>>(),
        }
    }

    /// Blend `above` onto  `self`, storing the results in `self`
    fn blend_under(&mut self, above: &Image) {
        assert!(self.height == above.height && self.width == above.width);

        for (above_pix, below_pix) in above.pixels.iter().zip(self.pixels.iter_mut()) {
            *below_pix = Pixel::blend(above_pix, &below_pix)
        }
    }

    #[inline]
    fn swap_pixels(&mut self, (r0, c0): (usize, usize), (r1, c1): (usize, usize)) {
        self.pixels.swap(
            r0 * self.width + c0,
            r1 * self.width + c1,
        );
    }
}

/// A read-only interface for mixing-and-matching image types
pub trait ImageLike {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn try_pix_at(&self, r: usize, c: usize) -> Option<&Pixel>;
}

/// An extension of `ImageLike` that provides `pix_at`,
/// which foregoes a bounds-check (solely for efficiency).
/// Minimal implementation: `pix_at` OR `pix_at_flat`
/// (implementing `pix_at_flat` is more efficient)
pub trait ImageLikeUnchecked: ImageLike + {
    #[inline]
    fn pix_at(&self, r: usize, c: usize) -> &Pixel {
        self.pix_at_flat(r * self.width() + c)
    }

    #[inline]
    fn pix_at_flat(&self, i:usize) -> &Pixel {
        self.pix_at(i / self.width(), i % self.width())
    }
}

/// An unresizable image interface that allows modification of pixels
pub trait ImageLikeMut: ImageLike + {
    fn try_pix_at_mut(&mut self, r: usize, c: usize) -> Option<&mut Pixel>;
}

/// An extension of `ImageLikeMut` that provides `pix_at_mut`,
/// which foregoes a bounds-check (solely for efficiency).
/// Minimal implementation: `pix_at_mut` OR `pix_at_mut_flat`
/// (implementing `pix_at_flat` is more efficient)
trait ImageLikeUncheckedMut: ImageLikeUnchecked + ImageLikeMut + {
    #[inline]
    fn pix_at_mut(&mut self, r: usize, c: usize) -> &mut Pixel {
        self.pix_at_mut_flat(r * self.width() + c)
    }

    #[inline]
    fn pix_at_mut_flat(&mut self, i:usize) -> &mut Pixel {
        self.pix_at_mut(i / self.width(), i % self.width())
    }
}

// Trait implementations for `Image`

impl ImageLike for Image {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    #[inline]
    fn try_pix_at(&self, r: usize, c: usize) -> Option<&Pixel> {
        if r as usize >= self.height || c as usize >= self.width {
            None
        } else {
            Some(self.pix_at(r, c))
        }
    }
}

impl ImageLikeUnchecked for Image {
    #[inline]
    fn pix_at_flat(&self, i: usize) -> &Pixel {
        &self.pixels[i]
    }
}

impl ImageLikeMut for Image {
    #[inline]
    fn try_pix_at_mut(&mut self, r: usize, c: usize) -> Option<&mut Pixel> {
        if r as usize >= self.height || c as usize >= self.width {
            None
        } else {
            Some(self.pix_at_mut(r, c))
        }
    }
}

impl ImageLikeUncheckedMut for Image {
    #[inline]
    fn pix_at_mut_flat(&mut self, i: usize) -> &mut Pixel {
        &mut self.pixels[i]
    }
}

// DrawablePixel / DrawableImage
// same as Pixel/Image, but with pre-multiplied-alpha;
// this is necessary for drawing in cairo

/// The same data as `Pixel`, but fields in a different order,
/// plus a pre-multipled alpha (to allow for direct drawing
/// in cairo)
#[derive(Clone)]
#[allow(dead_code)]
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

    fn blend_onto(self, below: &Pixel) -> DrawablePixel {
        let below = below.to_drawable();
        let alpha_mult = 1.0 - (self.a as f64 / 255.0);
        DrawablePixel {
            r: (self.r as f64 + (below.r as f64) * alpha_mult) as u8,
            g: (self.g as f64 + (below.g as f64) * alpha_mult) as u8,
            b: (self.b as f64 + (below.b as f64) * alpha_mult) as u8,
            a: self.a.max(below.a), // ???
        }
    }
}

#[derive(Clone)]
pub struct DrawableImage {
    pixels: Vec<DrawablePixel>,
    width: usize,
    height: usize,
}

/// An `Image` that can be efficiently drawn to cairo
impl DrawableImage {
    pub fn from_image(image: &Image) -> Self {
        DrawableImage {
            width: image.width,
            height: image.height,
            pixels: image.pixels.iter().map(|p| p.to_drawable()).collect::<Vec<_>>(),
        }
    }

    /// Update `self`'s dimensions to match `images`'s.
    /// This invalidates `pixels`.
    pub fn resize_to_match(&mut self, image: &Image) {
        self.width = image.width;
        self.height = image.height;

        let target_length = self.width * self.height;

        // Fill to `target_length` with garbage values
        self.pixels.reserve(target_length);
        unsafe {
            self.pixels.set_len(target_length);
        }
    }

    pub fn to_surface_pattern(&mut self) -> cairo::SurfacePattern {
        unsafe {
            let (_, u8_slice, _) = self.pixels.align_to_mut::<u8>();

            let image_surface = cairo::ImageSurface::create_for_data_unsafe(
                u8_slice.as_mut_ptr(),
                cairo::Format::ARgb32,
                self.width as i32,
                self.height as i32,
                4 * self.width as i32
                ).unwrap();

            let surface_pattern = cairo::SurfacePattern::create(image_surface);
            surface_pattern.set_filter(cairo::Filter::Fast);

            surface_pattern
        }
    }

    pub fn to_repeated_surface_pattern(&mut self) -> cairo::SurfacePattern {
        let res = self.to_surface_pattern();
        res.set_extend(cairo::Extend::Repeat);
        res
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LayerProps {
    /// Name showed in the LayerWindow: purely visual,
    /// not tied to undo
    layer_name: String,
    locked: bool,
    visible: bool,
}

impl LayerProps {
    fn new(layer_name: &str) -> Self {
        LayerProps {
            layer_name: String::from(layer_name),
            locked: false,
            visible: true,
        }
    }

    fn default() -> Self {
        Self {
            layer_name: String::from("New Layer"),
            locked: false,
            visible: true,
        }
    }

    pub fn toggle_lock(&mut self) -> bool {
        self.locked = !self.locked;
        self.locked
    }

    pub fn toggle_visible(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }

    pub fn layer_name(&self) -> &str {
        self.layer_name.as_str()
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

/// `Layer` = `Image` + `LayerProps`
#[derive(Clone, Serialize, Deserialize)]
struct Layer {
    image: Image,
    props: LayerProps,
}

impl Layer {
    fn new(image: Image) ->  Self {
        Self {
            image,
            props: LayerProps::default(),
        }
    }
}

/// `FusedLayer` = `Image` + `DrawableImage` + `LayerProps`
/// This is effectively a data struct: no magic, just a container;
/// any updates to `drawable` must be done by the user.
#[derive(Clone)]
pub struct FusedLayer {
    image: Image,
    drawable: DrawableImage,
    props: LayerProps,
}

impl FusedLayer {
    pub fn from_image(image: Image) -> Self {
        FusedLayer {
            drawable: DrawableImage::from_image(&image),
            image,
            props: LayerProps::default(),
        }
    }

    pub fn from_image_with_name(image: Image, layer_name: &str) -> Self {
        FusedLayer {
            drawable: DrawableImage::from_image(&image),
            image,
            props: LayerProps::new(layer_name),
        }
    }

    pub fn from_layer(layer: Layer) -> Self {
        FusedLayer {
            drawable: DrawableImage::from_image(&layer.image),
            image: layer.image,
            props: layer.props,
        }
    }

    /// Return an `ImageLayer` which has the same data as `self`,
    /// except the `DrawableImage`
    pub fn unfused(&self) -> Layer {
        Layer {
            image: self.image.clone(),
            props: self.props.clone(),
        }
    }

    fn width(&self) -> usize {
        self.image.width
    }

    fn height(&self) -> usize {
        self.image.height
    }

    /// Updates the size of `self.drawable` to match
    /// that of `self.image`
    fn update_drawable_size(&mut self) {
        self.drawable.resize_to_match(&self.image);
    }


    fn re_compute_drawable(&mut self) {
        self.drawable.pixels = self.image
            .pixels.iter()
            .map(|p| p.to_drawable())
            .collect::<Vec<_>>();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerIndex {
    /// The bottom layer
    BaseLayer,
    /// The (n + 1)'th from bottom layer (0 = first from bottom)
    Nth(usize),
}

impl PartialOrd for LayerIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.to_usize().partial_cmp(&other.to_usize())
    }
}

impl LayerIndex {
    /// 0 => BaseLayer, n => Nth(n + 1)
    pub fn from_usize(n: usize) -> Self {
        match n {
            0 => Self::BaseLayer,
            _ => Self::Nth(n - 1),
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            Self::BaseLayer => 0,
            Self::Nth(n) => n + 1,
        }
    }
}

/// `FusedLayeredImage` = `Vec<FusedLayer>` + `DrawableImage`
/// A `FusedLayer` must be kept for each layer to draw
/// the thumbnails. The extra `DrawableImage` is used to
/// draw the entire thing: its pixels are blended downward
/// upon construction, then lazily as the layers are updated.
pub struct FusedLayeredImage {
    // Yes, it's inefficient to have so many `DrawableImages`,
    // but hey, at least we're using `u8`s: that makes the whole thing
    // (8x + 1) byes per pixel (where x is the number of layers).
    // That confidently beats one-image-per-layer with `f32`s (16x)
    // and `f64`s (32x)

    drawable: DrawableImage,
    base_layer: FusedLayer,
    /// Non-base layers, increasing in height
    other_layers: Vec<FusedLayer>,

    active_layer_index: LayerIndex,

    // Only one layer is active at a time:
    // the below keep track of changes made to
    // the currently-active layer

    pix_modified_since_draw: HashMap<usize, Pixel>,
    pix_modified_since_save: HashMap<usize, (Pixel, Pixel)>,
}

impl FusedLayeredImage {
    pub fn from_image(image: Image) -> Self {
        FusedLayeredImage {
            drawable: DrawableImage::from_image(&image),
            base_layer: FusedLayer::from_image_with_name(image, "Base Layer"),
            other_layers: Vec::new(),
            active_layer_index: LayerIndex::BaseLayer,
            pix_modified_since_draw: HashMap::new(),
            pix_modified_since_save: HashMap::new(),
        }
    }

    #[inline]
    fn active_image(&self) -> &FusedLayer {
        match self.active_layer_index {
            LayerIndex::BaseLayer => &self.base_layer,
            LayerIndex::Nth(n) => &self.other_layers[n],
        }
    }

    #[inline]
    fn active_image_mut(&mut self) -> &mut FusedLayer {
        match self.active_layer_index {
            LayerIndex::BaseLayer => &mut self.base_layer,
            LayerIndex::Nth(n) => &mut self.other_layers[n],
        }
    }

    #[inline]
    fn active_drawable_mut(&mut self) -> &mut DrawableImage {
        match self.active_layer_index {
            LayerIndex::BaseLayer => &mut self.base_layer.drawable,
            LayerIndex::Nth(n) => &mut self.other_layers[n].drawable,
        }
    }

    #[inline]
    fn fused_image_at_layer(&self, layer: LayerIndex) -> &FusedLayer {
        match layer {
            LayerIndex::BaseLayer => &self.base_layer,
            LayerIndex::Nth(n) => &self.other_layers[n],
        }
    }

    #[inline]
    fn image_at_layer(&self, layer: LayerIndex) -> &Image {
        &self.fused_image_at_layer(layer).image
    }

    #[inline]
    fn fused_image_at_layer_mut(&mut self, layer: LayerIndex) -> &mut FusedLayer {
        match layer {
            LayerIndex::BaseLayer => &mut self.base_layer,
            LayerIndex::Nth(n) => &mut self.other_layers[n],
        }
    }

    #[inline]
    fn image_at_layer_mut(&mut self, layer: LayerIndex) -> &mut Image {
        &mut self.fused_image_at_layer_mut(layer).image
    }

    /// Try to borrow two layers mutibly at the same time:
    /// this should be no problem, but it's annoying with the
    /// vector, so this function wraps it. `None` is returned
    /// if the two layers are the same.
    #[inline]
    fn dual_layer_borrow_mut(&mut self, layer1: LayerIndex, layer2: LayerIndex) -> Option<(&mut FusedLayer, &mut FusedLayer)> {
        match (layer1, layer2) {
            (LayerIndex::BaseLayer, LayerIndex::BaseLayer) => None,
            (LayerIndex::BaseLayer, LayerIndex::Nth(n)) => {
                Some((&mut self.base_layer, &mut self.other_layers[n]))
            },
            (LayerIndex::Nth(n), LayerIndex::BaseLayer) => {
                Some((&mut self.other_layers[n], &mut self.base_layer))
            },
            (LayerIndex::Nth(n), LayerIndex::Nth(m)) => {
                if n == m {
                    None
                } else {
                    // split `self.other_layers` to get two mutable references to it

                    if n < m {
                        let (left, right) = self.other_layers.split_at_mut(m);
                        Some((&mut left[n], &mut right[0]))
                    } else { // n > m
                        let (left, right) = self.other_layers.split_at_mut(n);
                        Some((&mut right[0], &mut left[m]))
                    }
                }
            },
        }
    }

    #[inline]
    pub fn image(&self) -> &Image {
        &self.active_image().image
    }

    /// Blends the cross-section (across all layers) of the given pixel,
    /// returning a drawable pixel (as seen from the top)
    #[inline]
    fn get_blended_pixel_at(&self, i: usize) -> DrawablePixel {
        self.other_layers.iter().rev()
            .chain(std::iter::once(&self.base_layer))
            .fold(DrawablePixel::from_rgba(0, 0, 0, 0), |x, layer| {
                if layer.props.is_visible() {
                    x.blend_onto(&layer.image.pixels[i])
                } else {
                    x
                }
            })
    }

    /// Updates the size of every drawable to
    /// match the image sizes (which should all match)
    fn update_drawable_sizes(&mut self) {
        self.base_layer.update_drawable_size();
        for layer in self.other_layers.iter_mut() {
            layer.update_drawable_size();
        }

        self.drawable.resize_to_match(&self.base_layer.image);
    }

    pub fn drawable(&mut self) -> &mut DrawableImage {
        fn update_pix_modified_dict(dict: &mut HashMap<usize, (Pixel, Pixel)>, i: usize, before: &Pixel, after: &Pixel) {
            let entry = dict.entry(i);
            if let std::collections::hash_map::Entry::Occupied(mut oe) = entry {
                oe.insert((oe.get().0.clone(), after.clone()));
            } else {
                dict.insert(i, (before.clone(), after.clone()));
            }
        }

        for (i, p_before) in self.pix_modified_since_draw.iter() {
            self.drawable.pixels[*i] = self.get_blended_pixel_at(*i);
            match self.active_layer_index {
                LayerIndex::BaseLayer => &mut self.base_layer,
                LayerIndex::Nth(n) => &mut self.other_layers[n],
            }.drawable.pixels[*i] = self.active_image().image.pixels[*i].to_drawable();
            let new_value = self.active_image().image.pixels[*i].clone();
            update_pix_modified_dict(&mut self.pix_modified_since_save, *i, p_before, &new_value);
        }

        self.pix_modified_since_draw.clear();
        &mut self.drawable
    }

    pub fn layer_drawable(&mut self, layer_index: LayerIndex) -> &mut DrawableImage {
        &mut self.fused_image_at_layer_mut(layer_index).drawable
    }

    pub fn set_layer_name(&mut self, layer_index: LayerIndex, new_name: &str) {
        self.fused_image_at_layer_mut(layer_index).props.layer_name = String::from(new_name)
    }

    fn get_and_reset_modified(&mut self) -> (HashMap<usize, (Pixel, Pixel)>, LayerIndex) {
        self.drawable(); // flush pix_modified_since_draw

        let mut mod_pix = HashMap::new();
        std::mem::swap(&mut mod_pix, &mut self.pix_modified_since_save);

        (mod_pix, self.active_layer_index)
    }

    #[inline]
    fn re_compute_layer_drawable_pixel(&mut self, i: usize, layer: LayerIndex) {
        match layer {
            LayerIndex::BaseLayer => &mut self.base_layer,
            LayerIndex::Nth(n) => &mut self.other_layers[n],
        }.drawable.pixels[i] = self.image_at_layer(layer).pixels[i].to_drawable();
    }

    fn re_compute_main_drawable_pixel(&mut self, i: usize) {
        self.drawable.pixels[i] = self.get_blended_pixel_at(i);
    }

    fn re_compute_main_drawable(&mut self) {
        self.drawable.pixels = (0..self.drawable.pixels.len())
            .map(|i| self.get_blended_pixel_at(i))
            .collect::<Vec<_>>();
    }

    fn re_compute_drawable_at_index(&mut self, layer_index: LayerIndex) {
        self.fused_image_at_layer_mut(layer_index).re_compute_drawable();
    }

    pub fn layer_indices(&self) -> impl Iterator<Item = LayerIndex> {
        std::iter::once(LayerIndex::BaseLayer)
            .chain(
                (0..self.other_layers.len())
                    .map(|i| LayerIndex::Nth(i))
            )
    }

    pub fn num_layers(&self) -> usize {
        self.other_layers.len() + 1
    }

    pub fn active_layer_index(&self) -> &LayerIndex {
        &self.active_layer_index
    }

    pub fn next_unused_layer_idx(&self) -> LayerIndex {
        LayerIndex::Nth(self.other_layers.len())
    }

    fn append_new_layer(&mut self, fill_color: gtk::gdk::RGBA, idx: LayerIndex) {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let pixels = vec![Pixel::from_rgba_struct(fill_color); width * height];

        self.append_layer_with_image(Layer::new(Image::new(pixels, width, height)), idx);
    }

    fn append_layer_with_image(&mut self, layer: Layer, idx: LayerIndex) {
        let mut new_image = FusedLayer::from_layer(layer);

        match idx {
            LayerIndex::BaseLayer => {
                std::mem::swap(&mut new_image, &mut self.base_layer);
                self.other_layers.insert(0, new_image);
            },
            LayerIndex::Nth(n) => {
                if n == self.other_layers.len() {
                    self.other_layers.push(new_image);
                } else {
                    self.other_layers.insert(n, new_image);
                }
            }
        }
    }

    fn remove_layer(&mut self, idx: LayerIndex) {
        match idx {
            LayerIndex::BaseLayer => {
                assert!(self.other_layers.len() != 0);
                let new_base = self.other_layers.remove(0);
                self.base_layer = new_base;
                self.active_layer_index = LayerIndex::BaseLayer;
            },
            LayerIndex::Nth(n) => {
                self.other_layers.remove(n);
                self.active_layer_index = LayerIndex::from_usize(idx.to_usize() - 1);
            }
        }

        if self.active_layer_index.to_usize() >= self.num_layers() {
            self.active_layer_index = LayerIndex::from_usize(self.num_layers());
        }
    }

    fn swap_layers(&mut self, i1: LayerIndex, i2: LayerIndex) {
        if let Some((l1, l2)) = self.dual_layer_borrow_mut(i1, i2) {
            std::mem::swap(l1, l2);
        }
    }

    fn merge_layers(&mut self, top_idx: LayerIndex, bot_idx: LayerIndex) {
        if let Some((below, above)) = self.dual_layer_borrow_mut(bot_idx, top_idx) {
            below.image.blend_under(&above.image);

            self.remove_layer(top_idx); // this calls `self.re_compute_drawables()`
        }
    }

    pub fn layer_propss(&self) -> impl Iterator<Item = &LayerProps> + '_ {
        std::iter::once(&self.base_layer.props)
        .chain(self.other_layers.iter().map(|layer| &layer.props))
    }

    pub fn gen_entire_blended_image(&self) -> Image {
        let mut res = self.base_layer.image.clone();
        for layer in self.other_layers.iter() {
            res.blend_under(&layer.image)
        }

        res
    }

    pub fn toggle_layer_lock(&mut self, layer_index: LayerIndex) {
        self.fused_image_at_layer_mut(layer_index).props.toggle_lock();
    }

    pub fn toggle_layer_visibility(&mut self, layer_index: LayerIndex) {
        let is_visible = self.fused_image_at_layer_mut(layer_index).props.toggle_visible();

        if is_visible {
            self.re_compute_drawable_at_index(self.active_layer_index);
        }
        self.re_compute_main_drawable();
    }
}

/// An interface of `FusedLayeredImage` that only exposes
/// undoable operations (used by `DoableAction`)
pub trait TrackedLayeredImage {
    fn pix_at(&self, r: i32, c: i32) -> &Pixel;
    fn pix_at_mut(&mut self, r: i32, c: i32) -> &mut Pixel;
    fn try_pix_at(&self, r: i32, c: i32) -> Option<&Pixel>;
    fn try_pix_at_mut(&mut self, r: i32, c: i32) -> Option<&mut Pixel>;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
}

impl TrackedLayeredImage for FusedLayeredImage {
    #[inline]
    fn pix_at(&self, r: i32, c: i32) -> &Pixel {
        let i = (r * self.width() + c) as usize;
        &self.active_image().image.pixels[i]
    }

    #[inline]
    fn pix_at_mut(&mut self, r: i32, c: i32) -> &mut Pixel {
        let i = (r * self.width() + c) as usize;

        let current_value = self.active_image().image.pixels[i].clone();
        self.pix_modified_since_draw.entry(i).or_insert(current_value);

        &mut self.active_image_mut().image.pixels[i]
    }

    #[inline]
    fn try_pix_at(&self, r: i32, c: i32) -> Option<&Pixel> {
        let image = &self.active_image().image;
        if r < 0 || c < 0 || r as usize >= image.height || c as usize >= image.width {
            None
        } else {
            Some(self.pix_at(r, c))
        }
    }

    #[inline]
    fn try_pix_at_mut(&mut self, r: i32, c: i32) -> Option<&mut Pixel> {
        let image = &self.active_image().image;
        if r < 0 || c < 0 || r as usize >= image.height || c as usize >= image.width {
            None
        } else {
            Some(self.pix_at_mut(r, c))
        }
    }

    #[inline]
    fn width(&self) -> i32 {
        self.active_image().image.width as i32
    }

    #[inline]
    fn height(&self) -> i32 {
        self.active_image().image.height as i32
    }
}
