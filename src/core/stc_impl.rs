use std::fmt::Display;

use crate::{
    SeResult,
    traits::{ErrorPosition, UseTarget},
};

use super::{
    context::{ContextAdd, ErrContext, WithContext},
    domain::{DomainFrom, DomainReason},
    error::{ErrStructAble, StructError, StructReason, ste_from_uvs},
    universal::{ConfRSEnum, UvsMakeAble, UvsReason, UvsReasonFrom},
};

impl<T: DomainReason> From<UvsReason> for StructError<T> {
    fn from(reason: UvsReason) -> Self {
        ste_from_uvs(reason)
    }
}

impl<R> UvsReasonFrom<StructError<R>> for StructError<R>
where
    R: DomainReason,
{
    fn from_sys_err<E>(e: E) -> Self
    where
        E: Display,
    {
        Self::from_uvs_rs(UvsReason::SysError(format!("{}", e)))
    }

    fn from_sys<S: Into<String>>(info: S) -> Self {
        Self::from_uvs_rs(UvsReason::SysError(info.into()))
    }

    fn from_rule_err<E>(e: E) -> Self
    where
        E: Display,
    {
        Self::from_uvs_rs(UvsReason::RuleError(format!("{}", e)))
    }
    fn from_rule<S: Into<String>>(info: S) -> Self {
        Self::from_uvs_rs(UvsReason::RuleError(info.into()))
    }
    fn from_logic_err<E>(e: E) -> Self
    where
        E: Display,
    {
        Self::from_uvs_rs(UvsReason::LogicError(format!("{}", e)))
    }
    fn from_logic<S: Into<String>>(info: S) -> Self {
        Self::from_uvs_rs(UvsReason::LogicError(info.into()))
    }
    fn from_biz_err<E>(e: E) -> Self
    where
        E: Display,
    {
        Self::from_uvs_rs(UvsReason::BizError(format!("{}", e)))
    }
    fn from_biz<S: Into<String>>(info: S) -> Self {
        Self::from_uvs_rs(UvsReason::BizError(info.into()))
    }

    fn from_conf_err<E: Display>(e: E) -> Self {
        Self::from_uvs_rs(UvsReason::ConfError(ConfRSEnum::Core(format!("{}", e))))
    }
    fn from_conf_err_msg<E: Display>(e: E, msg: String) -> Self {
        Self::from_uvs_rs(UvsReason::ConfError(ConfRSEnum::Core(format!(
            "{}/n{}",
            e, msg
        ))))
    }

    fn from_conf<S: Into<String>>(info: S) -> Self {
        Self::from_uvs_rs(UvsReason::ConfError(ConfRSEnum::Core(info.into())))
    }

    fn from_res_err<E>(e: E) -> Self
    where
        E: Display,
    {
        Self::from_uvs_rs(UvsReason::ResError(format!("{}", e)))
    }

    fn from_res<S: Into<String>>(info: S) -> Self {
        Self::from_uvs_rs(UvsReason::ResError(info.into()))
    }

    fn from_data<S: Into<String>>(info: S, pos: Option<usize>) -> Self {
        Self::from_uvs_rs(UvsReason::DataError(info.into(), pos))
    }

    fn from_data_err<E>(e: E) -> Self
    where
        E: Display,
    {
        Self::from_uvs_rs(UvsReason::DataError(format!("{}", e,), None))
    }
}

impl<T: DomainReason> UvsMakeAble for StructError<T> {
    fn make(reason: UvsReason, position: Option<String>) -> Self {
        Self::new(StructReason::Universal(reason)).with_position(position)
    }
}

pub fn stc_err_from<R1, R2>(error: StructError<R1>) -> StructError<R2>
where
    R1: DomainReason + Clone,
    R2: DomainReason,
    R2: From<R1>,
{
    let reason = match error.get_reason().clone() {
        StructReason::Universal(uvs) => StructReason::Universal(uvs),
        StructReason::Domain(domain) => StructReason::Domain(R2::from(domain)),
    };

    StructError::new(reason)
}
impl<R: DomainReason> DomainFrom<R, R> for StructError<R> {
    fn from_domain(reason: R) -> StructError<R> {
        StructError::new(StructReason::Domain(reason))
    }
    fn err_from_domain<T>(reason: R) -> Result<T, StructError<R>> {
        Err(Self::from_domain(reason))
    }
}

impl<R> From<R> for StructError<R>
where
    R: DomainReason,
{
    fn from(value: R) -> Self {
        Self::from_domain(value)
    }
}

impl<R, E> DomainFrom<(R, E), R> for StructError<R>
where
    R: DomainReason,
    E: Display,
{
    fn from_domain(value: (R, E)) -> StructError<R> {
        let detail = format!("{}", value.1);
        StructError::new(StructReason::Domain(value.0)).with_detail(detail)
    }
}

#[cfg(test)]
mod tests {
    use super::{DomainReason, StructError, stc_err_from};

    #[derive(Debug, PartialEq, Clone)]
    struct ReasonA {}

    impl DomainReason for ReasonA {}
    #[derive(Debug, PartialEq)]
    struct ReasonB {}
    impl DomainReason for ReasonB {}

    impl From<ReasonA> for ReasonB {
        fn from(_value: ReasonA) -> Self {
            ReasonB {}
        }
    }

    #[test]
    fn test_conv() {
        type ErrorA = StructError<ReasonA>;
        type ErrorB = StructError<ReasonB>;
        let e_a = ErrorA::from(ReasonA {});
        let e_b: ErrorB = stc_err_from(e_a);
        println!("e_b: {:?}", e_b)
    }
}
