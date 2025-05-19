use std::{fmt::Display, ops::Deref};

use crate::ErrorWith;

use super::{
    context::{ErrContext, WithContext},
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
    fn get_target(&self) -> Option<&String>;
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
        target: Option<String>,
        context: ErrContext,
    ) -> Self {
        StructError {
            imp: Box::new(StructErrorImpl {
                reason,
                detail,
                position,
                target,
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
        StructError::new(value, None, None, None, ErrContext::default())
    }
}

#[derive(Error, Debug, Clone, PartialEq, Getters, Serialize)]
pub struct StructErrorImpl<T: DomainReason> {
    reason: T,
    detail: Option<String>,
    position: Option<String>,
    target: Option<String>,
    context: ErrContext,
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
        other.imp.target,
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
    pub fn with_context(mut self, context: ErrContext) -> Self {
        self.imp.context = context;
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
}

impl<T: DomainReason> StructErrorTrait<T> for StructError<T> {
    fn get_reason(&self) -> &T {
        &self.reason
    }

    fn get_detail(&self) -> Option<&String> {
        self.detail.as_ref()
    }

    fn get_target(&self) -> Option<&String> {
        self.target.as_ref()
    }
}

impl<S1: Into<String>, S2: Into<String>, T: DomainReason> ContextAdd<(S1, S2)> for StructError<T> {
    fn add_context(&mut self, val: (S1, S2)) {
        self.imp.context.items.push((val.0.into(), val.1.into()));
    }
}

impl<T: DomainReason> ContextAdd<&WithContext> for StructError<T> {
    fn add_context(&mut self, ctx: &WithContext) {
        ctx.target()
            .clone()
            .map(|target| self.imp.target.replace(target));
        self.imp
            .context
            .items
            .append(&mut ctx.context().items.clone());
    }
}

impl<T: std::fmt::Display + DomainReason + ErrorCode> Display for StructError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 核心错误信息
        write!(f, "[{}] {}", self.error_code(), self.reason)?;

        // 位置信息优先显示
        if let Some(pos) = &self.position {
            write!(f, "\n  -> At: {}", pos)?;
        }

        // 目标资源信息
        if let Some(target) = &self.target {
            write!(f, "\n  -> Want: {}", target)?;
        }

        // 技术细节
        if let Some(detail) = &self.detail {
            write!(f, "\n  -> Details: {}", detail)?;
        }

        // 上下文信息
        if !self.context.items.is_empty() {
            write!(f, "\n  -> Context stack:")?;
            for (i, (k, v)) in self.context.items.iter().enumerate() {
                write!(f, "\n     {}. {}:{}", i + 1, k, v)?;
            }
        }

        Ok(())
    }
}

impl<T: DomainReason> ErrorWith for StructError<T> {
    fn want<S: Into<String>>(mut self, desc: S) -> Self {
        self.imp.target = Some(desc.into());
        self
    }
    fn position<S: Into<String>>(mut self, pos: S) -> Self {
        self.imp.position = Some(pos.into());
        self
    }

    fn with<C: Into<WithContext>>(mut self, ctx: C) -> Self {
        self.add_context(&ctx.into());
        self
    }
}

#[cfg(test)]
mod tests {

    use crate::UvsReason;

    use super::*;
    use derive_more::From;
    use serde_json::json;

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
        let mut context = ErrContext::default();
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
            Some("target_resource".to_string()),
            context,
        );

        // Serialize to JSON
        let json_value = serde_json::to_value(&error).unwrap();
        println!("{}", json_value);

        // Expected JSON structure
        let expected = json!({
            "reason":  "TestError" ,
            "detail": "Detailed error description",
            "position": "file.rs:10:5",
            "target": "target_resource",
            "context": {
                "items": [
                    ["key1", "value1"],
                    ["key2", "value2"]
                ]
            }
        });

        assert_eq!(json_value, expected);
    }
}
