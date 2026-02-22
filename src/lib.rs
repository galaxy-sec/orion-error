mod core;
mod testcase;
mod traits;

pub use core::ErrStrategy;
pub use core::{
    print_error, print_error_zh, ConfErrReason, DomainReason, ErrorCode, StructErrorTrait, UvsFrom,
    UvsReason,
};
pub use core::{ContextRecord, OperationContext, OperationScope, WithContext};
pub use core::{StructError, StructErrorBuilder};
pub use testcase::{TestAssert, TestAssertWithMsg};
pub use traits::{ConvStructError, ErrorConv, ErrorWith, ToStructError};
pub use traits::{ErrorOwe, ErrorOweBase};

/// Commonly used traits and types for convenient wildcard imports.
///
/// # Example
/// ```rust,ignore
/// use orion_error::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        ContextRecord, ErrorCode, ErrorConv, ErrorOwe, ErrorOweBase, ErrorWith, ToStructError,
        UvsFrom,
    };
    pub use crate::{OperationContext, OperationScope, StructError, StructErrorBuilder, UvsReason};
}

/// Grouped core types and enums.
pub mod types {
    pub use crate::{
        ConfErrReason, ErrStrategy, OperationContext, OperationScope, StructError,
        StructErrorBuilder, UvsReason, WithContext,
    };
}

/// Grouped conversion and context extension traits.
pub mod traits_ext {
    pub use crate::{
        ContextRecord, ConvStructError, ErrorCode, ErrorConv, ErrorOwe, ErrorOweBase, ErrorWith,
        ToStructError, UvsFrom,
    };
}
