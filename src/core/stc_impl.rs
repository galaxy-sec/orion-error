use std::fmt::Display;

use super::{
    domain::{DomainFrom, DomainReason},
    error::{StructError, StructErrorTrait, StructReason, ste_from_uvs},
    universal::{ConfRSEnum, ErrorPayload, UvsReason, UvsReasonFrom},
};

impl<T: DomainReason> From<UvsReason> for StructError<T> {
    fn from(reason: UvsReason) -> Self {
        ste_from_uvs(reason)
    }
}

impl<R> UvsReasonFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason,
{
    fn from_sys(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::SysError(ErrorPayload::new(info)))
    }

    fn from_rule(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::RuleError(ErrorPayload::new(info)))
    }
    fn from_logic(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::LogicError(ErrorPayload::new(info)))
    }
    fn from_biz(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::BizError(ErrorPayload::new(info)))
    }

    fn from_conf(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::ConfError(ConfRSEnum::Core(info)))
    }

    fn from_res(info: String) -> Self {
        Self::from_uvs_rs(UvsReason::ResError(ErrorPayload::new(info)))
    }

    fn from_data(info: String, pos: Option<usize>) -> Self {
        Self::from_uvs_rs(UvsReason::DataError(ErrorPayload::new(info), pos))
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
