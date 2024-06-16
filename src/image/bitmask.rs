use std::collections::VecDeque;
use gtk::cairo;

use super::{Image, ImageLike, Pixel};

/// Wrapper for flattened Vec<bool>; instances should
/// be viewed as immutable, else the cached `outline_path`
/// will be inaccurate
pub struct ImageBitmask {
    height: usize,
    width: usize,
    bits: Vec<bool>,
    outline_path: Option<cairo::Path>,
}

impl ImageBitmask {
    fn new(height: usize, width: usize) -> Self {
        ImageBitmask {
            height,
            width,
            bits: vec![false; height * width],
            outline_path: None,
        }
    }

    #[inline]
    fn flat_index(&mut self, r: usize, c: usize) -> &mut bool {
        &mut self.bits[r * self.width + c]
    }

    /// Generic function to flood-fill a `Canvas`'s `Image` to obtain
    /// a bitmask; used for both magic wand and fill
    pub fn from_flood_fill(image: &impl ImageLike, tolerance: f64, or: usize, oc: usize) -> Self {
        let w = image.width();
        let h = image.height();
        let mut res = ImageBitmask::new(h, w);

        let mut q = VecDeque::new();
        *res.flat_index(or, oc) = true;
        q.push_back((or, oc));

        while let Some((r, c)) = q.pop_front() {
            for (nr, nc) in in_bounds_4d_neighbors(r, c, w, h).into_iter() {
                if *res.flat_index(nr, nc) {
                    continue; // already visited, continue
                }

                if fulfills_tolerance(
                    image.try_pix_at(r, c).unwrap(),
                    image.try_pix_at(nr, nc).unwrap(),
                    tolerance,
                ) {
                    *res.flat_index(nr, nc) = true;
                    q.push_back((nr, nc));
                }
            }
        }

        res
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns (row, col) of active bits
    pub fn coords_of_active_bits(&self) -> Vec<(usize, usize)> {
        self.bits.iter()
            .enumerate()
            .filter(|(_idx, is_active)| **is_active)
            .map(|(idx, _is_active)| (idx / self.width, idx % self.width))
            .collect::<Vec<_>>()
    }

    /// Creates a `cairo::Path` of the outline of the top-leftmost
    /// connected group of bits in the mask (and thus serves as a
    /// total outline iff the bitmask has only one connected group)
    fn gen_outline_path(&self, cr: &cairo::Context) -> cairo::Path {
        // find the top-left-most set bit
        let top_leftmost_coords = (0..(self.height * self.width))
            .filter(|i| self.bits[*i])
            .map(|i| (i % self.width, i / self.width)) // (x, y)
            .next();

        cr.new_path();

        if let None = top_leftmost_coords {
            return cr.copy_path().unwrap();
        }

        // trace clockwise

        // `curr` is the coordinates of the cell whose top-right
        // corner is the current point on the path
        let top_leftmost_coords = top_leftmost_coords.unwrap();
        let mut curr = top_leftmost_coords.clone();
        cr.move_to(top_leftmost_coords.0 as f64, top_leftmost_coords.1 as f64);

        /// Given the current point, compute the next one,
        /// moving clockwise along the outline
        #[inline]
        fn next_point(bitmask: &ImageBitmask, curr: (usize, usize)) -> (usize, usize) {
            #[inline]
            fn is_active(bitmask: &ImageBitmask, (x, y): (i32, i32)) -> bool {
                x >= 0 && y >= 0 &&
                (x as usize) < bitmask.width && (y as usize) < bitmask.height &&
                bitmask.bits[y as usize * bitmask.width + x as usize]
            }

            todo!()
        }

        // We wishfully assume that cairo optimizes adjacent homo-linear
        // strokes, solely drawing unit segments (edges of pixels)

        loop { // do...
            let prev = curr;
            curr = next_point(self, curr);
            cr.line_to(curr.0 as f64, curr.1 as f64);

            // ...while (!)
            if curr == top_leftmost_coords {
                break;
            }
        }

        return cr.copy_path().unwrap();
    }

    pub fn outline_path(&mut self, cr: &cairo::Context)-> &cairo::Path {
        if let Some(ref path) = self.outline_path {
            path
        } else {
            let path = self.gen_outline_path(cr);
            self.outline_path = Some(path);
            self.outline_path.as_ref().unwrap()
        }
    }
}

/// Returns `true` iff `a` "tolerates" (is close to) `b`
#[inline]
fn fulfills_tolerance(a: &Pixel, b: &Pixel, tolerance: f64) -> bool {
    // TODO tweak this formula? It's highly unscientific, and probably inefficient.
    let alpha_diff = ((a.a as f64 - b.a as f64) / 255.0).abs();
    (
        ((a.r as f64 - b.r as f64) / 255.0).abs() +
        ((a.g as f64 - b.g as f64) / 255.0).abs() +
        ((a.b as f64 - b.b as f64) / 255.0).abs()
    ) / 3.0 * (1.0 - alpha_diff)
    + alpha_diff
    <= tolerance.powi(2)
}

/// Looks in 4 directions from (r, c), returning the coordinates
/// of any in-bounds cells
#[inline]
fn in_bounds_4d_neighbors(r: usize, c: usize, w: usize, h: usize) -> Vec<(usize, usize)> {
    let r = r as i32;
    let c = c as i32;
    vec![
        (r + 1, c),
        (r, c + 1),
        (r - 1, c),
        (r, c - 1),
    ]
    .into_iter()
    .map(|(rp, cp)| (rp as usize, cp as usize))
    .filter(|(rp, cp)| {
        *rp < h && *rp != usize::MAX &&
        *cp < w && *cp != usize::MAX
    })
    .collect::<Vec<_>>()
}
