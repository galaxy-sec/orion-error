use crate::WithContext;

pub trait ErrorWith {
    fn want<S: Into<String>>(self, desc: S) -> Self;
    fn position<S: Into<String>>(self, desc: S) -> Self;
    fn with<C: Into<WithContext>>(self, ctx: C) -> Self;
}

impl<T, E: ErrorWith> ErrorWith for Result<T, E> {
    fn want<S: Into<String>>(self, desc: S) -> Self {
        self.map_err(|e| e.want(desc))
    }
    fn position<S: Into<String>>(self, desc: S) -> Self {
        self.map_err(|e| e.position(desc))
    }
    fn with<C: Into<WithContext>>(self, ctx: C) -> Self {
        self.map_err(|e| e.with(ctx))
    }
}
