pub trait Note: Clone {
    fn parse(name: &str) -> Option<Self>;
    fn add_sharp(&self, by: usize) -> Self;
    fn add_octave(&self, by: isize) -> Self;
}

impl Note for char {
    fn parse(name: &str) -> Option<Self> {
        if name.len() == 1 {
            name.chars().next()
        } else {
            None
        }
    }

    fn add_sharp(&self, _: usize) -> Self {
        *self
    }

    fn add_octave(&self, _: isize) -> Self {
        *self
    }
}
