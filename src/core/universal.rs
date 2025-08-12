use serde::Serialize;
use std::fmt::Display;
use thiserror::Error;

use super::ErrorCode;

#[derive(Debug, Error, PartialEq, Clone, Serialize)]
pub enum ConfErrReason {
    #[error("core config > {0}")]
    Core(String),
    #[error("feature config error > {0}")]
    Feature(String),
    #[error("dynamic config error > {0}")]
    Dynamic(String),
}

/// Universal error reason classification
/// 统一错误原因分类
///
/// # Variants
/// - `LogicError`: Indicates business logic violations
/// - `SysError`: Represents system-level failures
#[derive(Debug, Error, PartialEq, Clone, Serialize)]
pub enum UvsReason {
    #[error("logic error << {0}")]
    LogicError(ErrorPayload),
    #[error("biz error << {0}")]
    BizError(ErrorPayload),
    #[error("data error << {0}")]
    DataError(ErrorPayload, Option<usize>),
    #[error("sys error << {0}")]
    SysError(ErrorPayload),
    #[error("res error << {0}")]
    ResError(ErrorPayload),
    #[error("conf error << {0}")]
    ConfError(ConfErrReason),
    #[error("rule error << {0}")]
    RuleError(ErrorPayload),
    #[error("privacy error << {0}")]
    PrivacyError(ErrorPayload),
    #[error("res error << {0}")]
    NetError(ErrorPayload),
    #[error("timeout << {0}")]
    Timeout(ErrorPayload),
}

impl UvsReason {
    pub fn core_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfError(ConfErrReason::Core(msg.into()))
    }
    pub fn feature_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfError(ConfErrReason::Feature(msg.into()))
    }
    pub fn dynamic_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfError(ConfErrReason::Dynamic(msg.into()))
    }
}

pub trait UvsConfFrom<S> {
    fn from_conf(info: S) -> Self;
}

pub trait UvsDataFrom<S> {
    fn from_data(info: S, pos: Option<usize>) -> Self;
}

pub trait UvsSysFrom<S> {
    fn from_sys(info: S) -> Self;
}
pub trait UvsRuleFrom<S> {
    fn from_rule(info: S) -> Self;
}
pub trait UvsLogicFrom<S> {
    fn from_logic(info: S) -> Self;
}

pub trait UvsBizFrom<S> {
    fn from_biz(info: S) -> Self;
}

pub trait UvsResFrom<S> {
    fn from_res(info: S) -> Self;
}

pub trait UvsNetFrom<S> {
    fn from_net(info: S) -> Self;
}

pub trait UvsTimeoutFrom<S> {
    fn from_timeout(info: S) -> Self;
}

/// 强类型错误负载包装
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ErrorPayload(String);

impl ErrorPayload {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }
}

impl Display for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl From<String> for ErrorPayload {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl<T> UvsConfFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_conf(reason: String) -> Self {
        T::from(UvsReason::ConfError(ConfErrReason::Core(reason)))
    }
}

impl<T> UvsConfFrom<ConfErrReason> for T
where
    T: From<UvsReason>,
{
    fn from_conf(reason: ConfErrReason) -> Self {
        T::from(UvsReason::ConfError(reason))
    }
}

impl<T> UvsDataFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_data(info: String, pos: Option<usize>) -> Self {
        T::from(UvsReason::DataError(ErrorPayload::new(info), pos))
    }
}

impl<T> UvsSysFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_sys(info: String) -> Self {
        T::from(UvsReason::SysError(ErrorPayload(info)))
    }
}

impl<T> UvsBizFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_biz(info: String) -> Self {
        T::from(UvsReason::BizError(ErrorPayload(info)))
    }
}

impl<T> UvsRuleFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_rule(info: String) -> T {
        T::from(UvsReason::RuleError(ErrorPayload(info)))
    }
}
impl<T> UvsLogicFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_logic(info: String) -> Self {
        T::from(UvsReason::LogicError(ErrorPayload(info)))
    }
}

impl<T> UvsResFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_res(info: String) -> Self {
        T::from(UvsReason::ResError(ErrorPayload(info)))
    }
}

impl ErrorCode for UvsReason {
    fn error_code(&self) -> i32 {
        match self {
            UvsReason::LogicError(_) => 100,
            UvsReason::BizError(_) => 101,
            UvsReason::DataError(_, _) => 102,
            UvsReason::SysError(_) => 103,
            UvsReason::ResError(_) => 104,
            UvsReason::ConfError(_) => 105,
            UvsReason::RuleError(_) => 106,
            UvsReason::PrivacyError(_) => 107,
            UvsReason::NetError(_) => 108,
            UvsReason::Timeout(_) => 109,
        }
    }
}

impl<T> UvsNetFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_net(info: String) -> Self {
        T::from(UvsReason::BizError(ErrorPayload(info)))
    }
}

impl<T> UvsTimeoutFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_timeout(info: String) -> Self {
        T::from(UvsReason::Timeout(ErrorPayload(info)))
    }
}
