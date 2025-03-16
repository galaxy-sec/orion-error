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

pub trait StcErrConv<T, R>
where
    R: DomainReason,
{
    fn conv_logic(self) -> SeResult<T, R>;
    fn conv_biz(self) -> SeResult<T, R>;
    fn conv_rule(self) -> SeResult<T, R>;
    fn conv_data(self) -> SeResult<T, R>;
    fn conv_conf(self) -> SeResult<T, R>;
    fn conv_res(self) -> SeResult<T, R>;
    fn conv_sys(self) -> SeResult<T, R>;
}

impl<T, R0, R> StcErrConv<T, R> for Result<T, StructError<R0>>
where
    R0: DomainReason + Display,
    R: DomainReason,
    R: Display,
{
    fn conv_logic(self) -> SeResult<T, R> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let msg = format!("{}", e.reason());
                StructError::err_from_uvs(e, UvsReason::LogicError(msg))
            }
        }
    }

    fn conv_biz(self) -> SeResult<T, R> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let msg = format!("{}", e.reason());
                StructError::err_from_uvs(e, UvsReason::BizError(msg))
            }
        }
    }

    fn conv_rule(self) -> SeResult<T, R> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let msg = format!("{}", e.reason());
                StructError::err_from_uvs(e, UvsReason::RuleError(msg))
            }
        }
    }

    fn conv_data(self) -> SeResult<T, R> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let msg = format!("{}", e.reason());
                StructError::err_from_uvs(e, UvsReason::DataError(msg, None))
            }
        }
    }

    fn conv_conf(self) -> SeResult<T, R> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let msg = format!("{}", e.reason());
                StructError::err_from_uvs(e, UvsReason::core_conf(msg))
            }
        }
    }

    fn conv_res(self) -> SeResult<T, R> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let msg = format!("{}", e.reason());
                StructError::err_from_uvs(e, UvsReason::ResError(msg))
            }
        }
    }

    fn conv_sys(self) -> SeResult<T, R> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let msg = format!("{}", e.reason());
                StructError::err_from_uvs(e, UvsReason::SysError(msg))
            }
        }
    }
}
