mod core;
mod testcase;
mod traits;

pub use core::StructError;
pub use core::WithContext;
pub use core::{
    DomainFrom, DomainReason, ErrorCode, StructErrorTrait, StructReason, UvsReason, UvsReasonFrom,
    stc_err_conv,
};
pub use traits::ErrorOwe;
pub use traits::{ErrorWith, WithTarget};
pub type SeResult<T, R> = Result<T, StructError<R>>;
/*
pub use context::WithContext;
pub use err_cov::ErrorConvDomain;
pub use err_cov::ErrorConvUvs;
pub use err_cov::StcErrConv;
pub use err_owe::ErrorOwe;
pub use err_with::ErrorWith;
pub use stc_err::DomainFrom;
pub use stc_err::DomainReason;
pub use stc_err::ErrStructAble;
pub use stc_err::NullReason;
pub use stc_err::stc_err_conv;
pub use stc_err::stc_err_from;
pub use target::CallTarget;
pub use universal::ConfRSEnum;
pub use universal::UvsErrMaker;
pub use universal::UvsMakeAble;
pub use universal::UvsReason;
pub use universal::UvsReasonFrom;
pub use universal::uvs_err2code;
pub use want::UseTarget;
pub type SRS<T> = StructReason<T>;
pub type SE<T> = StructError<T>;
pub type UvsResult<T> = Result<T, StructError<NullReason>>;
pub use stc_err::{StructError, StructReason};
pub enum ErrBaseProc {
    FixRetry,
    Tolerant,
    Throw,
}
*/
