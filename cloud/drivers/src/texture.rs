// TODO: add transparent stuff later if we need it

pub struct Texture {
    width: usize,
    height: usize,
    data: Vec<u16>,
}

impl Texture {
    // just panic, i guess
    pub fn new(width: usize, height: usize, data: Vec<u16>) -> Self {
        assert!(height * width == data.len());
        Self {
            height,
            width,
            data,
        }
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn data(&self) -> &[u16] {
        &self.data
    }
}
