use crate::{DomainReason, StructError, StructReason, core::convert_error};

pub trait ErrorConv<T, R: DomainReason>: Sized {
    fn err_conv(self) -> Result<T, StructError<R>>;
}

impl<T, R1, R2> ErrorConv<T, R2> for Result<T, StructError<R1>>
where
    R1: DomainReason,
    R2: DomainReason,
    StructReason<R2>: From<R1>,
{
    fn err_conv(self) -> Result<T, StructError<R2>> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(convert_error::<R1, R2>(e)),
        }
    }
}

/*
impl<R1, R2> From<StructError<R1>> for StructError<R2>
where
    R1: Into<StructReason<R2>>,
{
    fn from(err: StructError<R1>) -> Self {
        StructError {
            reason: err.reason.into(),
            context: err.context,
        }
    }
}
*/
/*
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

*/
