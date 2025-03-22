use std::fmt::Display;

use crate::ErrorWith;

use super::{
    ContextAdd,
    context::{ErrContext, WithContext},
    domain::DomainReason,
    universal::UvsReason,
};
use derive_getters::Getters;
use thiserror::Error;

#[macro_export]
macro_rules! location {
    () => {
        format!("{}:{}:{}", file!(), line!(), column!())
    };
}

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

/// Structured error type containing detailed error information
/// including error source, contextual data, and debugging information.
#[derive(Error, Debug, Clone, PartialEq, Getters)]
pub struct StructError<T: DomainReason> {
    reason: StructReason<T>,
    detail: Option<String>,
    position: Option<String>,
    target: Option<String>,
    context: ErrContext,
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

pub fn convert_error<R1, R2>(other: StructError<R1>) -> StructError<R2>
where
    R1: DomainReason,
    R2: DomainReason,
    StructReason<R2>: From<R1>,
{
    StructError {
        reason: convert_reason(other.reason),
        detail: other.detail,
        position: other.position,
        target: other.target,
        context: other.context,
    }
}

impl<T: DomainReason> StructError<T> {
    pub fn new(reason: StructReason<T>) -> Self {
        Self {
            reason,
            detail: None,
            position: None,
            target: None,
            context: ErrContext::default(),
        }
    }

    /// 使用示例
    ///self.with_position(location!());
    pub fn with_position(mut self, position: impl Into<String>) -> Self {
        self.position = Some(position.into());
        self
    }
    pub fn with_context(mut self, context: ErrContext) -> Self {
        self.context = context;
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
    /// 创建领域错误快捷方法
    pub fn domain(reason: impl Into<T>) -> StructError<T> {
        Self::default_other(StructReason::Domain(reason.into()))
    }

    /// 创建通用错误快捷方法
    pub fn universal(reason: UvsReason) -> Self {
        Self::default_other(StructReason::Universal(reason))
    }
    fn default_other(reason: StructReason<T>) -> Self {
        Self {
            reason,
            detail: None,
            position: None,
            target: None,
            context: ErrContext::default(),
        }
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

impl<S1: Into<String>, S2: Into<String>, T: DomainReason> ContextAdd<(S1, S2)> for StructError<T> {
    fn add_context(&mut self, val: (S1, S2)) {
        self.context.items.push((val.0.into(), val.1.into()));
    }
}

impl<T: DomainReason> ContextAdd<&WithContext> for StructError<T> {
    fn add_context(&mut self, ctx: &WithContext) {
        ctx.target()
            .clone()
            .map(|target| self.target.replace(target));
        self.context.items.append(&mut ctx.context().items.clone());
    }
}

impl<T: std::fmt::Display + DomainReason + ErrorCode> Display for StructError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 核心错误信息
        write!(f, "[{}] {}", self.error_code(), self.reason)?;

        // 位置信息优先显示
        if let Some(pos) = &self.position {
            write!(f, "\n  -> At: {}", pos)?;
        }

        // 目标资源信息
        if let Some(target) = &self.target {
            write!(f, "\n  -> Want: {}", target)?;
        }

        // 技术细节
        if let Some(detail) = &self.detail {
            write!(f, "\n  -> Details: {}", detail)?;
        }

        // 上下文信息
        if !self.context.items.is_empty() {
            write!(f, "\n  -> Context stack:")?;
            for (i, (k, v)) in self.context.items.iter().enumerate() {
                write!(f, "\n     {}. {}:{}", i + 1, k, v)?;
            }
        }

        Ok(())
    }
}

impl<T: DomainReason> ErrorWith for StructError<T> {
    fn want<S: Into<String>>(mut self, desc: S) -> Self {
        self.target = Some(desc.into());
        self
    }
    fn position<S: Into<String>>(mut self, pos: S) -> Self {
        self.position = Some(pos.into());
        self
    }

    fn with<C: Into<WithContext>>(mut self, ctx: C) -> Self {
        self.add_context(&ctx.into());
        self
    }
}

impl<R: DomainReason> From<UvsReason> for StructError<R> {
    fn from(reason: UvsReason) -> Self {
        Self::universal(reason)
    }
}

impl<T: DomainReason> From<T> for StructError<T> {
    fn from(reason: T) -> Self {
        Self::domain(reason)
    }
}

pub fn convert_error_type<R1, R2>(e: StructError<R1>) -> StructError<R2>
where
    R1: DomainReason,
    R2: DomainReason,
    StructReason<R2>: From<R1>,
{
    let reason = match e.reason {
        StructReason::Universal(u) => StructReason::<R2>::Universal(u),
        StructReason::Domain(d) => StructReason::<R2>::from(d),
    };
    StructError {
        reason,
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
            reason: StructReason::Universal(reason),
            position: e.position,
            detail: e.detail,
            target: e.target,
            context: e.context,
        }
    }
    pub fn from_uvs_rs(reason: UvsReason) -> StructError<R> {
        Self::from(reason)
    }

    pub fn err_from_uvs<T, R1>(e: StructError<R1>, reason: UvsReason) -> Result<T, StructError<R>>
    where
        R1: DomainReason,
    {
        Err(Self::from_uvs(e, reason))
    }
}
