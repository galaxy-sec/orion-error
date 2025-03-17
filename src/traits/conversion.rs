use std::fmt::Display;

use crate::{
    SeResult,
    core::{DomainFrom, DomainReason, StructError, StructReason, UvsReason},
};

pub trait ErrorConvDomain<T, R>
where
    R: DomainReason,
{
    fn to_domain(self) -> Result<T, StructError<R>>;
}

pub trait ErrorConvUvs<T, R>
where
    R: DomainReason,
{
    fn to_uvs(self) -> SeResult<T, R>;
}

impl<T, R, E> ErrorConvDomain<T, R> for Result<T, E>
where
    R: DomainReason,
    R: From<E>,
    E: Display + Clone, //R0: DomainReason + Display + Clone,
                        //UvsReason: From<R0>,
{
    fn to_domain(self) -> Result<T, StructError<R>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let domain = R::from(e.clone());
                Err(StructError::from_domain(domain).with_detail(e.to_string()))
            }
        }
    }
}

impl<T, R0, R> ErrorConvUvs<T, R> for Result<T, StructError<R0>>
where
    R0: DomainReason + Display + Clone,
    R: DomainReason,
    UvsReason: From<R0>,
{
    fn to_uvs(self) -> SeResult<T, R> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let uvs: UvsReason = match *e.reason().clone() {
                    StructReason::Universal(uvs) => uvs.clone(),
                    StructReason::Domain(domain) => UvsReason::from(domain.clone()),
                };
                Err(StructError::from_uvs(e, uvs))
            }
        }
    }
}
