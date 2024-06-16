use std::collections::VecDeque;

use super::{ImageLike, Pixel};

/// Returns `true` iff `a` "tolerates" (is close to) `b`
#[inline]
fn fulfills_tolerance(a: &Pixel, b: &Pixel, tolerance: f64) -> bool {
    // TODO tweak this algorithm? It's highly unscientific, and probably inefficient.
    ((a.r as f64 - b.r as f64) / 255.0).powi(2) +
    ((a.g as f64 - b.g as f64) / 255.0).powi(2) +
    ((a.b as f64 - b.b as f64) / 255.0).powi(2) +
    ((a.a as f64 - b.a as f64) / 255.0).powi(2)
    <= tolerance * 4.0
}

#[inline]
fn flat_index<T>(vec: &mut Vec<T>, r: usize, c: usize, w: usize) -> &mut T {
    &mut vec[r * w + c]
}

/// Looks in 4 directions from (r, c), returning the coordinates
/// of any in-bounds cells
#[inline]
fn in_bounds_4d_neighbors(r: usize, c: usize, w: usize, h: usize) -> Vec<(usize, usize)> {
    vec![
        (r + 1, c),
        (r, c + 1),
        (r - 1, c),
        (r, c - 1),
    ]
    .into_iter()
    .filter(|(rp, cp)| *rp < h && *rp != usize::MAX &&
                                       *cp < w && *cp != usize::MAX)
    .collect::<Vec<_>>()
}

/// Generic function to flood-fill a `Canvas`'s `Image` to obtain
/// a bitmask; used for both magic wand and fill
pub fn bfs_for_bitmask(image: &impl ImageLike, tolerance: f64, or: usize, oc: usize) -> Vec<bool> {
    let w = image.width();
    let h = image.height();
    let mut res = vec![false; w * h];

    let mut q = VecDeque::new();
    q.push_back((or, oc));

    while let Some((r, c)) = q.pop_front() {
        for (nr, nc) in in_bounds_4d_neighbors(r, c, w, h).into_iter() {
            if *flat_index(&mut res, nr, nc, w) {
                continue; // already visited, continue
            }

            if fulfills_tolerance(
                image.try_pix_at(r, c).unwrap(),
                image.try_pix_at(nr, nc).unwrap(),
                tolerance,
            ) {
                *flat_index(&mut res, nr, nc, w) = true;
                q.push_back((nr, nc));
            }
        }
    }

    res
}
