use std::fmt::Display;
use thiserror::Error;

use super::ErrorCode;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ConfRSEnum {
    #[error("core config > {0}")]
    Core(String),
    #[error("feature config error > {0}")]
    Feature(String),
    #[error("dynamic config error > {0}")]
    Dynamic(String),
}

/// 统一错误原因分类
#[derive(Debug, Error, PartialEq, Clone)]
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
    ConfError(ConfRSEnum),
    #[error("rule error << {0}")]
    RuleError(ErrorPayload),
    #[error("privacy error << {0}")]
    PrivacyError(ErrorPayload),
}

impl UvsReason {
    pub fn core_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfError(ConfRSEnum::Core(msg.into()))
    }
    pub fn feature_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfError(ConfRSEnum::Feature(msg.into()))
    }
    pub fn dynamic_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfError(ConfRSEnum::Dynamic(msg.into()))
    }
}

pub trait UvsReasonFrom<T, S> {
    fn from_sys(info: S) -> T;
    fn from_rule(info: S) -> T;
    fn from_logic(info: S) -> T;
    fn from_biz(info: S) -> T;
    //fn from_conf_err_msg<E: Display>(e: E, msg: String) -> T;
    fn from_conf(info: S) -> T;
    fn from_res(info: S) -> T;
    fn from_data(info: S, pos: Option<usize>) -> T;
}

/// 强类型错误负载包装
#[derive(Debug, PartialEq, Clone)]
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
        ErrorPayload(value)
    }
}

impl UvsReasonFrom<UvsReason, String> for UvsReason {
    fn from_sys(info: String) -> Self {
        UvsReason::SysError(ErrorPayload::new(info))
    }

    fn from_rule(info: String) -> Self {
        UvsReason::RuleError(ErrorPayload::new(info))
    }
    fn from_logic(info: String) -> Self {
        UvsReason::LogicError(ErrorPayload::new(info))
    }
    fn from_biz(info: String) -> Self {
        UvsReason::BizError(ErrorPayload::new(info))
    }

    fn from_conf(info: String) -> Self {
        UvsReason::ConfError(ConfRSEnum::Core(info.into()))
    }

    fn from_res(info: String) -> Self {
        UvsReason::ResError(ErrorPayload::new(info))
    }
    fn from_data(info: String, pos: Option<usize>) -> Self {
        UvsReason::DataError(ErrorPayload::new(info), pos)
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
        }
    }
}
