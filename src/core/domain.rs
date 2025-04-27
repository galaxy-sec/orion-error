use std::fmt::Display;

use serde::Serialize;

use super::error::StructError;

pub trait DomainReason: PartialEq + Display + Serialize {}

#[derive(Debug, PartialEq, Serialize)]
pub struct NullReason {}
impl DomainReason for NullReason {}

pub trait DomainFrom<E, R>
where
    R: DomainReason,
{
    fn from_domain(reason: E) -> StructError<R>;
    fn err_from_domain<T>(reason: E) -> Result<T, StructError<R>> {
        Err(Self::from_domain(reason))
    }
}

impl Display for NullReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NullReason")
    }
}
