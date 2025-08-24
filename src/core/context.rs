use derive_getters::Getters;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use thiserror::Error;


#[derive(Debug, Clone, Getters, Default, Serialize, Deserialize, PartialEq)]
pub struct OperationContext {
    target: Option<String>,
    context: CallContext,
    is_suc : bool,
    exit_log : bool,
}
#[allow(dead_code)]
pub type WithContext = OperationContext;
impl From<CallContext> for OperationContext {
    fn from(value: CallContext) -> Self {
        Self {
            target: None,
            context: value,
            is_suc: false,
            exit_log: false,
        }
    }
}

impl Drop for OperationContext {
    fn drop(&mut self) {
        if self.exit_log {
            if self.is_suc {
                info!("suc! {}", self.format_context());
            } else {
                error!("fail! {}", self.format_context());
            }
        }
    }
}

impl Display for OperationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(target) = &self.target {
            writeln!(f, "target: {target} ")?;
        }
        for (i, (k, v)) in self.context().items.iter().enumerate() {
            writeln!(f, "{}. {k}: {v} ", i + 1)?;
        }
        Ok(())
    }
}

impl OperationContext {
    pub fn new() -> Self {
        Self {
            target: None,
            context: CallContext::default(),
            is_suc: false,
            exit_log: false,
        }
    }
    pub fn want<S: Into<String>>(target: S) -> Self {
        Self {
            target: Some(target.into()),
            context: CallContext::default(),
            is_suc: false,
            exit_log: false,
        }
    }
    pub fn with_exit_log(mut self )-> Self {
        self.exit_log = true;
        self
    }
    pub fn with<S1: Into<String>, S2: Into<String>>(&mut self, key: S1, val: S2) {
        self.context.items.push((key.into(), val.into()));
    }
    pub fn with_path<S1: Into<String>, S2: Into<PathBuf>>(&mut self, key: S1, val: S2) {
        self.context
            .items
            .push((key.into(), format!("{}", val.into().display())));
    }

    pub fn with_want<S: Into<String>>(&mut self, target: S) {
        self.target = Some(target.into())
    }
    pub fn mark_suc(&mut self) {
        self.is_suc = true;
    }


    /// 格式化上下文信息，用于日志输出
    fn format_context(&self) -> String {
        if self.context.items.is_empty() {
            self.target.clone().unwrap_or_default()
        } else {
            format!("{}: {}", self.target.clone().unwrap_or_default(), self.context)
        }
    }

    /// 记录日志信息，在无错误情况下也可以提供有价值的上下文信息
    /// 注意：需要启用相应的日志特性才能使用这些方法
    pub fn info<S: AsRef<str>>(&self, message: S) {
        // 使用log::info宏记录信息级别日志
        info!("{}: {}", self.format_context(), message.as_ref());
    }

    pub fn debug<S: AsRef<str>>(&self, message: S) {
        // 使用log::debug宏记录调试级别日志
        debug!("{}: {}", self.format_context(), message.as_ref());
    }

    pub fn warn<S: AsRef<str>>(&self, message: S) {
        // 使用log::warn宏记录警告级别日志
        warn!("{}: {}", self.format_context(), message.as_ref());
    }

    pub fn error<S: AsRef<str>>(&self, message: S) {
        // 使用log::error宏记录错误级别日志
        error!("{}: {}", self.format_context(), message.as_ref());
    }

    pub fn trace<S: AsRef<str>>(&self, message: S) {
        // 使用log::trace宏记录跟踪级别日志
        trace!("{}: {}", self.format_context(), message.as_ref());
    }
}

impl From<String> for OperationContext {
    fn from(value: String) -> Self {
        Self {
            target: None,
            context: CallContext::from(("key", value.to_string())),
            is_suc: false,
            exit_log: false,
        }
    }
}

impl From<&PathBuf> for OperationContext {
    fn from(value: &PathBuf) -> Self {
        Self {
            target: None,
            context: CallContext::from(("path", format!("{}", value.display()))),
            is_suc: false,
            exit_log: false,
        }
    }
}

impl From<&Path> for OperationContext {
    fn from(value: &Path) -> Self {
        Self {
            target: None,
            context: CallContext::from(("path", format!("{}", value.display()))),
            is_suc: false,
            exit_log: false,
        }
    }
}

impl From<&str> for OperationContext {
    fn from(value: &str) -> Self {
        Self {
            target: None,
            context: CallContext::from(("key", value.to_string())),
            is_suc: false,
            exit_log: false,
        }
    }
}

impl From<(&str, &str)> for OperationContext {
    fn from(value: (&str, &str)) -> Self {
        Self {
            target: None,
            context: CallContext::from((value.0, value.1)),
            is_suc: false,
            exit_log: false,
        }
    }
}

impl From<(&str, String)> for OperationContext {
    fn from(value: (&str, String)) -> Self {
        Self {
            target: None,
            context: CallContext::from((value.0, value.1)),
            is_suc: false,
            exit_log: false,
        }
    }
}

impl From<(String, String)> for OperationContext {
    fn from(value: (String, String)) -> Self {
        Self {
            target: None,
            context: CallContext::from((value.0, value.1)),
            is_suc: false,
            exit_log: false,
        }
    }
}

impl From<&OperationContext> for OperationContext {
    fn from(value: &OperationContext) -> Self {
        value.clone()
    }
}

#[derive(Default, Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallContext {
    pub items: Vec<(String, String)>,
}

impl<K: AsRef<str>, V: AsRef<str>> From<(K, V)> for CallContext {
    fn from(value: (K, V)) -> Self {
        Self {
            items: vec![(value.0.as_ref().to_string(), value.1.as_ref().to_string())],
        }
    }
}

pub trait ContextAdd<T> {
    fn add_context(&mut self, val: T);
}

impl Display for CallContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.items.is_empty() {
            writeln!(f, "\ncall context:")?;
        }
        for (k, v) in &self.items {
            writeln!(f, "\t{k} : {v}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_withcontext_new() {
        let ctx = OperationContext::new();
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 0);
    }

    #[test]
    fn test_withcontext_want() {
        let ctx = OperationContext::want("test_target");
        assert_eq!(*ctx.target(), Some("test_target".to_string()));
        assert_eq!(ctx.context().items.len(), 0);
    }

    #[test]
    fn test_withcontext_with() {
        let mut ctx = OperationContext::new();
        ctx.with("key1", "value1");
        ctx.with("key2", "value2");

        assert_eq!(ctx.context().items.len(), 2);
        assert_eq!(
            ctx.context().items[0],
            ("key1".to_string(), "value1".to_string())
        );
        assert_eq!(
            ctx.context().items[1],
            ("key2".to_string(), "value2".to_string())
        );
    }

    #[test]
    fn test_withcontext_with_path() {
        let mut ctx = OperationContext::new();
        let path = PathBuf::from("/test/path");
        ctx.with_path("file_path", &path);

        assert_eq!(ctx.context().items.len(), 1);
        assert!(ctx.context().items[0].1.contains("/test/path"));
    }

    #[test]
    fn test_withcontext_with_want() {
        let mut ctx = OperationContext::new();
        ctx.with_want("new_target");

        assert_eq!(*ctx.target(), Some("new_target".to_string()));
    }

    #[test]
    fn test_errcontext_from_string() {
        let ctx = CallContext::from(("key".to_string(), "test_string".to_string()));
        assert_eq!(ctx.items.len(), 1);
        assert_eq!(ctx.items[0], ("key".to_string(), "test_string".to_string()));
    }

    #[test]
    fn test_errcontext_from_str() {
        let ctx = CallContext::from(("key", "test_str"));
        assert_eq!(ctx.items.len(), 1);
        assert_eq!(ctx.items[0], ("key".to_string(), "test_str".to_string()));
    }

    #[test]
    fn test_errcontext_from_string_pair() {
        let ctx = CallContext::from(("key1".to_string(), "value1".to_string()));
        assert_eq!(ctx.items.len(), 1);
        assert_eq!(ctx.items[0], ("key1".to_string(), "value1".to_string()));
    }

    #[test]
    fn test_errcontext_from_str_pair() {
        let ctx = CallContext::from(("key1", "value1"));
        assert_eq!(ctx.items.len(), 1);
        assert_eq!(ctx.items[0], ("key1".to_string(), "value1".to_string()));
    }

    #[test]
    fn test_errcontext_from_mixed_pair() {
        let ctx = CallContext::from(("key1", "value1".to_string()));
        assert_eq!(ctx.items.len(), 1);
        assert_eq!(ctx.items[0], ("key1".to_string(), "value1".to_string()));
    }

    #[test]
    fn test_errcontext_default() {
        let ctx = CallContext::default();
        assert_eq!(ctx.items.len(), 0);
    }

    #[test]
    fn test_errcontext_display_single() {
        let ctx = CallContext::from(("key", "test"));
        let display = format!("{}", ctx);
        assert!(display.contains("error context:"));
        assert!(display.contains("key : test"));
    }

    #[test]
    fn test_errcontext_display_multiple() {
        let mut ctx = CallContext::default();
        ctx.items.push(("key1".to_string(), "value1".to_string()));
        ctx.items.push(("key2".to_string(), "value2".to_string()));
        let display = format!("{}", ctx);
        assert!(display.contains("error context:"));
        assert!(display.contains("key1 : value1"));
        assert!(display.contains("key2 : value2"));
    }

    #[test]
    fn test_errcontext_display_empty() {
        let ctx = CallContext::default();
        let display = format!("{}", ctx);
        assert_eq!(display, "");
    }

    #[test]
    fn test_withcontext_from_string() {
        let ctx = OperationContext::from("test_string".to_string());
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert_eq!(
            ctx.context().items[0],
            ("key".to_string(), "test_string".to_string())
        );
    }

    #[test]
    fn test_withcontext_from_str() {
        let ctx = OperationContext::from("test_str".to_string());
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert_eq!(
            ctx.context().items[0],
            ("key".to_string(), "test_str".to_string())
        );
    }

    #[test]
    fn test_withcontext_from_pathbuf() {
        let path = PathBuf::from("/test/path");
        let ctx = OperationContext::from(&path);
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert!(ctx.context().items[0].1.contains("/test/path"));
    }

    #[test]
    fn test_withcontext_from_path() {
        let path = "/test/path";
        let ctx = OperationContext::from(path);
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert!(ctx.context().items[0].1.contains("/test/path"));
    }

    #[test]
    fn test_withcontext_from_string_pair() {
        let ctx = OperationContext::from(("key1".to_string(), "value1".to_string()));
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert_eq!(
            ctx.context().items[0],
            ("key1".to_string(), "value1".to_string())
        );
    }

    #[test]
    fn test_withcontext_from_str_pair() {
        let ctx = OperationContext::from(("key1", "value1"));
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert_eq!(
            ctx.context().items[0],
            ("key1".to_string(), "value1".to_string())
        );
    }

    #[test]
    fn test_withcontext_from_mixed_pair() {
        let ctx = OperationContext::from(("key1", "value1".to_string()));
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert_eq!(
            ctx.context().items[0],
            ("key1".to_string(), "value1".to_string())
        );
    }

    #[test]
    fn test_withcontext_from_path_pair() {
        let path = PathBuf::from("/test/path");
        let ctx = OperationContext::from(("file", path.to_string_lossy().as_ref()));
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert!(ctx.context().items[0].0.contains("file"));
        assert!(ctx.context().items[0].1.contains("/test/path"));
    }

    #[test]
    fn test_withcontext_display_with_target() {
        let mut ctx = OperationContext::want("test_target");
        ctx.with("key1", "value1");
        let display = format!("{}", ctx);
        assert!(display.contains("target: test_target"));
        assert!(display.contains("1. key1: value1"));
    }

    #[test]
    fn test_withcontext_display_without_target() {
        let mut ctx = OperationContext::new();
        ctx.with("key1", "value1");
        let display = format!("{}", ctx);
        assert!(!display.contains("target:"));
        assert!(display.contains("1. key1: value1"));
    }

    #[test]
    fn test_withcontext_from_errcontext() {
        let err_ctx = CallContext::from(("key1", "value1"));
        let ctx = OperationContext::from(err_ctx);
        assert!(ctx.target.is_none());
        assert_eq!(ctx.context().items.len(), 1);
        assert_eq!(
            ctx.context().items[0],
            ("key1".to_string(), "value1".to_string())
        );
    }

    #[test]
    fn test_withcontext_from_withcontext() {
        let mut ctx1 = OperationContext::want("target1");
        ctx1.with("key1", "value1");
        let ctx2 = OperationContext::from(&ctx1);
        assert_eq!(*ctx2.target(), Some("target1".to_string()));
        assert_eq!(ctx2.context().items.len(), 1);
        assert_eq!(
            ctx2.context().items[0],
            ("key1".to_string(), "value1".to_string())
        );
    }

    // ContextAdd trait tests are commented out due to trait implementation issues
    // These tests will be revisited when the ContextAdd trait is properly implemented

    #[test]
    fn test_withcontext_edge_cases() {
        let ctx1 = OperationContext::from("".to_string());
        assert_eq!(ctx1.context().items.len(), 1);
        assert_eq!(ctx1.context().items[0], ("key".to_string(), "".to_string()));

        let ctx2 = OperationContext::from(("".to_string(), "".to_string()));
        assert_eq!(ctx2.context().items.len(), 1);
        assert_eq!(ctx2.context().items[0], ("".to_string(), "".to_string()));
    }

    #[test]
    fn test_errcontext_equality() {
        let ctx1 = CallContext::from(("key1", "value1"));
        let ctx2 = CallContext::from(("key1", "value1"));
        let ctx3 = CallContext::from(("key1", "value2"));

        assert_eq!(ctx1, ctx2);
        assert_ne!(ctx1, ctx3);
    }

    #[test]
    fn test_withcontext_equality() {
        let ctx1 = OperationContext::from(("key1", "value1"));
        let ctx2 = OperationContext::from(("key1", "value1"));
        let ctx3 = OperationContext::from(("key1", "value2"));

        assert_eq!(ctx1, ctx2);
        assert_ne!(ctx1, ctx3);
    }

    #[test]
    fn test_withcontext_clone() {
        let mut ctx = OperationContext::want("target");
        ctx.with("key", "value");

        let cloned = ctx.clone();
        assert_eq!(ctx.target(), cloned.target());
        assert_eq!(ctx.context().items.len(), cloned.context().items.len());
        assert_eq!(ctx.context().items[0], cloned.context().items[0]);
    }

    #[test]
    fn test_withcontext_with_types() {
        let mut ctx = OperationContext::new();

        // 测试各种类型转换
        ctx.with("string_key", "string_value");
        ctx.with("string_key", 42.to_string()); // 数字转字符串
        ctx.with("bool_key", true.to_string()); // 布尔转字符串

        assert_eq!(ctx.context().items.len(), 3);

        // 验证最后一个添加的值
        assert_eq!(
            ctx.context().items[2],
            ("bool_key".to_string(), "true".to_string())
        );
    }
}
