use crate::{
    core::{DomainReason, UvsNetFrom, UvsReason},
    StructError, UvsDataFrom, UvsSysFrom, UvsTimeoutFrom,
};

/// 非结构错误(StructError) 转化为结构错误。
///
use std::fmt::Display;
pub trait ErrorOwe<T, R>
where
    R: DomainReason,
{
    fn owe(self, reason: R) -> Result<T, StructError<R>>;
    fn owe_logic(self) -> Result<T, StructError<R>>;
    fn owe_biz(self) -> Result<T, StructError<R>>;
    fn owe_validation(self) -> Result<T, StructError<R>>;
    fn owe_data(self) -> Result<T, StructError<R>>;
    fn owe_conf(self) -> Result<T, StructError<R>>;
    fn owe_res(self) -> Result<T, StructError<R>>;
    fn owe_net(self) -> Result<T, StructError<R>>;
    fn owe_timeout(self) -> Result<T, StructError<R>>;
    fn owe_sys(self) -> Result<T, StructError<R>>;
}

impl<T, E, R> ErrorOwe<T, R> for Result<T, E>
where
    E: Display,
    R: DomainReason + From<UvsReason>,
{
    fn owe(self, reason: R) -> Result<T, StructError<R>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(StructError::from(reason).with_detail(e.to_string())),
        }
    }

    fn owe_logic(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::system_error(e.to_string()))))
    }
    fn owe_biz(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::business_error(e.to_string()))))
    }
    fn owe_validation(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::validation_error(e.to_string()))))
    }
    fn owe_data(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::from_data(e.to_string(), None))))
    }
    fn owe_conf(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::core_conf(e.to_string()))))
    }
    fn owe_res(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::resource_error(e.to_string()))))
    }
    fn owe_net(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::from_net(e.to_string()))))
    }
    fn owe_timeout(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::from_timeout(e.to_string()))))
    }
    fn owe_sys(self) -> Result<T, StructError<R>> {
        self.map_err(|e| StructError::from(R::from(UvsReason::from_sys(e.to_string()))))
    }
}
