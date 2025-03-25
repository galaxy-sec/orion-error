use std::fmt::Display;

use super::{
    domain::{DomainFrom, DomainReason},
    error::StructError,
    universal::{ConfErrReason, ErrorPayload, UvsConfFrom, UvsDataFrom, UvsReason},
    StructReason, UvsBizFrom, UvsLogicFrom, UvsResFrom, UvsRuleFrom, UvsSysFrom,
};

impl<R> UvsConfFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason,
{
    fn from_conf(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::ConfError(ConfErrReason::Core(info)))
    }
}

impl<R> UvsDataFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason,
{
    fn from_data(info: String, pos: Option<usize>) -> Self {
        Self::from_uvs_rs(UvsReason::DataError(ErrorPayload::new(info), pos))
    }
}

impl<R> UvsSysFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason,
{
    fn from_sys(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::SysError(ErrorPayload::new(info)))
    }
}

impl<R> UvsRuleFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason,
{
    fn from_rule(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::RuleError(ErrorPayload::new(info)))
    }
}
impl<R> UvsLogicFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason,
{
    fn from_logic(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::LogicError(ErrorPayload::new(info)))
    }
}
impl<R> UvsBizFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason,
{
    fn from_biz(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::BizError(ErrorPayload::new(info)))
    }
}
impl<R> UvsResFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason,
{
    fn from_res(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::ResError(ErrorPayload::new(info)))
    }
}

impl<R: DomainReason> DomainFrom<R, R> for StructError<R> {
    fn from_domain(reason: R) -> StructError<R> {
        StructError::from(StructReason::Domain(reason))
    }
    fn err_from_domain<T>(reason: R) -> Result<T, StructError<R>> {
        Err(Self::from_domain(reason))
    }
}

impl<R, E> DomainFrom<(R, E), R> for StructError<R>
where
    R: DomainReason,
    E: Display,
{
    fn from_domain(value: (R, E)) -> StructError<R> {
        let detail = format!("{}", value.1);
        StructError::from(StructReason::Domain(value.0)).with_detail(detail)
    }
}
