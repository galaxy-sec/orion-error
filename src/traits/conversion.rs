use crate::{core::convert_error, DomainReason, StructError, StructReason};

pub trait ErrorConv<T, R: DomainReason>: Sized {
    fn err_conv(self) -> Result<T, StructError<R>>;
}

pub trait ConvStructError<R: DomainReason>: Sized {
    fn conv(self) -> StructError<R>;
}

impl<T, R1, R2> ErrorConv<T, R2> for Result<T, StructError<R1>>
where
    R1: DomainReason,
    R2: DomainReason,
    StructReason<R2>: From<R1>,
{
    fn err_conv(self) -> Result<T, StructError<R2>> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(convert_error::<R1, R2>(e)),
        }
    }
}

impl<R1, R2> ConvStructError<R2> for StructError<R1>
where
    R1: DomainReason,
    R2: DomainReason,
    StructReason<R2>: From<R1>,
{
    fn conv(self) -> StructError<R2> {
        convert_error::<R1, R2>(self)
    }
}
