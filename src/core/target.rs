use std::fmt::Display;

#[derive(Debug)]
pub struct CallTarget {
    desc: String,
}

impl Display for CallTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "target: {} ", self.desc)
    }
}

impl CallTarget {
    pub fn new<S: Into<String>>(desc: S) -> Self {
        CallTarget { desc: desc.into() }
    }
}
