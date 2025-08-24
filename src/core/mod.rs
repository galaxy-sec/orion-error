mod case;
mod context;
mod domain;
mod error;
mod reason;
mod universal;
use std::fmt::Display;

pub use context::ContextAdd;
pub use context::{ContextTake, OperationContext, WithContext};
pub use domain::DomainReason;
pub use error::{convert_error, StructError, StructErrorTrait};
pub use reason::ErrorCode;
pub use universal::{
    ConfErrReason, UvsBizFrom, UvsConfFrom, UvsDataFrom, UvsExternalFrom, UvsLogicFrom, UvsNetFrom,
    UvsNotFoundFrom, UvsPermissionFrom, UvsReason, UvsResFrom, UvsSysFrom, UvsTimeoutFrom,
    UvsValidationFrom,
};

pub enum ErrStrategy {
    /// 带退避策略的重试（包含基本参数）
    Retry,
    /// 静默忽略错误
    Ignore,
    /// 传播错误（默认行为）
    Throw,
}

pub fn print_error<R: DomainReason + ErrorCode + Display>(err: &StructError<R>) {
    println!("[错误代码 {}] \n{err}", err.reason().error_code());
    for ctx in err.context() {
        println!("上下文: {ctx}", ctx = ctx.context());
    }
    println!("{}", "-".repeat(50));
}
