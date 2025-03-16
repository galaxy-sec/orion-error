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
    pub fn make<S: Into<String>>(desc: S) -> Self {
        CallTarget::new(desc)
    }
    pub fn make_ignorable<S: Into<String>>(desc: S) -> Self {
        CallTarget::new(desc)
    }
    pub fn make_repeat<S: Into<String>>(desc: S) -> Self {
        CallTarget::new(desc)
    }
    pub fn make_escalate<S: Into<String>>(desc: S) -> Self {
        CallTarget::new(desc)
    }
}
