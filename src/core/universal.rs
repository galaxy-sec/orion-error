use serde::Serialize;
use std::fmt::Display;
use thiserror::Error;

use super::ErrorCode;

/// Configuration error sub-classification
/// 配置错误子分类
#[derive(Debug, Error, PartialEq, Clone, Serialize)]
pub enum ConfErrReason {
    #[error("core config > {0}")]
    Core(String),
    #[error("feature config error > {0}")]
    Feature(String),
    #[error("dynamic config error > {0}")]
    Dynamic(String),
}

/// Universal error reason classification with clear hierarchical structure
/// 统一错误原因分类 - 采用清晰的分层结构
///
/// # Error Code Ranges
/// - 100-199: Business Layer Errors (业务层错误)
/// - 200-299: Infrastructure Layer Errors (基础设施层错误)
/// - 300-399: Configuration & External Layer Errors (配置和外部层错误)
///
/// # Classification Principles
/// - Business Layer: User-facing errors that are expected in normal operation
/// - Infrastructure Layer: System-level failures that should be rare
/// - Configuration & External: Environment and third-party service issues
#[derive(Debug, Error, PartialEq, Clone, Serialize)]
pub enum UvsReason {
    // === Business Layer Errors (100-199) ===
    /// Input validation errors (格式错误、参数校验失败等)
    #[error("validation error << {0}")]
    ValidationError(ErrorPayload),

    /// Business logic rule violations (业务规则违反、状态冲突等)
    #[error("business logic error << {0}")]
    BusinessError(ErrorPayload),

    /// Resource not found (查询的资源不存在)
    #[error("not found error << {0}")]
    NotFoundError(ErrorPayload),

    /// Permission and authorization errors (权限不足、认证失败)
    #[error("permission error << {0}")]
    PermissionError(ErrorPayload),

    // === Infrastructure Layer Errors (200-299) ===
    /// Database and data processing errors (数据库操作、数据格式错误)
    #[error("data error << {0}")]
    DataError(ErrorPayload, Option<usize>),

    /// File system and OS-level errors (文件系统、操作系统错误)
    #[error("system error << {0}")]
    SystemError(ErrorPayload),

    /// Network connectivity and protocol errors (网络连接、HTTP请求错误)
    #[error("network error << {0}")]
    NetworkError(ErrorPayload),

    /// Resource exhaustion (内存不足、磁盘空间不足等)
    #[error("resource error << {0}")]
    ResourceError(ErrorPayload),

    /// Operation timeouts (操作超时)
    #[error("timeout error << {0}")]
    TimeoutError(ErrorPayload),

    // === Configuration & External Layer Errors (300-399) ===
    /// Configuration-related errors (配置相关错误)
    #[error("configuration error << {0}")]
    ConfigError(ConfErrReason),

    /// Third-party service errors (第三方服务错误)
    #[error("external service error << {0}")]
    ExternalError(ErrorPayload),

    /// Third-party service errors (第三方服务错误)
    #[error("BUG :logic error << {0}")]
    LogicError(ErrorPayload),
}

impl UvsReason {
    // === Configuration Error Constructors ===
    pub fn core_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(ConfErrReason::Core(msg.into()))
    }

    pub fn feature_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(ConfErrReason::Feature(msg.into()))
    }

    pub fn dynamic_conf<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(ConfErrReason::Dynamic(msg.into()))
    }

    // === Business Layer Constructors ===
    pub fn validation_error<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(ErrorPayload::new(msg))
    }

    pub fn business_error<S: Into<String>>(msg: S) -> Self {
        Self::BusinessError(ErrorPayload::new(msg))
    }

    pub fn not_found_error<S: Into<String>>(msg: S) -> Self {
        Self::NotFoundError(ErrorPayload::new(msg))
    }

    pub fn permission_error<S: Into<String>>(msg: S) -> Self {
        Self::PermissionError(ErrorPayload::new(msg))
    }

    // === Infrastructure Layer Constructors ===
    pub fn data_error<S: Into<String>>(msg: S, pos: Option<usize>) -> Self {
        Self::DataError(ErrorPayload::new(msg), pos)
    }

    pub fn system_error<S: Into<String>>(msg: S) -> Self {
        Self::SystemError(ErrorPayload::new(msg))
    }

    pub fn network_error<S: Into<String>>(msg: S) -> Self {
        Self::NetworkError(ErrorPayload::new(msg))
    }

    pub fn resource_error<S: Into<String>>(msg: S) -> Self {
        Self::ResourceError(ErrorPayload::new(msg))
    }

    pub fn timeout_error<S: Into<String>>(msg: S) -> Self {
        Self::TimeoutError(ErrorPayload::new(msg))
    }

    // === External Layer Constructors ===
    pub fn external_error<S: Into<String>>(msg: S) -> Self {
        Self::ExternalError(ErrorPayload::new(msg))
    }
    pub fn logic_error<S: Into<String>>(msg: S) -> Self {
        Self::LogicError(ErrorPayload::new(msg))
    }
}

// === Trait Definitions for Type Conversion ===

pub trait UvsConfFrom<S> {
    fn from_conf(info: S) -> Self;
}

pub trait UvsDataFrom<S> {
    fn from_data(info: S, pos: Option<usize>) -> Self;
}

pub trait UvsSysFrom<S> {
    fn from_sys(info: S) -> Self;
}

pub trait UvsBizFrom<S> {
    fn from_biz(info: S) -> Self;
}
pub trait UvsLogicFrom<S> {
    fn from_logic(info: S) -> Self;
}

pub trait UvsResFrom<S> {
    fn from_res(info: S) -> Self;
}

pub trait UvsNetFrom<S> {
    fn from_net(info: S) -> Self;
}

pub trait UvsTimeoutFrom<S> {
    fn from_timeout(info: S) -> Self;
}

// Additional traits for new error types
pub trait UvsValidationFrom<S> {
    fn from_validation(info: S) -> Self;
}

pub trait UvsNotFoundFrom<S> {
    fn from_not_found(info: S) -> Self;
}

pub trait UvsPermissionFrom<S> {
    fn from_permission(info: S) -> Self;
}

pub trait UvsExternalFrom<S> {
    fn from_external(info: S) -> Self;
}

/// Strongly typed error payload wrapper
/// 强类型错误负载包装
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ErrorPayload(String);

impl ErrorPayload {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Display for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<String> for ErrorPayload {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ErrorPayload {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

// === Trait Implementations ===

impl<T> UvsConfFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_conf(reason: String) -> Self {
        T::from(UvsReason::core_conf(reason))
    }
}

impl<T> UvsConfFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_conf(reason: &str) -> Self {
        T::from(UvsReason::core_conf(reason))
    }
}

impl<T> UvsConfFrom<ConfErrReason> for T
where
    T: From<UvsReason>,
{
    fn from_conf(reason: ConfErrReason) -> Self {
        T::from(UvsReason::ConfigError(reason))
    }
}

impl<T> UvsDataFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_data(info: String, pos: Option<usize>) -> Self {
        T::from(UvsReason::data_error(info, pos))
    }
}

impl<T> UvsDataFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_data(info: &str, pos: Option<usize>) -> Self {
        T::from(UvsReason::data_error(info, pos))
    }
}

impl<T> UvsSysFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_sys(info: String) -> Self {
        T::from(UvsReason::system_error(info))
    }
}

impl<T> UvsSysFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_sys(info: &str) -> Self {
        T::from(UvsReason::system_error(info))
    }
}

impl<T> UvsBizFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_biz(info: String) -> Self {
        T::from(UvsReason::business_error(info))
    }
}

impl<T> UvsBizFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_biz(info: &str) -> Self {
        T::from(UvsReason::business_error(info))
    }
}

impl<T> UvsResFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_res(info: String) -> Self {
        T::from(UvsReason::resource_error(info))
    }
}

impl<T> UvsResFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_res(info: &str) -> Self {
        T::from(UvsReason::resource_error(info))
    }
}

impl<T> UvsNetFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_net(info: String) -> Self {
        T::from(UvsReason::network_error(info)) // Fixed: was incorrectly mapping to BizError
    }
}

impl<T> UvsNetFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_net(info: &str) -> Self {
        T::from(UvsReason::network_error(info)) // Fixed: was incorrectly mapping to BizError
    }
}

impl<T> UvsTimeoutFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_timeout(info: String) -> Self {
        T::from(UvsReason::timeout_error(info))
    }
}

impl<T> UvsTimeoutFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_timeout(info: &str) -> Self {
        T::from(UvsReason::timeout_error(info))
    }
}

// New trait implementations for additional error types
impl<T> UvsValidationFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_validation(info: String) -> Self {
        T::from(UvsReason::validation_error(info))
    }
}

impl<T> UvsValidationFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_validation(info: &str) -> Self {
        T::from(UvsReason::validation_error(info))
    }
}

impl<T> UvsNotFoundFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_not_found(info: String) -> Self {
        T::from(UvsReason::not_found_error(info))
    }
}

impl<T> UvsNotFoundFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_not_found(info: &str) -> Self {
        T::from(UvsReason::not_found_error(info))
    }
}

impl<T> UvsPermissionFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_permission(info: String) -> Self {
        T::from(UvsReason::permission_error(info))
    }
}

impl<T> UvsPermissionFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_permission(info: &str) -> Self {
        T::from(UvsReason::permission_error(info))
    }
}

impl<T> UvsExternalFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_external(info: String) -> Self {
        T::from(UvsReason::external_error(info))
    }
}

impl<T> UvsExternalFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_external(info: &str) -> Self {
        T::from(UvsReason::external_error(info))
    }
}

impl<T> UvsLogicFrom<String> for T
where
    T: From<UvsReason>,
{
    fn from_logic(info: String) -> Self {
        T::from(UvsReason::logic_error(info))
    }
}

impl<T> UvsLogicFrom<&str> for T
where
    T: From<UvsReason>,
{
    fn from_logic(info: &str) -> Self {
        T::from(UvsReason::logic_error(info))
    }
}

impl ErrorCode for UvsReason {
    fn error_code(&self) -> i32 {
        match self {
            // === Business Layer Errors (100-199) ===
            UvsReason::ValidationError(_) => 100,
            UvsReason::BusinessError(_) => 101,
            UvsReason::NotFoundError(_) => 102,
            UvsReason::PermissionError(_) => 103,
            UvsReason::LogicError(_) => 104,

            // === Infrastructure Layer Errors (200-299) ===
            UvsReason::DataError(_, _) => 200,
            UvsReason::SystemError(_) => 201,
            UvsReason::NetworkError(_) => 202,
            UvsReason::ResourceError(_) => 203,
            UvsReason::TimeoutError(_) => 204,

            // === Configuration & External Layer Errors (300-399) ===
            UvsReason::ConfigError(_) => 300,
            UvsReason::ExternalError(_) => 301,
        }
    }
}

// === Helper Functions for Common Use Cases ===

impl UvsReason {
    /// Check if this error is retryable
    /// 检查错误是否可重试
    pub fn is_retryable(&self) -> bool {
        match self {
            // Infrastructure errors are often retryable
            UvsReason::NetworkError(_) => true,
            UvsReason::TimeoutError(_) => true,
            UvsReason::ResourceError(_) => true,
            UvsReason::SystemError(_) => true,
            UvsReason::ExternalError(_) => true,

            // Business logic errors are generally not retryable
            UvsReason::ValidationError(_) => false,
            UvsReason::BusinessError(_) => false,
            UvsReason::NotFoundError(_) => false,
            UvsReason::PermissionError(_) => false,

            // Configuration errors require manual intervention
            UvsReason::ConfigError(_) => false,
            UvsReason::DataError(_, _) => false,
            UvsReason::LogicError(_) => false,
        }
    }

    /// Check if this error should be logged with high severity
    /// 检查错误是否需要高优先级记录
    pub fn is_high_severity(&self) -> bool {
        match self {
            // System and infrastructure issues are high severity
            UvsReason::SystemError(_) => true,
            UvsReason::ResourceError(_) => true,
            UvsReason::ConfigError(_) => true,

            // Others are normal business operations
            _ => false,
        }
    }

    /// Get error category name for monitoring and metrics
    /// 获取错误类别名称用于监控和指标
    pub fn category_name(&self) -> &'static str {
        match self {
            UvsReason::ValidationError(_) => "validation",
            UvsReason::BusinessError(_) => "business",
            UvsReason::NotFoundError(_) => "not_found",
            UvsReason::PermissionError(_) => "permission",
            UvsReason::DataError(_, _) => "data",
            UvsReason::SystemError(_) => "system",
            UvsReason::NetworkError(_) => "network",
            UvsReason::ResourceError(_) => "resource",
            UvsReason::TimeoutError(_) => "timeout",
            UvsReason::ConfigError(_) => "config",
            UvsReason::ExternalError(_) => "external",
            UvsReason::LogicError(_) => "logic",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_ranges() {
        // Business layer (100-199)
        assert_eq!(UvsReason::validation_error("test").error_code(), 100);
        assert_eq!(UvsReason::business_error("test").error_code(), 101);
        assert_eq!(UvsReason::not_found_error("test").error_code(), 102);
        assert_eq!(UvsReason::permission_error("test").error_code(), 103);

        // Infrastructure layer (200-299)
        assert_eq!(UvsReason::data_error("test", None).error_code(), 200);
        assert_eq!(UvsReason::system_error("test").error_code(), 201);
        assert_eq!(UvsReason::network_error("test").error_code(), 202);
        assert_eq!(UvsReason::resource_error("test").error_code(), 203);
        assert_eq!(UvsReason::timeout_error("test").error_code(), 204);

        // Configuration & external layer (300-399)
        assert_eq!(UvsReason::core_conf("test").error_code(), 300);
        assert_eq!(UvsReason::external_error("test").error_code(), 301);
    }

    #[test]
    fn test_retryable_errors() {
        assert!(UvsReason::network_error("timeout").is_retryable());
        assert!(UvsReason::timeout_error("request timeout").is_retryable());
        assert!(!UvsReason::validation_error("invalid input").is_retryable());
        assert!(!UvsReason::business_error("insufficient funds").is_retryable());
    }

    #[test]
    fn test_high_severity_errors() {
        assert!(UvsReason::system_error("disk full").is_high_severity());
        assert!(UvsReason::resource_error("out of memory").is_high_severity());
        assert!(!UvsReason::validation_error("bad format").is_high_severity());
        assert!(!UvsReason::NotFoundError("user not found".into()).is_high_severity());
    }

    #[test]
    fn test_category_names() {
        assert_eq!(UvsReason::network_error("test").category_name(), "network");
        assert_eq!(
            UvsReason::business_error("test").category_name(),
            "business"
        );
        assert_eq!(UvsReason::core_conf("test").category_name(), "config");
    }

    #[test]
    fn test_trait_implementations() {
        // Test that trait implementations work correctly
        let reason: UvsReason = UvsReason::from_net("network error".to_string());
        assert_eq!(reason.error_code(), 202);

        let reason: UvsReason = UvsReason::from_validation("validation error".to_string());
        assert_eq!(reason.error_code(), 100);

        let reason: UvsReason = UvsReason::from_external("external error".to_string());
        assert_eq!(reason.error_code(), 301);
    }
}
