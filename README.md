# orion-error
[Chinese Version](#orion-error-zh)

Structured error handling library for building large-scale applications, providing complete error context tracking and flexible aggregation strategies.

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/galaxy-sec/orion-error)

[![CI](https://github.com/galaxy-sec/orion-error/workflows/CI/badge.svg)](https://github.com/galaxy-sec/orion-error/actions)
[![Coverage Status](https://codecov.io/gh/galaxy-sec/orion-error/branch/main/graph/badge.svg)](https://codecov.io/gh/galaxy-sec/orion-error)
[![crates.io](https://img.shields.io/crates/v/orion-error.svg)](https://crates.io/crates/orion-error)

## Features

- **Structured Errors**: Support multi-layer error aggregation with full error chain
- **Hierarchical Error Classification**: Three-tier classification system with clear boundaries
  - **Business Layer (100-199)**: User-facing expected errors
  - **Infrastructure Layer (200-299)**: System-level failures
  - **Configuration & External Layer (300-399)**: Environment and third-party issues
- **Error Types**: 10 specific error types with semantic meaning
- **Smart Error Analysis**: Built-in retryability and severity assessment
- **Context Tracing**: Support multi-level context information
- **Error Codes**: Organized code system by error layers
- **Error Conversion**: Multiple conversion strategies:
  ```rust
  .owe()           // Convert with specific reason
  .owe_validation() // Convert to validation error
  .owe_biz()       // Convert to business error
  .owe_sys()       // Mark as system error
  .err_conv()      // Automatic type conversion
  ```

## Installation

Add to Cargo.toml:
```toml
[dependencies]
orion-error = "0.3"
```

## Quick Start
## Core Concepts

### Error Definition
Define domain errors with enum and implement `DomainReason`:
```rust
#[derive(Debug, Display)]
enum OrderReason {
    InsufficientFunds,
    UserNotFound,
}

impl DomainReason for OrderReason {}
```

### Error Construction
Build error context with chaining:
```rust
validate_user(user_id)
    .want("Validate user")       // Add operation description
    .with_detail("uid:123")      // Add debug details
    .owe(OrderError::UserNotFound) // Convert to upper error type
```

### Error Handling
#### Conversion Strategies
| Method        | Description                      |
|---------------|----------------------------------|
| `.owe()`      | Convert to specific biz error    |
| `.owe_sys()`  | Mark as system error             |
| `.err_conv()` | Auto-detect error type conversion|

#### Handling Patterns
```rust
// Pattern 1: Direct conversion
parse_input().map_err(|e| e.owe(OrderError::ParseFailed))?;

// Pattern 2: Add context
db.query()
   .want("Read order data")
   .with_detail(format!("order_id={id}"))
   .err_conv()?;
```

## Advanced Features

### Error Composition
```rust
use orion_error::{UvsReason, StructError, ErrorWith, WithContext};

// Compose errors with rich context
fn complex_operation() -> Result<(), StructError<UvsReason>> {
    let mut ctx = WithContext::want("complex_operation");
    ctx.with("step", "validation");
    ctx.with("input_type", "user_request");

    // Validation error with context
    validate_input(&request)
        .want("input validation")
        .with(&ctx)?;

    ctx.with("step", "business_logic");

    // Business error with context
    check_business_rules(&request)
        .want("business rules check")
        .with(&ctx)?;

    ctx.with("step", "persistence");

    // System error with context
    save_to_database(&processed_data)
        .want("data persistence")
        .with(&ctx)?;

    Ok(())
}
```

### Error Propagation Strategies
```rust
use orion_error::{UvsReason, StructError, ErrorOwe};

// Different conversion strategies
fn process_with_strategies() -> Result<(), StructError<UvsReason>> {
    // Strategy 1: Convert to validation error
    let input = get_input().owe_validation()?;

    // Strategy 2: Convert to business error
    let validated = validate(input).owe_biz()?;

    // Strategy 3: Convert to system error
    let result = process(validated).owe_sys()?;

    // Strategy 4: Convert with custom reason
    let final_result = finalize(result).owe(UvsReason::business_error("finalization failed"))?;

    Ok(final_result)
}
```

### Error Recovery Patterns
```rust
use orion_error::UvsReason;

fn robust_operation() -> Result<(), MyError> {
    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        attempts += 1;

        match attempt_operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                // Check if error is retryable and within attempt limit
                if error.is_retryable() && attempts < max_attempts {
                    log::warn!("Attempt {} failed, retrying: {}", attempts, error);
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    continue;
                } else {
                    return Err(error.into());
                }
            }
        }
    }
}

// Fallback pattern
fn operation_with_fallback() -> Result<String, MyError> {
    // Try primary method
    primary_method().map_err(|e| {
        log::warn!("Primary method failed: {}", e);
        // Convert to business error with fallback context
        UvsReason::business_error("primary method unavailable, fallback not implemented")
    })
}
```

### Error Monitoring Integration
```rust
use orion_error::UvsReason;

// Integration with monitoring systems
struct ErrorMonitor;

impl ErrorMonitor {
    fn track_error(&self, error: &UvsReason) {
        // Send to monitoring system
        let event = MonitoringEvent {
            error_code: error.error_code(),
            category: error.category_name(),
            severity: if error.is_high_severity() { "high" } else { "normal" },
            retryable: error.is_retryable(),
            message: error.to_string(),
            timestamp: chrono::Utc::now(),
        };

        self.send_to_monitoring(event);
    }

    fn should_alert(&self, error: &UvsReason) -> bool {
        error.is_high_severity() ||
        error.error_code() >= 200 // Infrastructure layer errors
    }
}

// Usage in application
fn handle_api_error(error: UvsReason) -> HttpResponse {
    let monitor = ErrorMonitor::new();
    monitor.track_error(&error);

    if monitor.should_alert(&error) {
        alert_team(&error);
    }

    // Convert error to HTTP response based on category
    match error.category_name() {
        "validation" => HttpResponse::BadRequest().json(error.to_string()),
        "business" => HttpResponse::Conflict().json(error.to_string()),
        "not_found" => HttpResponse::NotFound().json(error.to_string()),
        "permission" => HttpResponse::Unauthorized().json(error.to_string()),
        "system" | "network" | "timeout" | "resource" => {
            HttpResponse::ServiceUnavailable().json(error.to_string())
        }
        _ => HttpResponse::InternalServerError().json(error.to_string()),
    }
}
```

## Migration Guide

### From Version 0.2 to 0.3

The error classification system has been significantly improved with a new hierarchical structure. Here's how to migrate:

#### **Error Type Changes**
```rust
// Old way (v0.2)
use orion_error::UvsReason;

let error = UvsReason::BizError("business logic failed".into());
let error = UvsReason::LogicError("logic error".into());
let error = UvsReason::Timeout("timeout occurred".into());

// New way (v0.3)
use orion_error::UvsReason;

let error = UvsReason::business_error("business logic failed");
let error = UvsReason::validation_error("logic error");
let error = UvsReason::timeout_error("timeout occurred");
```

#### **Trait Method Changes**
```rust
// Old way (v0.2)
let error = string_value.from_biz();
let error = string_value.from_logic();
let error = string_value.from_rule();

// New way (v0.3)
let error = UvsReason::from_biz(string_value);
let error = UvsReason::from_validation(string_value);
// Note: Rule errors have been removed, use ValidationError instead
```

#### **Error Code Changes**
Error codes have been reorganized by layers:
```rust
// Old codes
BizError -> 101
LogicError -> 100
Timeout -> 109

// New codes
ValidationError -> 100
BusinessError -> 101
NotFoundError -> 102
PermissionError -> 103
TimeoutError -> 204
```

#### **New Features Usage**
```rust
// Check retryability
if error.is_retryable() {
    // Implement retry logic
}

// Check severity
if error.is_high_severity() {
    // Send high priority alert
}

// Get category for metrics
let category = error.category_name();
```

## Full Example
See [examples/order_case.rs](examples/order_case.rs) for a comprehensive example showing all the new error classification features in action.

## Error Classification System

The `UvsReason` provides a comprehensive error classification system organized in three distinct layers:

### 🏗️ Error Layer Architecture

#### **Business Layer Errors (100-199)**
These are user-facing errors that are expected in normal application operation.

| Error Type | Code | Description | When to Use |
|------------|------|-------------|-------------|
| `ValidationError` | 100 | Input validation failures | Invalid parameters, format errors, constraint violations |
| `BusinessError` | 101 | Business logic violations | Rule violations, state conflicts, domain-specific errors |
| `NotFoundError` | 102 | Resource not found | Database record missing, file not found, user doesn't exist |
| `PermissionError` | 103 | Authorization failures | Access denied, authentication failed, insufficient permissions |

#### **Infrastructure Layer Errors (200-299)**
System-level failures that should be rare and often require operational attention.

| Error Type | Code | Description | When to Use |
|------------|------|-------------|-------------|
| `DataError` | 200 | Data processing errors | Database failures, data corruption, serialization errors |
| `SystemError` | 201 | OS and file system errors | Disk full, file permission issues, OS-level failures |
| `NetworkError` | 202 | Network connectivity errors | HTTP timeouts, connection failures, DNS resolution |
| `ResourceError` | 203 | Resource exhaustion | Memory full, CPU overload, connection pool exhausted |
| `TimeoutError` | 204 | Operation timeouts | Database query timeout, external service timeout |

#### **Configuration & External Layer Errors (300-399)**
Environment-related issues and third-party service failures.

| Error Type | Code | Description | When to Use |
|------------|------|-------------|-------------|
| `ConfigError` | 300 | Configuration issues | Missing config files, invalid configuration values |
| `ExternalError` | 301 | Third-party service errors | Payment gateway failures, external API failures |

## Error Classification
## Error Display
Built-in Display implementation shows full error chain:
```text
[Error Code 500] Insufficient funds
Caused by:
  0: User not found (code103)
     Detail: uid:456
     Context: "Validate funds"
  1: Storage full (code500)
     Context: "Save order"
```

## Contributing
Issues and PRs are welcome. Please follow existing code style.

## License
MIT License



---

# orion-error-zh <a name="orion-error-zh"></a>


用于构建大型应用程序的结构化错误处理库，提供完整的错误上下文追踪和灵活的归集策略

## 功能特性

- **结构化错误**：支持多层错误归集，保留完整错误链
- **错误分类**：
  - 业务错误（BizError） - 领域相关的可恢复错误
  - 系统错误（SysError） - 基础设施级别的严重错误
- **上下文追踪**：支持添加多级上下文信息
- **错误代码**：支持自定义错误代码体系
- **错误转换**：提供多种错误转换策略：
  ```rust
  .owe()      // 转换为业务错误
  .owe_sys()  // 转换为系统错误
  .err_conv() // 自动推导转换
  ```
  ## 安装

  在 Cargo.toml 中添加：
  ```toml
  [dependencies]
  orion-error = "0.2"
  ```



## 核心概念

### 错误定义
使用枚举定义领域错误类型，实现 `DomainReason` trait：
```rust
#[derive(Debug, Display)]
enum OrderReason {
    InsufficientFunds,
    UserNotFound,
}

impl DomainReason for OrderReason {}
```

### 错误构造
使用链式调用构建错误上下文：
```rust
validate_user(user_id)
    .want("验证用户")          // 添加操作描述
    .with_detail("uid:123")    // 添加调试细节
    .owe(OrderError::UserNotFound) // 转换为上层错误类型
```

### 错误处理
#### 转换策略
| 方法         | 说明                          |
|--------------|-----------------------------|
| `.owe()`     | 转换为指定业务错误，保留原始错误链   |
| `.owe_sys()` | 标记为系统级错误                |
| `.err_conv()`| 自动推导错误类型转换             |

#### 处理模式
```rust
// 模式1：直接转换
parse_input().map_err(|e| e.owe(OrderError::ParseFailed))?;

// 模式2：添加上下文
db.query()
   .want("读取订单数据")
   .with_detail(format!("order_id={id}"))
   .err_conv()?;
```
## 完整示例
见 [examples/order_case.rs](examples/order_case.rs)

## 错误展示
内置 Display 实现可展示完整错误链：
```text
[错误代码 500] 账户余额不足
Caused by:
  0: 用户不存在 (代码103)
     详情: uid:456
     上下文: "验证资金"
  1: 存储空间不足 (代码500)
     上下文: "保存订单"
```
## 贡献指南
欢迎提交 issue 和 PR，请遵循现有代码风格

## 许可证
MIT License
