mod case;
mod context;
mod domain;
mod error;
mod reason;
mod stc_impl;
//mod target;
mod universal;
pub use context::ContextAdd;
pub use context::WithContext;
pub use domain::{DomainFrom, DomainReason};
pub use error::{convert_error, convert_error_type, StructError, StructErrorTrait};
pub use reason::{ErrorCode, StructReason};
pub use universal::{
    UvsBizFrom, UvsConfFrom, UvsDataFrom, UvsLogicFrom, UvsReason, UvsResFrom, UvsRuleFrom,
    UvsSysFrom,
};

pub enum ErrStrategy {
    /// 带退避策略的重试（包含基本参数）
    Retry,
    /// 静默忽略错误
    Ignore,
    /// 传播错误（默认行为）
    Throw,
}
