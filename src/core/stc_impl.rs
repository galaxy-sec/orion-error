use super::{
    domain::DomainReason,
    error::StructError,
    universal::{ConfErrReason, ErrorPayload, UvsConfFrom, UvsDataFrom, UvsReason},
    UvsBizFrom, UvsLogicFrom, UvsResFrom, UvsRuleFrom, UvsSysFrom,
};

impl<R> UvsConfFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason + From<UvsReason>,
{
    fn from_conf(info: String) -> Self {
        Self::from(R::from(UvsReason::ConfError(ConfErrReason::Core(info))))
    }
}

impl<R> UvsDataFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason + From<UvsReason>,
{
    fn from_data(info: String, pos: Option<usize>) -> Self {
        Self::from(R::from(UvsReason::DataError(ErrorPayload::new(info), pos)))
    }
}

impl<R> UvsSysFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason + From<UvsReason>,
{
    fn from_sys(info: String) -> Self {
        Self::from(R::from(UvsReason::SysError(ErrorPayload::new(info))))
    }
}

impl<R> UvsRuleFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason + From<UvsReason>,
{
    fn from_rule(info: String) -> Self {
        Self::from(R::from(UvsReason::RuleError(ErrorPayload::new(info))))
    }
}
impl<R> UvsLogicFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason + From<UvsReason>,
{
    fn from_logic(info: String) -> Self {
        Self::from(R::from(UvsReason::LogicError(ErrorPayload::new(info))))
    }
}
impl<R> UvsBizFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason + From<UvsReason>,
{
    fn from_biz(info: String) -> Self {
        Self::from(R::from(UvsReason::BizError(ErrorPayload::new(info))))
    }
}
impl<R> UvsResFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason + From<UvsReason>,
{
    fn from_res(info: String) -> Self {
        Self::from(R::from(UvsReason::ResError(ErrorPayload::new(info))))
    }
}

/*
impl<R: DomainReason> DomainFrom<R, R> for StructError<R> {
    fn from_domain(reason: R) -> StructError<R> {
        StructError::from(StructReason::Domain(reason))
    }
    fn err_from<T>(reason: R) -> Result<T, StructError<R>> {
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

*/
