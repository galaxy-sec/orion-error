# orion-error
[Chinese Version](#orion-error-zh)

Structured error handling library for building large-scale applications, providing complete error context tracking and flexible aggregation strategies.

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/galaxy-sec/orion-error)

## Features

- **Structured Errors**: Support multi-layer error aggregation with full error chain
- **Error Classification**:
  - Business Error (BizError) - Domain-specific recoverable errors
  - System Error (SysError) - Infrastructure-level critical errors
- **Context Tracing**: Support multi-level context information
- **Error Codes**: Customizable error code system
- **Error Conversion**: Multiple conversion strategies:
  ```rust
  .owe()      // Convert to business error
  .owe_sys()  // Mark as system error
  .err_conv() // Automatic type conversion
  ```

## Installation

Add to Cargo.toml:
```toml
[dependencies]
orion-error = "0.2"
```

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

## Full Example
See [examples/order_case.rs](examples/order_case.rs)

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
