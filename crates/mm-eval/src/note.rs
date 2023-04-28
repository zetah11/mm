pub trait Note: Clone {
    fn parse(name: &str) -> Option<Self>;
}

impl Note for char {
    fn parse(name: &str) -> Option<Self> {
        if name.len() == 1 {
            name.chars().next()
        } else {
            None
        }
    }
}
