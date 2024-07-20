use std::f64::consts::PI;
use gtk::cairo;

pub fn point_tuple_dist((x0, y0): (f64, f64), (x1, y1): (f64, f64)) -> f64 {
    ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
}

pub fn xywh_to_matrix(x: usize, y: usize, w: usize, h: usize) -> cairo::Matrix {
    let mut matrix = cairo::Matrix::identity();
    matrix.translate(x as f64, y as f64);
    matrix.scale(w as f64, h as f64);

    matrix
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

/// The angle of rotation of the matrix
pub fn matrix_rotation_angle(matrix: &cairo::Matrix) -> f64 {
    let up_vec = (0.0, 1.0);
    let matrix_up_vec = normalized_vec(matrix.transform_distance(0.0, 1.0));

    let res = f64::atan2(
        cross_product(up_vec, matrix_up_vec),
        dot_product(up_vec, matrix_up_vec),
    );

    // fix flipped sign
    if res < 0.0 {
        2.0 * PI + res
    } else {
        res
    }
}

pub fn normalized_vec(vec@(x, y): (f64, f64)) -> (f64, f64) {
    let magnitude = vec_magnitude(vec);
    (x / magnitude, y / magnitude)
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
