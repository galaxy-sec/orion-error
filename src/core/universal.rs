use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ConfRSEnum {
    #[error("core config > {0}")]
    Core(String),
    #[error("feature config error > {0}")]
    Feature(String),
    #[error("dynamic config error > {0}")]
    Dynamic(String),
}
#[derive(Debug, Error, PartialEq, Clone)]
pub enum UvsReason {
    #[error("logic error > {0}")]
    LogicError(String),
    #[error("biz error > {0}")]
    BizError(String),
    #[error("data error > {0}")]
    DataError(String, Option<usize>),
    #[error("sys error > {0}")]
    SysError(String),
    #[error("res error > {0}")]
    ResError(String),
    #[error("conf error > {0}")]
    ConfError(ConfRSEnum),
    #[error("rule error > {0}")]
    RuleError(String),
    #[error("privacy error > {0}")]
    PrivacyError(String),
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

pub trait UvsReasonFrom<T> {
    fn from_sys_err<E: Display>(e: E) -> T;
    fn from_sys<S: Into<String>>(info: S) -> T;
    fn from_rule_err<E: Display>(e: E) -> T;
    fn from_rule<S: Into<String>>(info: S) -> T;
    fn from_logic_err<E: Display>(e: E) -> T;
    fn from_logic<S: Into<String>>(info: S) -> T;
    fn from_biz_err<E: Display>(e: E) -> T;
    fn from_biz<S: Into<String>>(info: S) -> T;
    fn from_conf_err<E: Display>(e: E) -> T;
    fn from_conf_err_msg<E: Display>(e: E, msg: String) -> T;
    fn from_conf<S: Into<String>>(info: S) -> T;
    fn from_res_err<E: Display>(e: E) -> T;
    fn from_res<S: Into<String>>(info: S) -> T;
    fn from_data<S: Into<String>>(info: S, pos: Option<usize>) -> T;
    fn from_data_err<E: Display>(e: E) -> T;
}

impl UvsReasonFrom<UvsReason> for UvsReason {
    fn from_sys_err<E>(e: E) -> Self
    where
        E: Display,
    {
        UvsReason::SysError(format!("{}", e))
    }

    fn from_sys<S: Into<String>>(info: S) -> Self {
        UvsReason::SysError(info.into())
    }

    fn from_rule_err<E>(e: E) -> Self
    where
        E: Display,
    {
        UvsReason::RuleError(format!("{}", e))
    }
    fn from_rule<S: Into<String>>(info: S) -> Self {
        UvsReason::RuleError(info.into())
    }
    fn from_logic_err<E>(e: E) -> Self
    where
        E: Display,
    {
        UvsReason::LogicError(format!("{}", e))
    }
    fn from_logic<S: Into<String>>(info: S) -> Self {
        UvsReason::LogicError(info.into())
    }
    fn from_biz_err<E>(e: E) -> Self
    where
        E: Display,
    {
        UvsReason::BizError(format!("{}", e))
    }
    fn from_biz<S: Into<String>>(info: S) -> Self {
        UvsReason::BizError(info.into())
    }

    fn from_conf_err<E: Display>(e: E) -> Self {
        UvsReason::ConfError(ConfRSEnum::Core(format!("{}", e)))
    }
    fn from_conf_err_msg<E: Display>(e: E, msg: String) -> Self {
        UvsReason::ConfError(ConfRSEnum::Core(format!("{}/n{}", e, msg)))
    }

    fn from_conf<S: Into<String>>(info: S) -> Self {
        UvsReason::ConfError(ConfRSEnum::Core(info.into()))
    }

    fn from_res_err<E>(e: E) -> Self
    where
        E: Display,
    {
        UvsReason::ResError(format!("{}", e))
    }

    fn from_res<S: Into<String>>(info: S) -> Self {
        UvsReason::ResError(info.into())
    }

    fn from_data<S: Into<String>>(info: S, pos: Option<usize>) -> Self {
        UvsReason::DataError(info.into(), pos)
    }

    fn from_data_err<E>(e: E) -> Self
    where
        E: Display,
    {
        UvsReason::DataError(format!("{}", e,), None)
    }
}

pub trait UvsMakeAble {
    fn make(reason: UvsReason, position: Option<String>) -> Self;
}

pub struct UvsErrMaker<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> UvsReasonFrom<T> for UvsErrMaker<T>
where
    T: UvsMakeAble,
{
    fn from_sys_err<E: Display>(e: E) -> T {
        T::make(UvsReason::from_sys_err(e), None)
    }

    fn from_sys<S: Into<String>>(info: S) -> T {
        T::make(UvsReason::from_sys(info), None)
    }

    fn from_rule_err<E: Display>(e: E) -> T {
        T::make(UvsReason::from_rule_err(e), None)
    }

    fn from_rule<S: Into<String>>(info: S) -> T {
        T::make(UvsReason::from_rule(info), None)
    }

    fn from_logic_err<E: Display>(e: E) -> T {
        T::make(UvsReason::from_logic_err(e), None)
    }

    fn from_logic<S: Into<String>>(info: S) -> T {
        T::make(UvsReason::from_logic(info), None)
    }

    fn from_biz_err<E: Display>(e: E) -> T {
        T::make(UvsReason::from_biz_err(e), None)
    }

    fn from_biz<S: Into<String>>(info: S) -> T {
        T::make(UvsReason::from_biz(info), None)
    }

    fn from_conf_err<E: Display>(e: E) -> T {
        T::make(UvsReason::from_conf_err(e), None)
    }
    fn from_conf_err_msg<E: Display>(e: E, msg: String) -> T {
        T::make(UvsReason::from_conf_err_msg(e, msg), None)
    }

    fn from_conf<S: Into<String>>(info: S) -> T {
        T::make(UvsReason::from_conf(info), None)
    }

    fn from_res_err<E: Display>(e: E) -> T {
        T::make(UvsReason::from_res_err(e), None)
    }

    fn from_res<S: Into<String>>(info: S) -> T {
        T::make(UvsReason::from_res(info), None)
    }

    fn from_data<S: Into<String>>(info: S, pos: Option<usize>) -> T {
        T::make(UvsReason::from_data(info, pos), None)
    }

    fn from_data_err<E: Display>(e: E) -> T {
        T::make(UvsReason::from_data_err(e), None)
    }
}

pub fn uvs_err2code(e: &UvsReason) -> i32 {
    match e {
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
