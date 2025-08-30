use std::{fmt::Display, ops::Deref};

use crate::ErrorWith;

use super::{
    context::{CallContext, OperationContext},
    domain::DomainReason,
    ContextAdd, ErrorCode,
};
use derive_getters::Getters;
use serde::Serialize;
use thiserror::Error;

#[macro_export]
macro_rules! location {
    () => {
        format!("{}:{}:{}", file!(), line!(), column!())
    };
}

pub trait StructErrorTrait<T: DomainReason> {
    fn get_reason(&self) -> &T;
    fn get_detail(&self) -> Option<&String>;
    fn get_target(&self) -> Option<String>;
}

impl<T: DomainReason + ErrorCode> ErrorCode for StructError<T> {
    fn error_code(&self) -> i32 {
        self.reason.error_code()
    }
}

/// Structured error type containing detailed error information
/// including error source, contextual data, and debugging information.
#[derive(Error, Debug, Clone, PartialEq, Getters)]
pub struct StructError<T: DomainReason> {
    imp: Box<StructErrorImpl<T>>,
}

impl<T: DomainReason> Serialize for StructError<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.imp.serialize(serializer)
    }
}

impl<T: DomainReason> Deref for StructError<T> {
    type Target = StructErrorImpl<T>;

    fn deref(&self) -> &Self::Target {
        &self.imp
    }
}
impl<T: DomainReason> StructError<T> {
    pub fn new(
        reason: T,
        detail: Option<String>,
        position: Option<String>,
        context: Vec<OperationContext>,
    ) -> Self {
        StructError {
            imp: Box::new(StructErrorImpl {
                reason,
                detail,
                position,
                context,
            }),
        }
    }
}

impl<T> From<T> for StructError<T>
where
    T: DomainReason,
{
    fn from(value: T) -> Self {
        StructError::new(value, None, None, Vec::new())
    }
}

#[derive(Error, Debug, Clone, PartialEq, Getters, Serialize)]
pub struct StructErrorImpl<T: DomainReason> {
    reason: T,
    detail: Option<String>,
    position: Option<String>,
    context: Vec<OperationContext>,
}

pub fn convert_error<R1, R2>(other: StructError<R1>) -> StructError<R2>
where
    R1: DomainReason,
    R2: DomainReason + From<R1>,
{
    StructError::new(
        other.imp.reason.into(),
        other.imp.detail,
        other.imp.position,
        other.imp.context,
    )
}

impl<T: DomainReason> StructError<T> {
    /// 使用示例
    ///self.with_position(location!());
    pub fn with_position(mut self, position: impl Into<String>) -> Self {
        self.imp.position = Some(position.into());
        self
    }
    pub fn with_context(mut self, context: CallContext) -> Self {
        self.imp.context.push(OperationContext::from(context));
        self
    }

    // 提供修改方法
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.imp.detail = Some(detail.into());
        self
    }
    pub fn err<V>(self) -> Result<V, Self> {
        Err(self)
    }
    pub fn target(&self) -> Option<String> {
        self.context.first().and_then(|x| x.target().clone())
    }
}

impl<T: DomainReason> StructErrorTrait<T> for StructError<T> {
    fn get_reason(&self) -> &T {
        &self.reason
    }

    fn get_detail(&self) -> Option<&String> {
        self.detail.as_ref()
    }

    fn get_target(&self) -> Option<String> {
        self.target()
    }
}

/*
impl<S1: Into<String>, S2: Into<String>, T: DomainReason> ContextAdd<(S1, S2)> for StructError<T> {
    fn add_context(&mut self, val: (S1, S2)) {
        self.imp.context.items.push((val.0.into(), val.1.into()));
    }
}
*/

impl<T: DomainReason> ContextAdd<&OperationContext> for StructError<T> {
    fn add_context(&mut self, ctx: &OperationContext) {
        self.imp.context.push(ctx.clone());
    }
}
impl<T: DomainReason> ContextAdd<OperationContext> for StructError<T> {
    fn add_context(&mut self, ctx: OperationContext) {
        self.imp.context.push(ctx);
    }
}

impl<T: std::fmt::Display + DomainReason + ErrorCode> Display for StructError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 核心错误信息
        write!(f, "[{}] {reason}", self.error_code(), reason = self.reason)?;

        // 位置信息优先显示
        if let Some(pos) = &self.position {
            write!(f, "\n  -> At: {pos}")?;
        }

        // 目标资源信息
        if let Some(target) = &self.target() {
            write!(f, "\n  -> Want: {target}")?;
        }

        // 技术细节
        if let Some(detail) = &self.detail {
            write!(f, "\n  -> Details: {detail}")?;
        }

        // 上下文信息
        if !self.context.is_empty() {
            writeln!(f, "\n  -> Context stack:")?;

            for (i, c) in self.context.iter().enumerate() {
                writeln!(f, "context {i}: ")?;
                writeln!(f, "{c}")?;
            }
        }

        Ok(())
    }
}

impl<T: DomainReason> ErrorWith for StructError<T> {
    fn want<S: Into<String>>(mut self, desc: S) -> Self {
        if self.context().is_empty() {
            self.imp.context.push(OperationContext::want(desc));
        } else if let Some(x) = self.imp.context.last_mut() {
            x.with_want(desc.into())
        }
        self
    }
    fn position<S: Into<String>>(mut self, pos: S) -> Self {
        self.imp.position = Some(pos.into());
        self
    }

    fn with<C: Into<OperationContext>>(mut self, ctx: C) -> Self {
        self.add_context(&ctx.into());
        self
    }
}

#[cfg(test)]
mod tests {

    use crate::UvsReason;

    use super::*;
    use derive_more::From;

    // Define a simple DomainReason for testing
    #[derive(Debug, Clone, PartialEq, Serialize, Error, From)]
    enum TestDomainReason {
        #[error("test error")]
        TestError,
        #[error("{0}")]
        Uvs(UvsReason),
    }

    impl ErrorCode for TestDomainReason {
        fn error_code(&self) -> i32 {
            match self {
                TestDomainReason::TestError => 1001,
                TestDomainReason::Uvs(uvs_reason) => uvs_reason.error_code(),
            }
        }
    }

    #[test]
    fn test_struct_error_serialization() {
        // Create a context
        let mut context = CallContext::default();
        context
            .items
            .push(("key1".to_string(), "value1".to_string()));
        context
            .items
            .push(("key2".to_string(), "value2".to_string()));

        // Create a StructError
        let error = StructError::new(
            TestDomainReason::TestError,
            Some("Detailed error description".to_string()),
            Some("file.rs:10:5".to_string()),
            vec![OperationContext::from(context)],
        );

        // Serialize to JSON
        let json_value = serde_json::to_value(&error).unwrap();
        println!("{json_value:#}");
    }
}
