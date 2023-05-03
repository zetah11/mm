#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Name {
    start: usize,
    end: usize,
}

#[derive(Debug, Default)]
pub struct Names {
    intern: String,
}

impl Names {
    pub fn new() -> Self {
        Self {
            intern: String::new(),
        }
    }

    pub fn make(&mut self, name: impl AsRef<str>) -> Name {
        let name = name.as_ref();
        let start = if let Some(start) = self.intern.find(name) {
            start
        } else {
            let start = self.intern.len();
            self.intern.push_str(name);
            start
        };

        Name {
            start,
            end: start + name.len(),
        }
    }

    pub fn get(&self, name: &Name) -> &str {
        &self.intern[name.start..name.end]
    }
}

#[cfg(test)]
pub(crate) fn names() -> impl FnMut(&str) -> Name {
    let mut names = Names::new();
    move |name| names.make(name)
}

#[cfg(test)]
mod tests {
    use super::Names;

    #[test]
    fn interning() {
        let a = String::from("abc");
        let b = String::from("abc");
        let c = String::from("def");
        let d = String::from("abc");

        let mut names = Names::new();
        let a = names.make(a);
        let b = names.make(b);
        let c = names.make(c);
        let d = names.make(d);

        assert_eq!(a, b);
        assert_eq!(a, d);
        assert_eq!(b, d);
        assert_ne!(a, c);
        assert_ne!(b, c);
        assert_ne!(c, d);

        assert_eq!("abc", names.get(&a));
        assert_eq!("abc", names.get(&b));
        assert_eq!("def", names.get(&c));
        assert_eq!("abc", names.get(&d));
    }

    #[test]
    fn substring() {
        let mut names = Names::new();
        let a = names.make("abcdefghijk");
        let b = names.make("efg");

        assert_eq!(0, a.start);
        assert_eq!(11, a.end);

        assert_eq!(4, b.start);
        assert_eq!(7, b.end);
    }
}
