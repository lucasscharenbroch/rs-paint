use gtk::cairo;

pub fn point_tuple_dist((x0, y0): (f64, f64), (x1, y1): (f64, f64)) -> f64 {
    ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
}

/// The effective width and height of a matrix's
/// unit square
pub fn matrix_width_height(matrix: &cairo::Matrix) -> (f64, f64) {
    // actual coordinates of the unit square's corners
    let p00 = matrix.transform_point(0.0, 0.0);
    let p10 = matrix.transform_point(1.0, 0.0);
    let p01 = matrix.transform_point(0.0, 1.0);

    (
        point_tuple_dist(p00, p10),
        point_tuple_dist(p00, p01),
    )
}

pub fn vec_magnitude((x, y): (f64, f64)) -> f64 {
    (x.powi(2) + y.powi(2)).sqrt()
}

pub fn cross_product((x0, y0): (f64, f64), (x1, y1): (f64, f64)) -> f64 {
    x0 * y1 - x1 * y0
}

pub fn dot_product((x0, y0): (f64, f64), (x1, y1): (f64, f64)) -> f64 {
    x0 * x1 + y0 * y1
}
