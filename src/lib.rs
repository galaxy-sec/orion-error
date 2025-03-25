mod core;
mod testcase;
mod traits;

pub use core::StructError;
pub use core::WithContext;
pub use core::{
    convert_error_type, ConfErrReason, DomainFrom, DomainReason, ErrorCode, StructErrorTrait,
    StructReason, UvsBizFrom, UvsConfFrom, UvsDataFrom, UvsLogicFrom, UvsReason, UvsResFrom,
    UvsRuleFrom, UvsSysFrom,
};
pub use traits::ErrorOwe;
pub use traits::{ErrorConv, ErrorWith};

pub use core::ErrStrategy;
pub use testcase::{TestAssert, TestAssertWithMsg};
