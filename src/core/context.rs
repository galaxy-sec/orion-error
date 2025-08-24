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
    is_suc: bool,
    exit_log: bool,
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
pub trait ContextTake<S1, S2> {
    fn take(&mut self, key: S1, val: S2);
}

impl<S1 > ContextTake<S1, String> for OperationContext
where
    S1: Into<String>,
{
    fn take(&mut self, key: S1, val: String) {
        self.context.items.push((key.into(), val.into()));
    }
}

impl<S1 > ContextTake<S1, &str> for OperationContext
where
    S1: Into<String>,
{
    fn take(&mut self, key: S1, val: &str) {
        self.context.items.push((key.into(), val.into()));
    }
}

// Wrapper type for path values to avoid conflicts

impl<S1> ContextTake<S1, &PathBuf> for OperationContext
where
    S1: Into<String>,
{
    fn take(&mut self, key: S1, val: &PathBuf) {
        self.context
            .items
            .push((key.into(), format!("{}", val.display())));
    }
}
impl<S1> ContextTake<S1, &Path> for OperationContext
where
    S1: Into<String>,
{
    fn take(&mut self, key: S1, val: &Path) {
        self.context
            .items
            .push((key.into(), format!("{}", val.display())));
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
    pub fn with_exit_log(mut self) -> Self {
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
            format!(
                "{}: {}",
                self.target.clone().unwrap_or_default(),
                self.context
            )
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
// Marker trait to exclude types that are already covered by other implementations
trait NotAsRefStr: AsRef<Path> {}

// Implement for concrete path types but not for &str
impl NotAsRefStr for PathBuf {}
impl NotAsRefStr for Path {}
impl<T: AsRef<Path> + ?Sized> NotAsRefStr for &T where T: NotAsRefStr {}

impl<V: AsRef<Path>> From<(&str, V)> for OperationContext
where
    V: NotAsRefStr,
{
    fn from(value: (&str, V)) -> Self {
        Self {
            target: None,
            context: CallContext {
                items: vec![(
                    value.0.to_string(),
                    format!("{}", value.1.as_ref().display()),
                )],
            },
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

impl<K: Into<String>> ContextAdd<(K, String)> for OperationContext {
    fn add_context(&mut self, val: (K, String)) {
        self.with(val.0.into(), val.1);
    }
}
impl<K: Into<String>> ContextAdd<(K, &String)> for OperationContext {
    fn add_context(&mut self, val: (K, &String)) {
        self.with(val.0.into(), val.1.clone());
    }
}
impl<K: Into<String>> ContextAdd<(K, &str)> for OperationContext {
    fn add_context(&mut self, val: (K, &str)) {
        self.with(val.0.into(), val.1.to_string());
    }
}

impl<K: Into<String>> ContextAdd<(K, &PathBuf)> for OperationContext {
    fn add_context(&mut self, val: (K, &PathBuf)) {
        self.with(val.0.into(), format!("{}", val.1.display()));
    }
}
impl<K: Into<String>> ContextAdd<(K, &Path)> for OperationContext {
    fn add_context(&mut self, val: (K, &Path)) {
        self.with(val.0.into(), format!("{}", val.1.display()));
    }
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
        assert!(display.contains("call context:"));
        assert!(display.contains("key : test"));
    }

    #[test]
    fn test_errcontext_display_multiple() {
        let mut ctx = CallContext::default();
        ctx.items.push(("key1".to_string(), "value1".to_string()));
        ctx.items.push(("key2".to_string(), "value2".to_string()));
        let display = format!("{}", ctx);
        assert!(display.contains("call context:"));
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

    #[test]
    fn test_withcontext_from_str_path_pair() {
        let path = PathBuf::from("/test/path");
        let ctx = OperationContext::from(("file", &path));
        assert_eq!(ctx.context().items.len(), 1);
        assert_eq!(ctx.context().items[0].0, "file");
        assert!(ctx.context().items[0].1.contains("/test/path"));
    }

    #[test]
    fn test_withcontext_from_str_pathbuf_pair() {
        let path = PathBuf::from("/test/pathbuf");
        let ctx = OperationContext::from(("file", path));
        assert_eq!(ctx.context().items.len(), 1);
        assert_eq!(ctx.context().items[0].0, "file");
        assert!(ctx.context().items[0].1.contains("/test/pathbuf"));
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

    #[test]
    fn test_mark_suc() {
        let mut ctx = OperationContext::new();
        assert!(!ctx.is_suc);

        ctx.mark_suc();
        assert!(ctx.is_suc);
    }

    #[test]
    fn test_with_exit_log() {
        let ctx = OperationContext::new().with_exit_log();
        assert!(ctx.exit_log);

        let ctx2 = OperationContext::want("test").with_exit_log();
        assert!(ctx2.exit_log);
        assert_eq!(*ctx2.target(), Some("test".to_string()));
    }

    #[test]
    fn test_format_context_with_target() {
        let mut ctx = OperationContext::want("test_target");
        ctx.with("key1", "value1");

        let formatted = ctx.format_context();
        assert_eq!(formatted, "test_target: \ncall context:\n\tkey1 : value1\n");
    }

    #[test]
    fn test_format_context_without_target() {
        let mut ctx = OperationContext::new();
        ctx.with("key1", "value1");

        let formatted = ctx.format_context();
        assert_eq!(formatted, ": \ncall context:\n\tkey1 : value1\n");
    }

    #[test]
    fn test_format_context_empty() {
        let ctx = OperationContext::new();
        let formatted = ctx.format_context();
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_format_context_with_target_only() {
        let ctx = OperationContext::want("test_target");
        let formatted = ctx.format_context();
        assert_eq!(formatted, "test_target");
    }

    #[test]
    fn test_logging_methods() {
        let ctx = OperationContext::want("test_target");

        // 这些方法主要测试它们不会panic，实际日志输出需要日志框架支持
        ctx.info("info message");
        ctx.debug("debug message");
        ctx.warn("warn message");
        ctx.error("error message");
        ctx.trace("trace message");
    }

    #[test]
    fn test_logging_methods_with_empty_context() {
        let ctx = OperationContext::new();

        // 测试空上下文时的日志方法
        ctx.info("info message");
        ctx.debug("debug message");
        ctx.warn("warn message");
        ctx.error("error message");
        ctx.trace("trace message");
    }

    #[test]
    fn test_context_add_trait() {
        let mut ctx = OperationContext::new();

        // 测试ContextAdd trait的实现
        ctx.add_context(("key1", "value1"));
        ctx.add_context(("key2", "value2"));

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
    fn test_drop_trait_with_success() {
        {
            let mut ctx = OperationContext::want("test_drop").with_exit_log();
            ctx.with("operation", "test");
            ctx.mark_suc(); // 标记为成功
                            // ctx 在这里离开作用域，会触发Drop trait
        }
        // 注意：Drop trait的日志输出需要日志框架配置才能看到
        // 这里主要测试Drop trait不会panic
    }

    #[test]
    fn test_drop_trait_with_failure() {
        {
            let mut ctx = OperationContext::want("test_drop_fail").with_exit_log();
            ctx.with("operation", "test_fail");
            // 不调用mark_suc，保持is_suc = false
            // ctx 在这里离开作用域，会触发Drop trait
        }
        // 注意：Drop trait的日志输出需要日志框架配置才能看到
        // 这里主要测试Drop trait不会panic
    }

    #[test]
    fn test_drop_trait_without_exit_log() {
        {
            let mut ctx = OperationContext::want("test_no_log");
            ctx.with("operation", "no_log");
            ctx.mark_suc();
            // exit_log = false，不会触发日志输出
            // ctx 在这里离开作用域，Drop trait应该什么都不做
        }
        // 测试通过即可
    }

    #[test]
    fn test_complex_context_scenario() {
        // 模拟一个复杂的操作场景
        let mut ctx = OperationContext::want("user_registration").with_exit_log();

        // 添加各种上下文信息
        ctx.with("user_id", "12345");
        ctx.with("email", "test@example.com");
        ctx.with("role", "user");

        // 记录各种级别的日志
        ctx.info("开始用户注册流程");
        ctx.debug("验证用户输入");
        ctx.warn("检测到潜在的安全风险");

        // 模拟操作成功
        ctx.mark_suc();
        ctx.info("用户注册成功");

        // 验证上下文状态
        assert!(ctx.is_suc);
        assert!(ctx.exit_log);
        assert_eq!(*ctx.target(), Some("user_registration".to_string()));
        assert_eq!(ctx.context().items.len(), 3);

        // 验证format_context输出
        let formatted = ctx.format_context();
        assert!(formatted.contains("user_registration"));
        assert!(formatted.contains("user_id"));
        assert!(formatted.contains("email"));
        assert!(formatted.contains("role"));
    }

    #[test]
    fn test_context_serialization() {
        let mut ctx = OperationContext::want("serialization_test");
        ctx.with("key1", "value1");
        ctx.with("key2", "value2");

        // 测试序列化
        let serialized = serde_json::to_string(&ctx).expect("序列化失败");
        assert!(serialized.contains("serialization_test"));
        assert!(serialized.contains("key1"));
        assert!(serialized.contains("value1"));

        // 测试反序列化
        let deserialized: OperationContext =
            serde_json::from_str(&serialized).expect("反序列化失败");
        assert_eq!(ctx, deserialized);
    }

    #[test]
    fn test_context_with_special_characters() {
        let mut ctx = OperationContext::new();

        // 测试特殊字符
        ctx.with("key_with_spaces", "value with spaces");
        ctx.with("key_with_unicode", "值包含中文");
        ctx.with("key_with_symbols", "value@#$%^&*()");

        assert_eq!(ctx.context().items.len(), 3);
        assert_eq!(
            ctx.context().items[0],
            (
                "key_with_spaces".to_string(),
                "value with spaces".to_string()
            )
        );
        assert_eq!(
            ctx.context().items[1],
            ("key_with_unicode".to_string(), "值包含中文".to_string())
        );
        assert_eq!(
            ctx.context().items[2],
            ("key_with_symbols".to_string(), "value@#$%^&*()".to_string())
        );

        // 测试显示
        let display = format!("{}", ctx);
        assert!(display.contains("key_with_spaces"));
        assert!(display.contains("值包含中文"));
        assert!(display.contains("value@#$%^&*()"));
    }

    #[test]
    fn test_context_builder_pattern() {
        // 测试构建器模式的使用
        let ctx = OperationContext::want("builder_test").with_exit_log();

        assert_eq!(*ctx.target(), Some("builder_test".to_string()));
        assert!(ctx.exit_log);
    }

    #[test]
    fn test_context_multiple_with_calls() {
        let mut ctx = OperationContext::new();

        // 多次调用with方法
        ctx.with("key1", "value1");
        ctx.with("key2", "value2");
        ctx.with("key3", "value3");
        ctx.with("key1", "new_value1"); // 覆盖key1

        // 注意：当前实现允许重复的key，这是预期的行为
        assert_eq!(ctx.context().items.len(), 4);
        assert_eq!(
            ctx.context().items[0],
            ("key1".to_string(), "value1".to_string())
        );
        assert_eq!(
            ctx.context().items[3],
            ("key1".to_string(), "new_value1".to_string())
        );
    }

    #[test]
    fn test_context_from_various_types() {
        // 测试从各种类型创建OperationContext
        let ctx1 = OperationContext::from("simple_string");
        assert_eq!(
            ctx1.context().items[0],
            ("key".to_string(), "simple_string".to_string())
        );

        let ctx2 = OperationContext::from(("custom_key", "custom_value"));
        assert_eq!(
            ctx2.context().items[0],
            ("custom_key".to_string(), "custom_value".to_string())
        );

        let path = PathBuf::from("/test/path/file.txt");
        let ctx3 = OperationContext::from(&path);
        assert!(ctx3.context().items[0].0.contains("path"));
        assert!(ctx3.context().items[0].1.contains("/test/path/file.txt"));
    }

    // ContextTake trait 测试用例
    #[test]
    fn test_context_take_with_string_types() {
        let mut ctx = OperationContext::new();
        
        // 测试字符串类型的ContextTake实现
        ctx.take("string_key", "string_value");
        ctx.take("string_key2", String::from("string_value2"));
        ctx.take(String::from("string_key3"), "string_value3");
        ctx.take(String::from("string_key4"), String::from("string_value4"));
        
        assert_eq!(ctx.context().items.len(), 4);
        assert_eq!(ctx.context().items[0], ("string_key".to_string(), "string_value".to_string()));
        assert_eq!(ctx.context().items[1], ("string_key2".to_string(), "string_value2".to_string()));
        assert_eq!(ctx.context().items[2], ("string_key3".to_string(), "string_value3".to_string()));
        assert_eq!(ctx.context().items[3], ("string_key4".to_string(), "string_value4".to_string()));
    }

    #[test]
    fn test_context_take_with_numeric_types() {
        let mut ctx = OperationContext::new();
        
        // 测试数字类型的ContextTake实现（需要转换为字符串）
        ctx.take("int_key", 42.to_string());
        ctx.take("float_key", 3.14.to_string());
        ctx.take("bool_key", true.to_string());
        
        assert_eq!(ctx.context().items.len(), 3);
        assert_eq!(ctx.context().items[0], ("int_key".to_string(), "42".to_string()));
        assert_eq!(ctx.context().items[1], ("float_key".to_string(), "3.14".to_string()));
        assert_eq!(ctx.context().items[2], ("bool_key".to_string(), "true".to_string()));
    }

    #[test]
    fn test_context_take_with_path_context() {
        let mut ctx = OperationContext::new();
        
        // 测试PathContext包装类型的ContextTake实现
        let path1 = PathBuf::from("/test/path1.txt");
        let path2 = Path::new("/test/path2.txt");
        
        ctx.take("file1", &path1);
        ctx.take("file2", path2);
        
        assert_eq!(ctx.context().items.len(), 2);
        assert_eq!(ctx.context().items[0].0, "file1");
        assert!(ctx.context().items[0].1.contains("/test/path1.txt"));
        assert_eq!(ctx.context().items[1].0, "file2");
        assert!(ctx.context().items[1].1.contains("/test/path2.txt"));
    }

    #[test]
    fn test_context_take_mixed_types() {
        let mut ctx = OperationContext::new();
        
        // 测试混合使用字符串和PathContext类型
        ctx.take("name", "test_user");
        ctx.take("age", 25.to_string());
        ctx.take("config_file", &PathBuf::from("/etc/config.toml"));
        ctx.take("status", "active");
        
        assert_eq!(ctx.context().items.len(), 4);
        assert_eq!(ctx.context().items[0], ("name".to_string(), "test_user".to_string()));
        assert_eq!(ctx.context().items[1], ("age".to_string(), "25".to_string()));
        assert_eq!(ctx.context().items[2].0, "config_file");
        assert!(ctx.context().items[2].1.contains("/etc/config.toml"));
        assert_eq!(ctx.context().items[3], ("status".to_string(), "active".to_string()));
    }



    #[test]
    fn test_context_take_edge_cases() {
        let mut ctx = OperationContext::new();
        
        // 测试边界情况
        ctx.take("", ""); // 空字符串
        ctx.take("empty_value", ""); // 空值
        ctx.take("", "empty_key"); // 空键
        ctx.take("special_chars", "@#$%^&*()"); // 特殊字符
        ctx.take("unicode", "测试中文字符"); // Unicode字符
        
        assert_eq!(ctx.context().items.len(), 5);
        assert_eq!(ctx.context().items[0], ("".to_string(), "".to_string()));
        assert_eq!(ctx.context().items[1], ("empty_value".to_string(), "".to_string()));
        assert_eq!(ctx.context().items[2], ("".to_string(), "empty_key".to_string()));
        assert_eq!(ctx.context().items[3], ("special_chars".to_string(), "@#$%^&*()".to_string()));
        assert_eq!(ctx.context().items[4], ("unicode".to_string(), "测试中文字符".to_string()));
    }



    #[test]
    fn test_context_take_multiple_calls() {
        let mut ctx = OperationContext::new();
        
        // 测试多次调用take方法
        ctx.take("key1", "value1");
        ctx.take("key2", "value2");
        ctx.take("key1", "new_value1"); // 覆盖key1
        ctx.take("key3", &PathBuf::from("/path/file.txt"));
        ctx.take("key2", &PathBuf::from("/path/file2.txt")); // 覆盖key2
        
        // 注意：当前实现允许重复的key，这是预期的行为
        assert_eq!(ctx.context().items.len(), 5);
        assert_eq!(ctx.context().items[0], ("key1".to_string(), "value1".to_string()));
        assert_eq!(ctx.context().items[1], ("key2".to_string(), "value2".to_string()));
        assert_eq!(ctx.context().items[2], ("key1".to_string(), "new_value1".to_string()));
        assert_eq!(ctx.context().items[3].0, "key3");
        assert!(ctx.context().items[3].1.contains("/path/file.txt"));
        assert_eq!(ctx.context().items[4].0, "key2");
        assert!(ctx.context().items[4].1.contains("/path/file2.txt"));
    }

    #[test]
    fn test_context_take_with_existing_context() {
        // 创建一个已有上下文的OperationContext
        let mut ctx = OperationContext::from(("existing_key", "existing_value"));
        
        // 使用ContextTake添加更多上下文
        ctx.take("new_key1", "new_value1");
        ctx.take("new_key2", &PathBuf::from("/new/path.txt"));
        
        assert_eq!(ctx.context().items.len(), 3);
        assert_eq!(ctx.context().items[0], ("existing_key".to_string(), "existing_value".to_string()));
        assert_eq!(ctx.context().items[1], ("new_key1".to_string(), "new_value1".to_string()));
        assert_eq!(ctx.context().items[2].0, "new_key2");
        assert!(ctx.context().items[2].1.contains("/new/path.txt"));
    }




}
