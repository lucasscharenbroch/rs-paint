use std::collections::{VecDeque, HashMap};
use gtk::cairo;

use super::{ImageLike, Pixel};
use super::undo::action::{AutoDiffAction, ActionName};

/// Wrapper for flattened Vec<bool>; instances should
/// be viewed as immutable, else the cached `outline_path`
/// will be inaccurate
pub struct ImageBitmask {
    height: usize,
    width: usize,
    bits: Vec<bool>,
    outline_path: Option<cairo::Path>,
    edge_path: Option<cairo::Path>,
}

impl ImageBitmask {
    fn new(height: usize, width: usize) -> Self {
        ImageBitmask {
            height,
            width,
            bits: vec![false; height * width],
            outline_path: None,
            edge_path: None,
        }
    }

    pub fn from_flat_bits(height: usize, width: usize, bits: Vec<bool>) -> Self {
        assert!(bits.len() == height * width);

        ImageBitmask {
            height,
            width,
            bits,
            outline_path: None,
            edge_path: None,
        }
    }

    #[inline]
    fn flat_index(&mut self, r: usize, c: usize) -> &mut bool {
        &mut self.bits[r * self.width + c]
    }

    /// Generic function to flood-fill a `Canvas`'s `Image` to obtain
    /// a bitmask; used for both magic wand and fill
    pub fn from_flood_fill(
        image: &impl ImageLike,
        tolerance: f64,
        or: usize,
        oc: usize,
        tolerance_reference: Option<&Pixel>, // pixel to use in tolerance computation
    ) -> Self {
        let w = image.width();
        let h = image.height();
        let mut res = ImageBitmask::new(h, w);

        let mut q = VecDeque::new();
        *res.flat_index(or, oc) = true;
        q.push_back((or, oc));

        while let Some((r, c)) = q.pop_front() {
            // the pixel which we're computing the tolerance test with
            let reference = tolerance_reference.unwrap_or(image.try_pix_at(r, c).unwrap());

            for (nr, nc) in in_bounds_4d_neighbors(r, c, w, h).into_iter() {
                if *res.flat_index(nr, nc) {
                    continue; // already visited, continue
                }

                if fulfills_tolerance(
                    reference,
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
    pub fn coords_of_active_bits(&self) -> Box<dyn Iterator<Item = (usize, usize)> + '_> {
        Box::new(
            self.bits.iter()
                .enumerate()
                .filter(|(_idx, is_active)| **is_active)
                .map(|(idx, _is_active)| (idx / self.width, idx % self.width))
        )
    }

    /// Creates a `cairo::Path` from all of the set-pixel-to-unset-pixel
    /// boundaries
    fn gen_edge_path(&self, cr: &cairo::Context) -> cairo::Path {
        #[inline]
        fn is_active(bitmask: &ImageBitmask, x: i32, y: i32) -> bool {
            x >= 0 && y >= 0 &&
            (x as usize) < bitmask.width && (y as usize) < bitmask.height &&
            bitmask.bits[y as usize * bitmask.width + x as usize]
        }

        // gather segments by scanning each row and column
        let mut segments = vec![];
        let mut curr = None;

        // horizontal segments
        for i in 0..(self.height + 1) {
            for j in 0..(self.width + 1) {
                if is_active(&self, j as i32, i as i32 - 1) !=
                   is_active(&self, j as i32, i as i32) {
                    if let None = curr {
                        curr = Some(j);
                    }
                } else if let Some(oj) = curr {
                    segments.push(((oj, i), (j, i)));
                    curr = None;
                }
            }
            if let Some(oj) = curr {
                segments.push(((oj, i), (self.width, i)));
                curr = None;
            }
        }

        // vertical segments
        for j in 0..(self.width + 1) {
            for i in 0..(self.height + 1) {
                if is_active(&self, j as i32 - 1, i as i32) !=
                   is_active(&self, j as i32, i as i32) {
                    if let None = curr {
                        curr = Some(i);
                    }
                } else if let Some(oi) = curr {
                    segments.push(((j, oi), (j, i)));
                    curr = None;
                }
            }
            if let Some(oi) = curr {
                segments.push(((j, oi), (j, self.height)));
                curr = None;
            }
        }

        // map each endpoint to the segments that end on it

        let mut endpoints_to_seg_idxs = HashMap::new();

        for (i, (p0, p1)) in segments.iter().enumerate() {
            endpoints_to_seg_idxs.entry(p0).or_insert(vec![]).push(i);
            endpoints_to_seg_idxs.entry(p1).or_insert(vec![]).push(i);
        }

        // trace each "strongly-connected-component"

        cr.new_path();

        let mut vis = vec![false; segments.len()];

        for i in 0..vis.len() {
            if vis[i] {
                continue;
            }

            let (p0, p1) = segments[i];
            cr.move_to(p0.0 as f64, p0.1 as f64);

            let mut curr = i;
            let mut end_point = p1;

            loop {
                vis[curr] = true;
                cr.line_to(end_point.0 as f64, end_point.1 as f64);

                let neigh_idxs = &endpoints_to_seg_idxs[&end_point];
                if let Some(other_idx) = neigh_idxs.iter()
                    // filter out curr and other visited
                    .filter(|idx| !vis[**idx])
                    .next() {
                    curr = *other_idx;
                    let (p0, p1) = segments[*other_idx];
                    end_point = if p0 == end_point {
                        p1
                    } else {
                        p0
                    }
                } else {
                    break;
                }
            }
        }

        cr.copy_path().unwrap()
    }

    /// Creates a `cairo::Path` of the outline of the top-leftmost
    /// connected group of bits in the mask (and thus serves as a
    /// total outline iff the bitmask has only one connected group
    /// with no holes)
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

            let this = (curr.0 as i32, curr.1 as i32);
            let above = (curr.0 as i32, curr.1 as i32 - 1);
            let left = (curr.0 as i32 - 1, curr.1 as i32);
            let above_left = (curr.0 as i32 - 1, curr.1 as i32 - 1);

            // The following is basically just matching on the invariants
            // of "clockwise motion" (what direction should we move, given
            // which cells are active?)
            // I don't have proof that it works, and it blows up if
            // the bits aren't four-directionally connected.
            match (
                is_active(&bitmask, this),
                is_active(&bitmask, above),
                is_active(&bitmask, left),
                is_active(&bitmask, above_left),
            ) {
                (true, false, _, _) => (curr.0 + 1, curr.1), // right
                (false, _, true, _) => (curr.0, curr.1 + 1), // down
                (_, _, false, true) => (curr.0 - 1, curr.1), // left
                (_, true, _, false) => (curr.0, curr.1 - 1), // up
                x => panic!("{x:?}"),
            }
        }

        // We wishfully assume that cairo optimizes adjacent homo-linear
        // strokes, solely drawing unit segments (edges of pixels)

        loop { // do...
            curr = next_point(&self, curr);
            cr.line_to(curr.0 as f64, curr.1 as f64);

            // ...while (!)
            if curr == top_leftmost_coords {
                break;
            }
        }

        cr.copy_path().unwrap()
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

    pub fn edge_path(&mut self, cr: &cairo::Context)-> &cairo::Path {
        if let Some(ref path) = self.edge_path {
            path
        } else {
            let path = self.gen_edge_path(cr);
            self.edge_path = Some(path);
            self.edge_path.as_ref().unwrap()
        }
    }

    /// Returns the minimal rectangle (x, y, w, h) that contains
    /// all selected pixels in the mask
    pub fn bounding_box(&self) -> (usize, usize, usize, usize) {
        let (min_x, max_x, min_y, max_y) = self.coords_of_active_bits()
            .fold((self.width, 0, self.height, 0), |(min_x, max_x, min_y, max_y), (y, x)| {
                (
                    min_x.min(x),
                    max_x.max(x),
                    min_y.min(y),
                    max_y.max(y),
                )
            });

        if min_x > max_x || min_y > max_y {
            (0, 0, 0, 0)
        } else {
            (min_x, min_y, max_x - min_x, max_y - min_y)
        }
    }

    pub fn submask(&self, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut bits = Vec::new();

        for i in 0..h {
            for j in 0..w {
                bits.push(self.bits[(y + i) * self.width + (x + j)]);
            }
        }

        ImageBitmask::from_flat_bits(h, w, bits)
    }

    pub fn bit_at(&self, i: usize) -> bool {
        self.bits[i]
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

pub struct DeletePix<I>
where
    I: Iterator<Item = (usize, usize)>
{
    pix_iter: I,
}

impl<I> DeletePix<I>
where
    I: Iterator<Item = (usize, usize)>
{
    pub fn new(pix_iter: I) -> Self {
        Self {
            pix_iter,
        }
    }
}

impl<I> AutoDiffAction for DeletePix<I>
where
    I: Iterator<Item = (usize, usize)>
{
    fn exec(self, image: &mut impl crate::image::TrackedLayeredImage) {
        for (r, c) in self.pix_iter {
            *image.pix_at_mut(r as i32, c as i32) = Pixel::from_rgba(0, 0, 0, 0);
        }
    }

    fn name(&self) -> ActionName {
        ActionName::Delete
    }
}
