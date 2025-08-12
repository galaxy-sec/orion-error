/*
impl<R> UvsResFrom<StructError<R>, String> for StructError<R>
where
    R: DomainReason + From<UvsReason>,
{
    fn from_res(info: String) -> Self {
        Self::from(R::from(UvsReason::ResError(ErrorPayload::new(info))))
    }
}
*/

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
        let detail = format!("{:?}", value.1);
        StructError::from(StructReason::Domain(value.0)).with_detail(detail)
    }
}

*/
