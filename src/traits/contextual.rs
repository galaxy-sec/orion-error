use crate::core::{ContextAdd, DomainReason, StructError};

pub trait UseTarget {
    fn want<S: Into<String>>(self, desc: S) -> Self;
}

pub trait ErrorPosition {
    fn position<S: Into<String>>(self, desc: S) -> Self;
}

impl<T, E: UseTarget> UseTarget for Result<T, E> {
    fn want<S: Into<String>>(self, desc: S) -> Self {
        self.map_err(|e| e.want(desc))
    }
}

impl<T, E: ErrorPosition> ErrorPosition for Result<T, E> {
    fn position<S: Into<String>>(self, desc: S) -> Self {
        self.map_err(|e| e.position(desc))
    }
}

pub trait ErrorWith<T, R>
where
    R: DomainReason,
{
    fn with<C>(self, ctx: C) -> Result<T, StructError<R>>
    where
        StructError<R>: ContextAdd<C>;
}

impl<T, R> ErrorWith<T, R> for Result<T, StructError<R>>
where
    R: DomainReason,
{
    fn with<C>(self, ctx: C) -> Result<T, StructError<R>>
    where
        StructError<R>: ContextAdd<C>,
    {
        match self {
            Ok(v) => Ok(v),
            Err(mut e) => {
                e.add_context(ctx);
                Err(e)
            }
        }
    }
}
