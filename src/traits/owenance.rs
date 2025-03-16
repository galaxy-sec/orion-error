use crate::{
    SeResult, StructError,
    core::{DomainFrom, DomainReason, UvsReason},
};

/// 非结构错误(StructError) 转化为结构错误。
///
use std::fmt::Display;
pub trait ErrorOwe<T, R>
where
    R: DomainReason,
{
    fn owe(self, reason: R) -> Result<T, StructError<R>>;

    fn owe_logic(self) -> SeResult<T, R>;
    fn owe_biz(self) -> SeResult<T, R>;
    fn owe_rule(self) -> SeResult<T, R>;
    fn owe_data(self) -> SeResult<T, R>;
    fn owe_conf(self) -> SeResult<T, R>;
    //fn owe_conf_info(self, msg: String) -> SeResult<T,R>;
    fn owe_res(self) -> SeResult<T, R>;
    fn owe_sys(self) -> SeResult<T, R>;
}

impl<T, E, R> ErrorOwe<T, R> for Result<T, E>
where
    E: Display,
    R: DomainReason,
{
    fn owe(self, reason: R) -> Result<T, StructError<R>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(StructError::from_domain(reason).with_detail(e.to_string())),
        }
    }

    fn owe_logic(self) -> SeResult<T, R> {
        self.map_err(|e| StructError::from_uvs_rs(UvsReason::LogicError(e.to_string())))
    }
    fn owe_biz(self) -> SeResult<T, R> {
        self.map_err(|e| StructError::from_uvs_rs(UvsReason::BizError(e.to_string())))
    }
    fn owe_rule(self) -> SeResult<T, R> {
        self.map_err(|e| StructError::from_uvs_rs(UvsReason::RuleError(e.to_string())))
    }
    fn owe_data(self) -> SeResult<T, R> {
        self.map_err(|e| StructError::from_uvs_rs(UvsReason::DataError(e.to_string(), None)))
    }
    fn owe_conf(self) -> SeResult<T, R> {
        self.map_err(|e| StructError::from_uvs_rs(UvsReason::core_conf(e.to_string())))
    }
    fn owe_res(self) -> SeResult<T, R> {
        self.map_err(|e| StructError::from_uvs_rs(UvsReason::ResError(e.to_string())))
    }
    fn owe_sys(self) -> SeResult<T, R> {
        self.map_err(|e| StructError::from_uvs_rs(UvsReason::SysError(e.to_string())))
    }
}
