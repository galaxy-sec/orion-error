mod core;
mod testcase;
mod traits;

pub use core::StructError;
pub use core::WithContext;
pub use core::{
    DomainFrom, DomainReason, ErrorCode, StructErrorTrait, StructReason, UvsReason, UvsReasonFrom,
    stc_err_conv,
};
pub use testcase::{TCAssert0, TCAssert1};
pub use traits::ErrorOwe;
pub use traits::{ErrorConv, ErrorWith};
