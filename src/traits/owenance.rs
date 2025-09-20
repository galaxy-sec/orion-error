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
    fn owe_rule(self) -> Result<T, StructError<R>>;
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
            Err(e) => {
                let msg = e.to_string();
                Err(StructError::from(reason).with_detail(msg))
            }
        }
    }

    fn owe_logic(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from(UvsReason::logic_error(msg)))
    }
    fn owe_biz(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from(UvsReason::business_error(msg)))
    }
    fn owe_rule(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from(UvsReason::rule_error(msg)))
    }
    fn owe_validation(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from(UvsReason::validation_error(msg)))
    }
    fn owe_data(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from_data(msg, None))
    }
    fn owe_conf(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from(UvsReason::core_conf(msg)))
    }
    fn owe_res(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from(UvsReason::resource_error(msg)))
    }
    fn owe_net(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from_net(msg))
    }
    fn owe_timeout(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from_timeout(msg))
    }
    fn owe_sys(self) -> Result<T, StructError<R>> {
        map_err_with(self, |msg| R::from_sys(msg))
    }
}

fn map_err_with<T, E, R, F>(result: Result<T, E>, f: F) -> Result<T, StructError<R>>
where
    E: Display,
    R: DomainReason,
    F: FnOnce(String) -> R,
{
    result.map_err(|e| {
        let msg = e.to_string();
        let reason = f(msg.clone());
        StructError::from(reason).with_detail(msg)
    })
}
