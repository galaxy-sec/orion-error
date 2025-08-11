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

pub trait UvsConfFrom<T, S> {
    fn from_conf(info: S) -> T;
}

pub trait UvsDataFrom<T, S> {
    fn from_data(info: S, pos: Option<usize>) -> T;
}

pub trait UvsSysFrom<T, S> {
    fn from_sys(info: S) -> T;
}
pub trait UvsRuleFrom<T, S> {
    fn from_rule(info: S) -> T;
}
pub trait UvsLogicFrom<T, S> {
    fn from_logic(info: S) -> T;
}

pub trait UvsBizFrom<T, S> {
    fn from_biz(info: S) -> T;
}

pub trait UvsResFrom<T, S> {
    fn from_res(info: S) -> T;
}

pub trait UvsNetFrom<T, S> {
    fn from_net(info: S) -> T;
}

pub trait UvsTimeoutFrom<T, S> {
    fn from_timeout(info: S) -> T;
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
        write!(f, "{}", self.0)
    }
}
impl From<String> for ErrorPayload {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
impl UvsConfFrom<UvsReason, String> for UvsReason {
    fn from_conf(info: String) -> Self {
        UvsReason::ConfError(ConfErrReason::Core(info))
    }
}

impl UvsConfFrom<UvsReason, ConfErrReason> for UvsReason {
    fn from_conf(reason: ConfErrReason) -> Self {
        UvsReason::ConfError(reason)
    }
}

impl UvsDataFrom<UvsReason, String> for UvsReason {
    fn from_data(info: String, pos: Option<usize>) -> Self {
        UvsReason::DataError(ErrorPayload::new(info), pos)
    }
}

impl UvsSysFrom<UvsReason, String> for UvsReason {
    fn from_sys(info: String) -> Self {
        UvsReason::SysError(ErrorPayload::new(info))
    }
}
impl UvsBizFrom<UvsReason, String> for UvsReason {
    fn from_biz(info: String) -> Self {
        UvsReason::SysError(ErrorPayload::new(info))
    }
}

impl UvsRuleFrom<UvsReason, String> for UvsReason {
    fn from_rule(info: String) -> Self {
        UvsReason::RuleError(ErrorPayload::new(info))
    }
}

impl UvsLogicFrom<UvsReason, String> for UvsReason {
    fn from_logic(info: String) -> Self {
        UvsReason::LogicError(ErrorPayload::new(info))
    }
}

impl UvsResFrom<UvsReason, String> for UvsReason {
    fn from_res(info: String) -> Self {
        UvsReason::ResError(ErrorPayload::new(info))
    }
}

impl UvsNetFrom<UvsReason, String> for UvsReason {
    fn from_net(info: String) -> Self {
        UvsReason::NetError(ErrorPayload::new(info))
    }
}
impl UvsTimeoutFrom<UvsReason, String> for UvsReason {
    fn from_timeout(info: String) -> Self {
        UvsReason::Timeout(ErrorPayload::new(info))
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
