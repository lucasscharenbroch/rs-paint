#[derive(Clone)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Clone)]
pub struct Image {
    pub pixels: Vec<Vec<Pixel>>,
}

const blue: Pixel = Pixel {
    r: 0,
    g: 0,
    b: 255,
    a: 255,
};

pub fn mk_test_image() -> Image {
    return Image {
        pixels: vec![vec![blue; 400]; 400],
    };
}

impl Image {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|row| row
                                             .iter()
                                             .flat_map(|pix| vec![pix.a, pix.r, pix.g, pix.b])
                                             .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    }
}
