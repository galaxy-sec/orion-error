use std::fmt::Display;

use crate::{
    SeResult,
    traits::{ErrorPosition, WithTarget},
};

use super::{
    ContextAdd,
    context::{ErrContext, WithContext},
    domain::DomainReason,
    universal::UvsReason,
};
use derive_getters::Getters;
use thiserror::Error;

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
pub trait StructErrorTrait<T: DomainReason> {
    fn get_reason(&self) -> &StructReason<T>;
    fn get_detail(&self) -> Option<&String>;
    fn get_target(&self) -> Option<&String>;
}

impl<T: DomainReason + ErrorCode> ErrorCode for StructError<T> {
    fn error_code(&self) -> i32 {
        self.reason.error_code()
    }
}

#[derive(Error, Debug, Clone, PartialEq, Getters)]
pub struct StructError<T: DomainReason> {
    reason: Box<StructReason<T>>,
    detail: Option<String>,
    position: Option<String>,
    target: Option<String>,
    context: Box<ErrContext>,
}

impl<T: DomainReason> StructError<T> {
    pub fn new(reason: StructReason<T>) -> Self {
        Self {
            reason: Box::new(reason),
            detail: None,
            position: None,
            target: None,
            context: Box::new(ErrContext::default()),
        }
    }
    pub fn with_position(mut self, position: Option<String>) -> Self {
        self.position = position;
        self
    }
    pub fn with_context(mut self, context: ErrContext) -> Self {
        self.context = Box::new(context);
        self
    }

    // 提供修改方法
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
    pub fn err<V>(self) -> Result<V, Self> {
        Err(self)
    }
}

impl<T: DomainReason> StructErrorTrait<T> for StructError<T> {
    fn get_reason(&self) -> &StructReason<T> {
        &self.reason
    }

    fn get_detail(&self) -> Option<&String> {
        self.detail.as_ref()
    }

    fn get_target(&self) -> Option<&String> {
        self.target.as_ref()
    }
}

impl<T: DomainReason> ContextAdd<String> for StructError<T> {
    fn add_context(&mut self, msg: String) {
        self.context.items.push(msg);
    }
}
impl<T: DomainReason> ContextAdd<&str> for StructError<T> {
    fn add_context(&mut self, msg: &str) {
        self.context.items.push(msg.to_string());
    }
}

impl<T: DomainReason> ContextAdd<&WithContext> for StructError<T> {
    fn add_context(&mut self, ctx: &WithContext) {
        self.target = ctx.target().clone();
        self.context.items.append(&mut ctx.context().items.clone());
    }
}

impl<T: DomainReason> ErrorPosition for StructError<T> {
    fn position<S: Into<String>>(mut self, pos: S) -> Self {
        self.position = Some(pos.into());
        self
    }
}

impl<T: std::fmt::Display + DomainReason> Display for StructError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.target, &self.detail) {
            (Some(target), Some(detail)) => {
                write!(f, "{}\nWant:{}\nDetail:{}", self.reason, target, detail)?;
                write!(f, "{}", self.context)
            }
            (Some(target), None) => {
                write!(f, "{}\nWant:{}", self.reason, target)?;
                write!(f, "{}", self.context)
            }
            (None, Some(detail)) => {
                write!(f, "{}\nWant:{}", self.reason, detail)?;
                write!(f, "{}", self.context)
            }
            (None, None) => {
                write!(f, "{}", self.reason)?;
                write!(f, "{}", self.context)
            }
        }
    }
}

impl<T: DomainReason> WithTarget for StructError<T> {
    fn want<S: Into<String>>(mut self, desc: S) -> Self {
        self.target = Some(desc.into());
        self
    }
}

pub fn ste_from_uvs<R: DomainReason>(reason: UvsReason) -> StructError<R> {
    StructError {
        reason: Box::new(StructReason::Universal(reason)),
        position: None,
        detail: None,
        target: None,
        context: Box::new(ErrContext::default()),
    }
}

pub fn stc_err_conv<R1, R2>(e: StructError<R1>) -> StructError<R2>
where
    R1: DomainReason,
    R2: DomainReason,
    StructReason<R2>: From<R1>,
{
    let reason = match *e.reason {
        StructReason::Universal(u) => StructReason::<R2>::Universal(u),
        StructReason::Domain(d) => StructReason::<R2>::from(d),
    };
    StructError {
        reason: Box::new(reason),
        position: e.position,
        detail: e.detail,
        target: e.target,
        context: e.context,
    }
}

impl<R> StructError<R>
where
    R: DomainReason,
{
    pub fn from_uvs<R1>(e: StructError<R1>, reason: UvsReason) -> StructError<R>
    where
        R1: DomainReason,
    {
        StructError {
            reason: Box::new(StructReason::Universal(reason)),
            position: e.position,
            detail: e.detail,
            target: e.target,
            context: e.context,
        }
    }
    pub fn from_uvs_rs(reason: UvsReason) -> StructError<R> {
        ste_from_uvs(reason)
    }

    pub fn err_from_uvs<T, R1>(e: StructError<R1>, reason: UvsReason) -> SeResult<T, R>
    where
        R1: DomainReason,
    {
        Err(Self::from_uvs(e, reason))
    }
}
