use thiserror::Error;

use super::{DomainReason, UvsReason};

/// Represents the root cause of an error, which can be either
/// a domain-specific reason or a universal system reason.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StructReason<T: DomainReason> {
    #[error("{0}")]
    Universal(UvsReason),
    #[error("{0}")]
    Domain(T),
}
impl<T: DomainReason> From<UvsReason> for StructReason<T> {
    fn from(value: UvsReason) -> Self {
        Self::Universal(value)
    }
}

impl<T: DomainReason> From<T> for StructReason<T> {
    fn from(value: T) -> Self {
        Self::Domain(value)
    }
}

pub trait ErrorCode {
    fn error_code(&self) -> i32 {
        500
    }
}
impl<T: DomainReason + ErrorCode> ErrorCode for StructReason<T> {
    fn error_code(&self) -> i32 {
        match self {
            StructReason::Universal(uvs_reason) => uvs_reason.error_code(),
            StructReason::Domain(domain) => domain.error_code(),
        }
    }
}

pub fn convert_reason<R1, R2>(other: StructReason<R1>) -> StructReason<R2>
where
    R1: DomainReason,
    R2: DomainReason,
    StructReason<R2>: From<R1>,
{
    match other {
        StructReason::Universal(uvs_reason) => StructReason::Universal(uvs_reason),
        StructReason::Domain(domain) => StructReason::from(domain),
    }
}
