# 使用教程

本教程将指导您如何使用 orion-error 库进行结构化错误处理。我们将从基础概念开始，逐步深入到高级用法。

## 目录

1. [快速开始](#快速开始)
2. [基础错误处理](#基础错误处理)
3. [错误上下文管理](#错误上下文管理)
4. [错误转换策略](#错误转换策略)
5. [重试机制实现](#重试机制实现)
6. [监控和日志集成](#监控和日志集成)
7. [完整示例：Web 应用](#完整示例web-应用)
8. [最佳实践](#最佳实践)

## 快速开始

### 项目设置

首先，将 orion-error 添加到您的 `Cargo.toml` 中：

```toml
[dependencies]
orion-error = "0.3"
```

### 基本错误定义

让我们从定义一个简单的领域错误开始：

```rust
use orion_error::{StructError, UvsReason, DomainReason, ErrorCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone, Serialize)]
pub enum UserServiceError {
    #[error("user not found")]
    UserNotFound,
    #[error("invalid user data")]
    InvalidUserData,
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("{0}")]
    Universal(UvsReason),
}

impl ErrorCode for UserServiceError {
    fn error_code(&self) -> i32 {
        match self {
            UserServiceError::UserNotFound => 1000,
            UserServiceError::InvalidUserData => 1001,
            UserServiceError::EmailAlreadyExists => 1002,
            UserServiceError::Universal(reason) => reason.error_code(),
        }
    }
}

impl DomainReason for UserServiceError {}
```

### 第一个错误处理示例

```rust
use orion_error::{ErrorWith, WithContext};

pub fn create_user(email: &str, name: &str) -> Result<User, StructError<UserServiceError>> {
    // 创建错误上下文
    let mut ctx = WithContext::want("create_user");
    ctx.with("email", email);
    ctx.with("name", name);

    // 验证邮箱格式
    if !email.contains('@') {
        return Err(StructError::from(UserServiceError::InvalidUserData)
            .want("email validation")
            .with(&ctx)
            .with_detail("Email must contain @ symbol"));
    }

    // 检查邮箱是否已存在
    if email == "existing@example.com" {
        return Err(StructError::from(UserServiceError::EmailAlreadyExists)
            .want("email uniqueness check")
            .with(&ctx));
    }

    // 创建用户
    let user = User {
        id: 1,
        email: email.to_string(),
        name: name.to_string(),
    };

    Ok(user)
}
```

## 基础错误处理

### 定义领域错误

领域错误是您业务逻辑中的特定错误类型。让我们详细看看如何定义：

```rust
#[derive(Debug, Error, PartialEq, Clone, Serialize)]
pub enum OrderError {
    #[error("order not found")]
    OrderNotFound,
    #[error("invalid order status")]
    InvalidOrderStatus,
    #[error("insufficient inventory")]
    InsufficientInventory,
    #[error("order amount too large")]
    OrderAmountTooLarge,
    #[error("{0}")]
    Universal(UvsReason),
}

impl ErrorCode for OrderError {
    fn error_code(&self) -> i32 {
        match self {
            OrderError::OrderNotFound => 2000,
            OrderError::InvalidOrderStatus => 2001,
            OrderError::InsufficientInventory => 2002,
            OrderError::OrderAmountTooLarge => 2003,
            OrderError::Universal(reason) => reason.error_code(),
        }
    }
}

impl DomainReason for OrderError {}
```

### 使用 `StructError::builder` 简化构造

当需要附加详细信息或上下文时，可使用 `StructError::builder` 链式构建错误，避免在 `Result` 链中频繁调用 `with_*`：

```rust
let err = StructError::builder(OrderError::InsufficientInventory)
    .detail(format!("剩余库存 {remaining}，请求数量 {amount}"))
    .context_ref(&ctx)
    .finish();

return Err(err);
```

`context_ref` 接受现有的 `OperationContext` 引用，内部自动克隆上下文栈，可与 `err_conv()` 等转换方法组合使用。

### 基本错误创建和使用

#### 使用预定义的错误类型

```rust
use orion_error::UvsReason;

fn validate_user_input(input: &str) -> Result<(), StructError<UvsReason>> {
    if input.is_empty() {
        return Err(StructError::from(
            UvsReason::validation_error("input cannot be empty")
        ));
    }

    if input.len() > 100 {
        return Err(StructError::from(
            UvsReason::validation_error("input too long, max 100 characters")
        ));
    }

    Ok(())
}

fn check_business_rules(user_id: u32, amount: f64) -> Result<(), StructError<UvsReason>> {
    if amount <= 0.0 {
        return Err(StructError::from(
            UvsReason::business_error("amount must be positive")
        ));
    }

    if user_id == 0 {
        return Err(StructError::from(
            UvsReason::business_error("invalid user id")
        ));
    }

    Ok(())
}

fn find_user(user_id: u32) -> Result<User, StructError<UvsReason>> {
    if user_id != 123 {
        return Err(StructError::from(
            UvsReason::not_found_error("user does not exist")
        ));
    }

    Ok(User {
        id: user_id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    })
}
```

#### 转换为结构化错误

```rust
fn process_request() -> Result<String, StructError<UvsReason>> {
    // 从其他错误类型转换
    let io_result: Result<String, std::io::Error> = std::fs::read_to_string("config.json");
    
    // 转换为我们的错误类型
    let config_content = io_result.map_err(|e| {
        StructError::from(UvsReason::system_error(format!("failed to read config: {}", e)))
    })?;

    Ok(config_content)
}
```

## 错误上下文管理

### 添加错误上下文

错误上下文提供了错误发生时的环境信息，有助于调试和问题定位。

```rust
use orion_error::{ErrorWith, WithContext};

pub fn process_order(order_id: u32, user_id: u32) -> Result<Order, StructError<OrderError>> {
    // 创建主上下文
    let mut main_ctx = WithContext::want("process_order");
    main_ctx.with("order_id", order_id.to_string());
    main_ctx.with("user_id", user_id.to_string());

    // 步骤1: 验证订单存在
    let order = find_order(order_id)
        .want("find order")
        .with(&main_ctx)?;

    // 步骤2: 验证用户权限
    check_user_permission(user_id)
        .want("check user permission")
        .with(&main_ctx)?;

    // 步骤3: 处理订单逻辑
    let processed_order = process_order_logic(&order)
        .want("process order logic")
        .with(&main_ctx)?;

    Ok(processed_order)
}

fn find_order(order_id: u32) -> Result<Order, StructError<OrderError>> {
    let mut ctx = WithContext::want("find_order");
    ctx.with("order_id", order_id.to_string());
    ctx.with("source", "database");

    if order_id == 0 {
        return Err(StructError::from(OrderError::OrderNotFound)
            .want("validate order id")
            .with(&ctx)
            .with_detail("order_id cannot be zero"));
    }

    // 模拟数据库查找
    if order_id != 12345 {
        return Err(StructError::from(OrderError::OrderNotFound)
            .want("database query")
            .with(&ctx)
            .position("database.rs:42"));
    }

    Ok(Order {
        id: order_id,
        user_id: 123,
        amount: 100.0,
        status: "pending".to_string(),
    })
}
```

### 使用 `OperationScope` 自动标记成功

`OperationContext` 默认在 Drop 时视为失败，需要显式调用 `mark_suc()` 才会记录成功日志。通过 RAII guard 可以让成功路径更简单：

```rust
let mut ctx = OperationContext::want("process_order").with_auto_log();
{
    let mut scope = ctx.scoped_success();
    scope.record("order_id", order_id.to_string());

    if !validate(order_id) {
        scope.mark_failure(); // 保持失败状态
        return Err(build_error()?);
    }
}
```

`scope()` 创建默认失败的 guard；`scoped_success()` 在作用域结束时自动将结果标记为成功。若确实需要取消，可调用 `scope.cancel()`。

### 多层上下文传播

错误上下文可以在多层函数调用中传播，提供完整的调用链信息：

```rust
pub mod service {
    use super::*;

    pub fn user_service_create_user(email: &str, name: &str) -> Result<User, StructError<UserServiceError>> {
        let mut service_ctx = WithContext::want("user_service");
        service_ctx.with("service", "user_management");
        service_ctx.with("operation", "create_user");

        repository::user_repository_create(email, name)
            .want("create user in repository")
            .with(&service_ctx)
    }
}

pub mod repository {
    use super::*;

    pub fn user_repository_create(email: &str, name: &str) -> Result<User, StructError<UserServiceError>> {
        let mut repo_ctx = WithContext::want("user_repository");
        repo_ctx.with("table", "users");
        repo_ctx.with("operation", "insert");

        // 模拟数据库操作
        if email == "duplicate@example.com" {
            return Err(StructError::from(UserServiceError::EmailAlreadyExists)
                .want("database insert")
                .with(&repo_ctx)
                .position("database.rs:156")
                .with_detail("UNIQUE constraint violation: users.email"));
        }

        Ok(User {
            id: 1,
            email: email.to_string(),
            name: name.to_string(),
        })
    }
}

// 使用示例
fn main() {
    match service::user_service_create_user("duplicate@example.com", "John Doe") {
        Ok(user) => println!("Created user: {:?}", user),
        Err(error) => {
            println!("Error creating user:");
            println!("Error code: {}", error.error_code());
            println!("Error message: {}", error);
            
            // 上下文信息会显示完整的调用链
            for ctx in error.context() {
                println!("Context: {:?}", ctx);
            }
        }
    }
}
```

### 动态上下文信息

您可以根据运行时条件动态添加上下文信息：

```rust
pub fn process_payment(payment: &Payment) -> Result<(), StructError<PaymentError>> {
    let mut ctx = WithContext::want("process_payment");
    ctx.with("payment_id", payment.id.to_string());
    ctx.with("amount", payment.amount.to_string());
    ctx.with("currency", &payment.currency);

    // 根据支付方式添加特定上下文
    match payment.method {
        PaymentMethod::CreditCard => {
            ctx.with("payment_method", "credit_card");
            ctx.with("card_type", payment.card_type.clone());
            ctx.with("last_four", payment.last_four.clone());
        }
        PaymentMethod::PayPal => {
            ctx.with("payment_method", "paypal");
            ctx.with("paypal_email", payment.paypal_email.clone());
        }
        PaymentMethod::BankTransfer => {
            ctx.with("payment_method", "bank_transfer");
            ctx.with("bank_account", payment.bank_account.clone());
        }
    }

    // 根据环境添加上下文
    if cfg!(debug_assertions) {
        ctx.with("environment", "development");
    } else {
        ctx.with("environment", "production");
    }

    // 处理支付逻辑
    payment_gateway::process(payment)
        .want("payment gateway processing")
        .with(&ctx)
}
```

## 错误转换策略

### 使用 ErrorOwe trait

`ErrorOwe` trait 提供了将普通错误转换为特定类型错误的便捷方法。

```rust
use orion_error::ErrorOwe;

fn process_file_upload(file_path: &str) -> Result<String, StructError<UvsReason>> {
    // 读取文件，将 IO 错误转换为验证错误
    let file_content = std::fs::read_to_string(file_path)
        .owe_validation()?;

    // 验证文件内容，将业务逻辑错误转换为业务错误
    let validated_content = validate_content(&file_content)
        .owe_biz()?;

    // 处理文件，将处理错误转换为系统错误
    let processed_content = process_content(&validated_content)
        .owe_sys()?;

    Ok(processed_content)
}

fn validate_content(content: &str) -> Result<&str, String> {
    if content.is_empty() {
        return Err("content cannot be empty".to_string());
    }
    if content.len() > 10000 {
        return Err("content too large".to_string());
    }
    Ok(content)
}

fn process_content(content: &str) -> Result<String, String> {
    if content.contains("error") {
        return Err("processing failed: invalid content".to_string());
    }
    Ok(content.to_uppercase())
}
```

### 自定义错误转换

您可以根据业务需求实现自定义的错误转换逻辑：

```rust
pub fn convert_database_error<E: std::fmt::Display>(error: E) -> StructError<UvsReason> {
    let error_msg = error.to_string();
    
    let reason = match error_msg.as_str() {
        msg if msg.contains("connection refused") => {
            UvsReason::network_error("database connection failed")
        }
        msg if msg.contains("timeout") => {
            UvsReason::timeout_error("database operation timeout")
        }
        msg if msg.contains("out of memory") => {
            UvsReason::resource_error("database out of memory")
        }
        msg if msg.contains("permission denied") => {
            UvsReason::permission_error("database permission denied")
        }
        msg if msg.contains("duplicate key") => {
            UvsReason::business_error("duplicate entry constraint violation")
        }
        msg if msg.contains("syntax error") => {
            UvsReason::validation_error("database query syntax error")
        }
        _ => {
            UvsReason::system_error(format!("unexpected database error: {}", error_msg))
        }
    };
    
    StructError::from(reason)
        .with_detail(error_msg)
        .position("database.rs:42")
}

// 使用自定义转换
pub fn execute_query(query: &str) -> Result<QueryResult, StructError<UvsReason>> {
    let db_result: Result<QueryResult, sqlx::Error> = execute_database_query(query);
    
    db_result.map_err(convert_database_error)
}
```

### 条件错误转换

根据不同的条件进行不同的错误转换：

```rust
pub fn handle_external_api_response(response: HttpResponse) -> Result<ApiResponse, StructError<UvsReason>> {
    match response.status() {
        StatusCode::OK => Ok(response.json::<ApiResponse>()?),
        StatusCode::BAD_REQUEST => {
            Err(StructError::from(
                UvsReason::validation_error("invalid request parameters")
            ))
        }
        StatusCode::UNAUTHORIZED => {
            Err(StructError::from(
                UvsReason::permission_error("authentication failed")
            ))
        }
        StatusCode::FORBIDDEN => {
            Err(StructError::from(
                UvsReason::permission_error("insufficient permissions")
            ))
        }
        StatusCode::NOT_FOUND => {
            Err(StructError::from(
                UvsReason::not_found_error("resource not found")
            ))
        }
        StatusCode::CONFLICT => {
            Err(StructError::from(
                UvsReason::business_error("resource conflict")
            ))
        }
        StatusCode::TOO_MANY_REQUESTS => {
            Err(StructError::from(
                UvsReason::external_error("rate limit exceeded")
            ))
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            Err(StructError::from(
                UvsReason::external_error("internal server error")
            ))
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            Err(StructError::from(
                UvsReason::external_error("service temporarily unavailable")
            ))
        }
        status => {
            Err(StructError::from(
                UvsReason::external_error(format!("unexpected status code: {}", status))
            ))
        }
    }
}
```

## 重试机制实现

### 基本重试逻辑

利用 `is_retryable()` 方法实现智能重试机制：

```rust
use std::time::Duration;
use tokio::time::sleep;

pub async fn with_retry<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    base_delay: Duration,
) -> Result<T, StructError<E>>
where
    F: FnMut() -> Result<T, StructError<E>>,
    E: DomainReason + From<UvsReason> + ErrorCode,
{
    for attempt in 1..=max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                // 检查错误是否可重试
                let should_retry = error.reason().error_code() >= 200 && 
                                  error.reason().error_code() <= 204; // 基础设施层错误可重试

                if !should_retry || attempt == max_attempts {
                    return Err(error);
                }

                // 计算指数退避延迟
                let delay = base_delay * 2u32.pow(attempt - 1);
                
                log::warn!(
                    "Attempt {} failed, retrying in {:?}: {}",
                    attempt,
                    delay,
                    error
                );
                
                sleep(delay).await;
            }
        }
    }

    unreachable!("loop should always return")
}

// 使用示例
async fn fetch_data_with_retry() -> Result<Data, StructError<UvsReason>> {
    with_retry(
        || fetch_data_from_api(),
        3, // 最大重试次数
        Duration::from_secs(1), // 基础延迟
    ).await
}

fn fetch_data_from_api() -> Result<Data, StructError<UvsReason>> {
    // 模拟可能失败的 API 调用
    if rand::random() {
        Err(StructError::from(UvsReason::network_error("connection timeout")))
    } else {
        Ok(Data { value: "success".to_string() })
    }
}
```

### 高级重试策略

实现更复杂的重试策略，包括熔断模式和错误分类重试：

```rust
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub retryable_error_types: Vec<&'static str>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            retryable_error_types: vec!["network", "timeout", "system", "resource", "external"],
        }
    }
}

pub struct RetryManager {
    config: RetryConfig,
    circuit_breaker: CircuitBreaker,
}

impl RetryManager {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            circuit_breaker: CircuitBreaker::new(),
        }
    }

    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, StructError<E>>
    where
        F: Fn() -> Result<T, StructError<E>>,
        E: DomainReason + From<UvsReason> + ErrorCode,
    {
        // 检查熔断器状态
        if self.circuit_breaker.is_open() {
            return Err(StructError::from(
                UvsReason::external_error("circuit breaker is open")
            ));
        }

        for attempt in 1..=self.config.max_attempts {
            match operation() {
                Ok(result) => {
                    self.circuit_breaker.record_success();
                    return Ok(result);
                }
                Err(error) => {
                    let error_category = match error.reason().error_code() {
                        100 => "validation",
                        101 => "business",
                        102 => "not_found",
                        103 => "permission",
                        200 => "data",
                        201 => "system",
                        202 => "network",
                        203 => "resource",
                        204 => "timeout",
                        300 => "config",
                        301 => "external",
                        _ => "unknown",
                    };

                    // 检查是否应该重试
                    let should_retry = self.config.retryable_error_types.contains(&error_category) &&
                                      attempt < self.config.max_attempts;

                    if !should_retry {
                        self.circuit_breaker.record_failure();
                        return Err(error);
                    }

                    // 计算延迟时间
                    let delay_ms = (self.config.base_delay.as_millis() as f64 * 
                                   self.config.backoff_multiplier.powi(attempt as i32 - 1))
                        .min(self.config.max_delay.as_millis() as f64);
                    
                    let delay = Duration::from_millis(delay_ms as u64);

                    log::warn!(
                        "Retry attempt {}/{} for {} error in {:?}: {}",
                        attempt,
                        self.config.max_attempts,
                        error_category,
                        delay,
                        error
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(StructError::from(
            UvsReason::external_error("max retry attempts exceeded")
        ))
    }
}

// 简单的熔断器实现
#[derive(Debug)]
struct CircuitBreaker {
    failure_count: u32,
    last_failure_time: std::time::Instant,
    is_open: bool,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new() -> Self {
        Self {
            failure_count: 0,
            last_failure_time: std::time::Instant::now(),
            is_open: false,
            threshold: 5,
            timeout: Duration::from_secs(60),
        }
    }

    fn is_open(&self) -> bool {
        if self.is_open {
            // 检查是否超时可以重试
            self.last_failure_time.elapsed() < self.timeout
        } else {
            false
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.is_open = false;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = std::time::Instant::now();
        
        if self.failure_count >= self.threshold {
            self.is_open = true;
            log::error!("Circuit breaker opened after {} failures", self.threshold);
        }
    }
}

// 使用示例
async fn robust_api_call() -> Result<Data, StructError<UvsReason>> {
    let retry_config = RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 1.5,
        retryable_error_types: vec!["network", "timeout", "external"],
    };

    let retry_manager = RetryManager::new(retry_config);

    retry_manager.execute(|| call_external_api()).await
}
```

### 分类重试策略

根据不同的错误类型实施不同的重试策略：

```rust
pub enum RetryStrategy {
    /// 立即重试，适用于暂时性错误
    Immediate,
    /// 指数退避重试，适用于网络和超时错误
    ExponentialBackoff { base_delay: Duration, max_delay: Duration },
    /// 固定间隔重试，适用于资源不足错误
    FixedInterval { interval: Duration },
    /// 不重试，适用于业务逻辑错误
    NoRetry,
}

impl RetryStrategy {
    pub fn for_error<E: ErrorCode + From<UvsReason>>(error: &StructError<E>) -> Self {
        match error.reason().error_code() {
            // 网络错误 - 指数退避
            202 => RetryStrategy::ExponentialBackoff {
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(30),
            },
            // 超时错误 - 指数退避
            204 => RetryStrategy::ExponentialBackoff {
                base_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(60),
            },
            // 资源错误 - 固定间隔
            203 => RetryStrategy::FixedInterval {
                interval: Duration::from_secs(5),
            },
            // 系统错误 - 固定间隔
            201 => RetryStrategy::FixedInterval {
                interval: Duration::from_secs(10),
            },
            // 外部服务错误 - 指数退避
            301 => RetryStrategy::ExponentialBackoff {
                base_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(300),
            },
            // 验证错误 - 不重试
            100 => RetryStrategy::NoRetry,
            // 业务错误 - 不重试
            101 => RetryStrategy::NoRetry,
            // 资源不存在 - 不重试
            102 => RetryStrategy::NoRetry,
            // 权限错误 - 不重试
            103 => RetryStrategy::NoRetry,
            // 数据错误 - 不重试
            200 => RetryStrategy::NoRetry,
            // 配置错误 - 不重试
            300 => RetryStrategy::NoRetry,
            _ => RetryStrategy::NoRetry,
        }
    }
}

pub async fn smart_retry<F, T, E>(
    operation: F,
    max_attempts: u32,
) -> Result<T, StructError<E>>
where
    F: FnMut() -> Result<T, StructError<E>>,
    E: DomainReason + From<UvsReason> + ErrorCode,
{
    for attempt in 1..=max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                let strategy = RetryStrategy::for_error(&error);

                match strategy {
                    RetryStrategy::NoRetry | RetryStrategy::ExponentialBackoff { .. }
                        if attempt == max_attempts => return Err(error),
                    RetryStrategy::NoRetry => return Err(error),
                    RetryStrategy::Immediate => {
                        log::info!("Immediate retry attempt {}/{}", attempt, max_attempts);
                        // 立即重试，不延迟
                    }
                    RetryStrategy::ExponentialBackoff { base_delay, max_delay } => {
                        let delay_ms = (base_delay.as_millis() as f64 * 2f64.powi(attempt as i32 - 1))
                            .min(max_delay.as_millis() as f64);
                        let delay = Duration::from_millis(delay_ms as u64);
                        
                        log::info!("Exponential backoff retry attempt {}/{} in {:?}", 
                                 attempt, max_attempts, delay);
                        tokio::time::sleep(delay).await;
                    }
                    RetryStrategy::FixedInterval { interval } => {
                        log::info!("Fixed interval retry attempt {}/{} in {:?}", 
                                 attempt, max_attempts, interval);
                        tokio::time::sleep(interval).await;
                    }
                }
            }
        }
    }

    unreachable!("loop should always return")
}
```

## 监控和日志集成

### 错误监控集成

将错误处理与监控系统集成，实现错误统计和告警：

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub timestamp: DateTime<Utc>,
    pub error_code: i32,
    pub error_category: String,
    pub error_message: String,
    pub severity: String,
    pub retryable: bool,
    pub context: Vec<(String, String)>,
    pub position: Option<String>,
    pub target: Option<String>,
    pub details: Option<String>,
}

pub struct ErrorMonitor {
    metrics: ErrorMetrics,
    alert_thresholds: AlertThresholds,
}

impl ErrorMonitor {
    pub fn new() -> Self {
        Self {
            metrics: ErrorMetrics::new(),
            alert_thresholds: AlertThresholds::default(),
        }
    }

    pub fn track_error<E: ErrorCode + std::fmt::Display>(&mut self, error: &StructError<E>) {
        let event = ErrorEvent {
            timestamp: Utc::now(),
            error_code: error.error_code(),
            error_category: self.get_error_category(error),
            error_message: error.to_string(),
            severity: self.get_severity(error),
            retryable: self.is_retryable(error),
            context: self.extract_context(error),
            position: error.position().clone(),
            target: error.target(),
            details: error.detail().clone(),
        };

        // 更新指标
        self.metrics.record_error(&event);

        // 检查告警条件
        if self.should_alert(&event) {
            self.send_alert(&event);
        }

        // 发送到监控系统
        self.send_to_monitoring_system(&event);
    }

    fn get_error_category<E: ErrorCode>(&self, error: &StructError<E>) -> String {
        match error.reason().error_code() {
            100 => "validation".to_string(),
            101 => "business".to_string(),
            102 => "not_found".to_string(),
            103 => "permission".to_string(),
            200 => "data".to_string(),
            201 => "system".to_string(),
            202 => "network".to_string(),
            203 => "resource".to_string(),
            204 => "timeout".to_string(),
            300 => "config".to_string(),
            301 => "external".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn get_severity<E: ErrorCode>(&self, error: &StructError<E>) -> String {
        match error.reason().error_code() {
            201 | 203 | 300 => "high".to_string(), // System, Resource, Config errors
            100 | 101 | 102 | 103 => "normal".to_string(), // Business layer errors
            _ => "medium".to_string(), // Other infrastructure and external errors
        }
    }

    fn is_retryable<E: ErrorCode>(&self, error: &StructError<E>) -> bool {
        matches!(error.reason().error_code(), 201 | 202 | 203 | 204 | 301)
    }

    fn extract_context<E: ErrorCode>(&self, error: &StructError<E>) -> Vec<(String, String)> {
        error.context().iter()
            .flat_map(|ctx| ctx.context().items.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn should_alert(&self, event: &ErrorEvent) -> bool {
        // 高严重性错误总是告警
        if event.severity == "high" {
            return true;
        }

        // 检查错误率阈值
        let error_rate = self.metrics.get_error_rate(&event.error_category, Duration::from_secs(300));
        if error_rate > self.alert_thresholds.error_rate_threshold {
            return true;
        }

        // 检查连续错误次数
        let consecutive_errors = self.metrics.get_consecutive_errors(&event.error_category);
        if consecutive_errors > self.alert_thresholds.consecutive_error_threshold {
            return true;
        }

        false
    }

    fn send_alert(&self, event: &ErrorEvent) {
        log::error!(
            "🚨 HIGH SEVERITY ALERT - Error Code: {}, Category: {}, Message: {}",
            event.error_code,
            event.error_category,
            event.error_message
        );

        // 这里可以集成到实际的告警系统，如 Slack、PagerDuty 等
        // alert_service::send_alert(event).await;
    }

    fn send_to_monitoring_system(&self, event: &ErrorEvent) {
        log::info!("Tracking error event: {:?}", event);
        
        // 这里可以发送到 Prometheus、Datadog 等监控系统
        // metrics_service::record_error(event).await;
    }
}

#[derive(Debug)]
pub struct ErrorMetrics {
    error_counts: std::collections::HashMap<String, u64>,
    error_timestamps: std::collections::HashMap<String, Vec<DateTime<Utc>>>,
    consecutive_errors: std::collections::HashMap<String, u32>,
}

impl ErrorMetrics {
    fn new() -> Self {
        Self {
            error_counts: std::collections::HashMap::new(),
            error_timestamps: std::collections::HashMap::new(),
            consecutive_errors: std::collections::HashMap::new(),
        }
    }

    fn record_error(&mut self, event: &ErrorEvent) {
        // 更新错误计数
        *self.error_counts.entry(event.error_category.clone()).or_insert(0) += 1;

        // 记录错误时间戳
        self.error_timestamps
            .entry(event.error_category.clone())
            .or_insert_with(Vec::new)
            .push(event.timestamp);

        // 更新连续错误计数
        if event.severity == "high" {
            *self.consecutive_errors.entry(event.error_category.clone()).or_insert(0) += 1;
        } else {
            // 非高严重性错误重置连续计数
            self.consecutive_errors.insert(event.error_category.clone(), 0);
        }

        // 清理过期的时间戳
        self.cleanup_old_timestamps();
    }

    fn get_error_rate(&self, category: &str, duration: Duration) -> f64 {
        if let Some(timestamps) = self.error_timestamps.get(category) {
            let now = Utc::now();
            let recent_count = timestamps
                .iter()
                .filter(|&&ts| now.signed_duration_since(ts).to_std().unwrap_or_default() < duration)
                .count();
            
            recent_count as f64 / duration.as_secs_f64()
        } else {
            0.0
        }
    }

    fn get_consecutive_errors(&self, category: &str) -> u32 {
        self.consecutive_errors.get(category).copied().unwrap_or(0)
    }

    fn cleanup_old_timestamps(&mut self) {
        let now = Utc::now();
        let cutoff_duration = Duration::from_secs(3600); // 保留1小时的数据

        for timestamps in self.error_timestamps.values_mut() {
            timestamps.retain(|&ts| {
                now.signed_duration_since(ts).to_std().unwrap_or_default() < cutoff_duration
            });
        }
    }
}

#[derive(Debug)]
pub struct AlertThresholds {
    pub error_rate_threshold: f64,
    pub consecutive_error_threshold: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            error_rate_threshold: 0.1, // 每秒0.1个错误
            consecutive_error_threshold: 5, // 连续5个高严重性错误
        }
    }
}

// 使用示例
pub fn handle_api_error<E: ErrorCode + std::fmt::Display>(error: StructError<E>) -> HttpResponse {
    let mut monitor = ErrorMonitor::new();
    monitor.track_error(&error);

    // 根据错误类型返回不同的 HTTP 状态码
    match error.reason().error_code() {
        100 => HttpResponse::BadRequest().json(error.to_string()), // ValidationError
        101 => HttpResponse::Conflict().json(error.to_string()),   // BusinessError
        102 => HttpResponse::NotFound().json(error.to_string()),   // NotFoundError
        103 => HttpResponse::Unauthorized().json(error.to_string()), // PermissionError
        200 | 201 | 202 | 203 | 204 => { // Infrastructure layer errors
            HttpResponse::ServiceUnavailable().json(error.to_string())
        }
        300 | 301 => HttpResponse::InternalServerError().json(error.to_string()), // Config & External
        _ => HttpResponse::InternalServerError().json(error.to_string()),
    }
}
```

### 结构化日志记录

实现结构化日志记录，便于后续分析和监控：

```rust
use serde_json::json;

pub struct StructuredLogger {
    service_name: String,
    environment: String,
}

impl StructuredLogger {
    pub fn new(service_name: String, environment: String) -> Self {
        Self { service_name, environment }
    }

    pub fn log_error<E: ErrorCode + std::fmt::Display>(&self, error: &StructError<E>) {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": "ERROR",
            "service": self.service_name,
            "environment": self.environment,
            "error_code": error.error_code(),
            "error_category": self.get_error_category(error),
            "error_message": error.to_string(),
            "severity": self.get_severity(error),
            "retryable": self.is_retryable(error),
            "position": error.position(),
            "target": error.target(),
            "details": error.detail(),
            "context": self.extract_context(error),
            "stack_trace": self.get_stack_trace(),
        });

        // 输出 JSON 格式的日志
        println!("{}", serde_json::to_string(&log_entry).unwrap());
    }

    pub fn log_warning<E: ErrorCode + std::fmt::Display>(&self, error: &StructError<E>) {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": "WARN",
            "service": self.service_name,
            "environment": self.environment,
            "error_code": error.error_code(),
            "error_category": self.get_error_category(error),
            "error_message": error.to_string(),
            "severity": self.get_severity(error),
            "retryable": self.is_retryable(error),
        });

        println!("{}", serde_json::to_string(&log_entry).unwrap());
    }

    fn get_error_category<E: ErrorCode>(&self, error: &StructError<E>) -> String {
        match error.reason().error_code() {
            100 => "validation",
            101 => "business",
            102 => "not_found",
            103 => "permission",
            200 => "data",
            201 => "system",
            202 => "network",
            203 => "resource",
            204 => "timeout",
            300 => "config",
            301 => "external",
            _ => "unknown",
        }
    }

    fn get_severity<E: ErrorCode>(&self, error: &StructError<E>) -> String {
        match error.reason().error_code() {
            201 | 203 | 300 => "high",
            100 | 101 | 102 | 103 => "normal",
            _ => "medium",
        }
    }

    fn is_retryable<E: ErrorCode>(&self, error: &StructError<E>) -> bool {
        matches!(error.reason().error_code(), 201 | 202 | 203 | 204 | 301)
    }

    fn extract_context<E: ErrorCode>(&self, error: &StructError<E>) -> Vec<serde_json::Value> {
        error.context().iter()
            .flat_map(|ctx| ctx.context().items.iter())
            .map(|(k, v)| {
                json!({
                    "key": k,
                    "value": v
                })
            })
            .collect()
    }

    fn get_stack_trace(&self) -> String {
        // 在实际应用中，可以使用 backtrace crate 获取堆栈跟踪
        // 这里返回一个示例值
        "stack_trace_placeholder".to_string()
    }
}

// 使用示例
pub fn error_handling_with_logging() {
    let logger = StructuredLogger::new("user-service".to_string(), "production".to_string());

    let result = some_operation_that_might_fail();

    match result {
        Ok(data) => {
            println!("Operation succeeded: {:?}", data);
        }
        Err(error) => {
            if error.is_high_severity() {
                logger.log_error(&error);
            } else {
                logger.log_warning(&error);
            }
        }
    }
}
```

## 完整示例：Web 应用

让我们创建一个完整的 Web 应用示例，展示如何在实际项目中使用 orion-error。

### 项目结构

```
web-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── errors/
│   │   ├── mod.rs
│   │   ├── app_error.rs
│   │   └── user_error.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── user_service.rs
│   │   └── auth_service.rs
│   ├── repositories/
│   │   ├── mod.rs
│   │   └── user_repository.rs
│   └── routes/
│       ├── mod.rs
│       ├── user_routes.rs
│       └── auth_routes.rs
```

### 错误定义 (`errors/app_error.rs`)

```rust
use orion_error::{StructError, UvsReason, DomainReason, ErrorCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone, Serialize)]
pub enum AppError {
    #[error("user not found")]
    UserNotFound,
    #[error("invalid user data")]
    InvalidUserData,
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("insufficient permissions")]
    InsufficientPermissions,
    #[error("service temporarily unavailable")]
    ServiceUnavailable,
    #[error("{0}")]
    Universal(UvsReason),
}

impl ErrorCode for AppError {
    fn error_code(&self) -> i32 {
        match self {
            AppError::UserNotFound => 1000,
            AppError::InvalidUserData => 1001,
            AppError::EmailAlreadyExists => 1002,
            AppError::AuthenticationFailed => 1003,
            AppError::InsufficientPermissions => 1004,
            AppError::ServiceUnavailable => 1005,
            AppError::Universal(reason) => reason.error_code(),
        }
    }
}

impl DomainReason for AppError {}

impl AppError {
    pub fn http_status(&self) -> u16 {
        match self {
            AppError::UserNotFound => 404,
            AppError::InvalidUserData => 400,
            AppError::EmailAlreadyExists => 409,
            AppError::AuthenticationFailed => 401,
            AppError::InsufficientPermissions => 403,
            AppError::ServiceUnavailable => 503,
            AppError::Universal(reason) => match reason.error_code() {
                100 => 400, // ValidationError
                101 => 409, // BusinessError
                102 => 404, // NotFoundError
                103 => 403, // PermissionError
                200 => 422, // DataError
                201 => 500, // SystemError
                202 => 502, // NetworkError
                203 => 503, // ResourceError
                204 => 504, // TimeoutError
                300 => 500, // ConfigError
                301 => 502, // ExternalError
                _ => 500,
            },
        }
    }
}
```

### 用户服务 (`services/user_service.rs`)

```rust
use crate::errors::{AppError, AppResult};
use crate::repositories::UserRepository;
use orion_error::{ErrorWith, WithContext};

pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub async fn create_user(&self, email: &str, name: &str, password: &str) -> AppResult<User> {
        let mut ctx = WithContext::want("create_user");
        ctx.with("email", email);
        ctx.with("name", name);

        // 验证输入
        self.validate_user_data(email, name, password)
            .want("validate user data")
            .with(&ctx)?;

        // 检查邮箱是否已存在
        self.check_email_uniqueness(email)
            .want("check email uniqueness")
            .with(&ctx)?;

        // 创建用户
        let user = self.user_repository
            .create(email, name, password)
            .await
            .map_err(|e| e.owe_sys())? // 数据库错误转换为系统错误
            .want("create user in database")
            .with(&ctx)?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: u32) -> AppResult<User> {
        let mut ctx = WithContext::want("get_user_by_id");
        ctx.with("user_id", user_id.to_string());

        let user = self.user_repository
            .find_by_id(user_id)
            .await
            .map_err(|e| e.owe_sys())?
            .want("find user by id")
            .with(&ctx)?;

        user.ok_or_else(|| {
            StructError::from(AppError::UserNotFound)
                .want("user not found")
                .with(&ctx)
                .with_detail(format!("User with id {} does not exist", user_id))
        })
    }

    pub async fn update_user(&self, user_id: u32, update_data: UserUpdateData) -> AppResult<User> {
        let mut ctx = WithContext::want("update_user");
        ctx.with("user_id", user_id.to_string());
        ctx.with("update_fields", format!("{:?}", update_data.fields_updated()));

        // 验证用户存在
        let existing_user = self.get_user_by_id(user_id).await?;

        // 验证更新数据
        self.validate_update_data(&update_data)
            .want("validate update data")
            .with(&ctx)?;

        // 执行更新
        let updated_user = self.user_repository
            .update(user_id, update_data)
            .await
            .map_err(|e| e.owe_sys())?
            .want("update user in database")
            .with(&ctx)?;

        Ok(updated_user)
    }

    pub async fn delete_user(&self, user_id: u32, requesting_user_id: u32) -> AppResult<()> {
        let mut ctx = WithContext::want("delete_user");
        ctx.with("user_id", user_id.to_string());
        ctx.with("requesting_user_id", requesting_user_id.to_string());

        // 验证权限
        if requesting_user_id != user_id {
            return Err(StructError::from(AppError::InsufficientPermissions)
                .want("check delete permission")
                .with(&ctx)
                .with_detail("Users can only delete their own accounts"));
        }

        // 执行删除
        self.user_repository
            .delete(user_id)
            .await
            .map_err(|e| e.owe_sys())?
            .want("delete user from database")
            .with(&ctx)?;

        Ok(())
    }

    fn validate_user_data(&self, email: &str, name: &str, password: &str) -> AppResult<()> {
        if email.is_empty() || !email.contains('@') {
            return Err(StructError::from(AppError::InvalidUserData)
                .want("validate email")
                .with_detail("Email must be a valid email address"));
        }

        if name.len() < 2 || name.len() > 50 {
            return Err(StructError::from(AppError::InvalidUserData)
                .want("validate name")
                .with_detail("Name must be between 2 and 50 characters"));
        }

        if password.len() < 8 {
            return Err(StructError::from(AppError::InvalidUserData)
                .want("validate password")
                .with_detail("Password must be at least 8 characters long"));
        }

        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(StructError::from(AppError::InvalidUserData)
                .want("validate password")
                .with_detail("Password must contain at least one digit"));
        }

        if !password.chars().any(|c| c.is_ascii_alphabetic()) {
            return Err(StructError::from(AppError::InvalidUserData)
                .want("validate password")
                .with_detail("Password must contain at least one letter"));
        }

        Ok(())
    }

    async fn check_email_uniqueness(&self, email: &str) -> AppResult<()> {
        match self.user_repository.find_by_email(email).await {
            Ok(Some(_)) => Err(StructError::from(AppError::EmailAlreadyExists)
                .want("check email uniqueness")
                .with_detail(format!("Email {} is already registered", email))),
            Ok(None) => Ok(()),
            Err(e) => Err(e.owe_sys())?,
        }
    }

    fn validate_update_data(&self, update_data: &UserUpdateData) -> AppResult<()> {
        if let Some(email) = &update_data.email {
            if email.is_empty() || !email.contains('@') {
                return Err(StructError::from(AppError::InvalidUserData)
                    .want("validate email in update")
                    .with_detail("Email must be a valid email address"));
            }
        }

        if let Some(name) = &update_data.name {
            if name.len() < 2 || name.len() > 50 {
                return Err(StructError::from(AppError::InvalidUserData)
                    .want("validate name in update")
                    .with_detail("Name must be between 2 and 50 characters"));
            }
        }

        Ok(())
    }
}

// 辅助类型定义
pub type AppResult<T> = Result<T, StructError<AppError>>;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct UserUpdateData {
    pub email: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
}

impl UserUpdateData {
    pub fn fields_updated(&self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        if self.email.is_some() {
            fields.push("email");
        }
        if self.name.is_some() {
            fields.push("name");
        }
        if self.password.is_some() {
            fields.push("password");
        }
        fields
    }
}
```

### 路由处理 (`routes/user_routes.rs`)

```rust
use crate::errors::AppError;
use crate::services::UserService;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: u32,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

pub fn user_routes(
    user_service: UserService,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    create_user(user_service.clone())
        .or(get_user(user_service.clone()))
        .or(update_user(user_service.clone()))
        .or(delete_user(user_service))
}

fn with_user_service(
    user_service: UserService,
) -> impl Filter<Extract = (UserService,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || user_service.clone())
}

fn create_user(
    user_service: UserService,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("users")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_user_service(user_service))
        .and_then(|request: CreateUserRequest, service: UserService| async move {
            handle_create_user(request, service).await
        })
}

async fn handle_create_user(
    request: CreateUserRequest,
    service: UserService,
) -> Result<impl Reply, Rejection> {
    let mut ctx = WithContext::want("create_user_endpoint");
    ctx.with("endpoint", "POST /users");
    ctx.with("client_ip", "192.168.1.100"); // 在实际应用中从请求头获取

    match service.create_user(&request.email, &request.name, &request.password).await {
        Ok(user) => {
            let response = UserResponse::from(user);
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(error) => {
            log_error(&error);
            let status = warp::http::StatusCode::from_u16(error.reason().http_status())
                .unwrap_or(warp::http::StatusCode::INTERNAL_SERVER_ERROR);
            
            let error_response = ErrorResponse {
                error_code: error.error_code(),
                message: error.to_string(),
                details: error.detail().clone(),
                context: error.context().first().map(|ctx| ctx.target().clone()),
            };
            
            Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                status,
            ))
        }
    }
}

fn get_user(
    user_service: UserService,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("users" / u32)
        .and(warp::get())
        .and(with_user_service(user_service))
        .and_then(|user_id: u32, service: UserService| async move {
            handle_get_user(user_id, service).await
        })
}

async fn handle_get_user(
    user_id: u32,
    service: UserService,
) -> Result<impl Reply, Rejection> {
    let mut ctx = WithContext::want("get_user_endpoint");
    ctx.with("endpoint", "GET /users/{id}");
    ctx.with("user_id", user_id.to_string());

    match service.get_user_by_id(user_id).await {
        Ok(user) => {
            let response = UserResponse::from(user);
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        }
        Err(error) => {
            log_error(&error);
            let status = warp::http::StatusCode::from_u16(error.reason().http_status())
                .unwrap_or(warp::http::StatusCode::INTERNAL_SERVER_ERROR);
            
            let error_response = ErrorResponse {
                error_code: error.error_code(),
                message: error.to_string(),
                details: error.detail().clone(),
                context: error.context().first().map(|ctx| ctx.target().clone()),
            };
            
            Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                status,
            ))
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error_code: i32,
    message: String,
    details: Option<String>,
    context: Option<String>,
}

fn log_error<E: ErrorCode + std::fmt::Display>(error: &StructError<E>) {
    if error.is_high_severity() {
        log::error!("High severity error: {}", error);
    } else {
        log::warn!("Error occurred: {}", error);
    }
}
```

### 主应用程序 (`main.rs`)

```rust
mod errors;
mod services;
mod repositories;
mod routes;

use errors::AppError;
use repositories::UserRepository;
use routes::user_routes;
use services::UserService;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    log::info!("Starting web application");

    // 创建依赖
    let user_repository = UserRepository::new().await;
    let user_service = UserService::new(user_repository);

    // 设置路由
    let user_routes = user_routes(user_service);

    // 健康检查路由
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            log::info!("Health check requested");
            warp::reply::with_status("OK", warp::http::StatusCode::OK)
        });

    // 组合所有路由
    let routes = health
        .or(user_routes)
        .with(warp::log("web_app"));

    // 启动服务器
    log::info!("Server starting on port 8080");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
```

## 最佳实践

### 1. 错误类型选择原则

根据错误的具体性质选择合适的错误类型：

```rust
// ✅ 正确的错误类型选择
fn validate_email(email: &str) -> Result<(), StructError<UvsReason>> {
    if !email.contains('@') {
        return Err(StructError::from(
            UvsReason::validation_error("email format invalid")
        ));
    }
    Ok(())
}

fn check_user_balance(user_id: u32, amount: f64) -> Result<(), StructError<UvsReason>> {
    let balance = get_balance(user_id)?;
    if balance < amount {
        return Err(StructError::from(
            UvsReason::business_error("insufficient balance")
        ));
    }
    Ok(())
}

fn find_user(user_id: u32) -> Result<User, StructError<UvsReason>> {
    let user = database::find_user_by_id(user_id)?;
    user.ok_or_else(|| {
        StructError::from(UvsReason::not_found_error("user not found"))
    })
}

fn check_permission(user_id: u32, resource: &str) -> Result<(), StructError<UvsReason>> {
    if !has_permission(user_id, resource) {
        return Err(StructError::from(
            UvsReason::permission_error("access denied")
        ));
    }
    Ok(())
}
```

### 2. 上下文信息管理

提供丰富但相关的上下文信息：

```rust
// ✅ 好的上下文管理
fn process_order(order_id: u32, user_id: u32) -> Result<Order, StructError<OrderError>> {
    let mut ctx = WithContext::want("process_order");
    ctx.with("order_id", order_id.to_string());
    ctx.with("user_id", user_id.to_string());
    ctx.with("timestamp", chrono::Utc::now().to_rfc3339());
    
    // 根据环境添加不同的上下文
    if cfg!(debug_assertions) {
        ctx.with("environment", "development");
    } else {
        ctx.with("environment", "production");
    }

    // 处理订单逻辑
    let order = validate_order(order_id)
        .want("validate order")
        .with(&ctx)?;

    let processed = apply_business_logic(&order, user_id)
        .want("apply business logic")
        .with(&ctx)?;

    save_order(&processed)
        .want("save order")
        .with(&ctx)
}

// ❌ 过多的上下文信息
fn bad_context_management() -> Result<(), StructError<MyError>> {
    let mut ctx = WithContext::want("operation");
    // 添加过多不相关的信息
    ctx.with("server_ip", "192.168.1.100");
    ctx.with("server_hostname", "web-server-01");
    ctx.with("process_id", "12345");
    ctx.with("thread_id", "67890");
    ctx.with("memory_usage", "256MB");
    ctx.with("cpu_usage", "15%");
    ctx.with("disk_usage", "45%");
    ctx.with("network_traffic", "1MB/s");
    // ... 过多的信息会让错误日志变得混乱
}
```

### 3. 错误恢复策略

根据错误类型实施不同的恢复策略：

```rust
use orion_error::UvsReason;

pub enum RecoveryStrategy {
    /// 立即重试 - 适用于暂时性网络问题
    ImmediateRetry,
    /// 指数退避重试 - 适用于外部服务超时
    ExponentialBackoff { max_attempts: u32 },
    /// 降级处理 - 适用于服务不可用
    Fallback { fallback_service: String },
    /// 快速失败 - 适用于业务逻辑错误
    FailFast,
    /// 手动干预 - 适用于配置和系统错误
    ManualIntervention,
}

impl RecoveryStrategy {
    pub fn for_error(error: &UvsReason) -> Self {
        match error.error_code() {
            // 网络超时 - 指数退避重试
            202 => RecoveryStrategy::ExponentialBackoff { max_attempts: 3 },
            
            // 外部服务错误 - 降级处理
            301 => RecoveryStrategy::Fallback { 
                fallback_service: "cache_service".to_string() 
            },
            
            // 系统资源不足 - 手动干预
            203 => RecoveryStrategy::ManualIntervention,
            
            // 配置错误 - 手动干预
            300 => RecoveryStrategy::ManualIntervention,
            
            // 业务逻辑错误 - 快速失败
            101 => RecoveryStrategy::FailFast,
            
            // 验证错误 - 快速失败
            100 => RecoveryStrategy::FailFast,
            
            // 默认 - 立即重试
            _ => RecoveryStrategy::ImmediateRetry,
        }
    }
}

pub async fn robust_operation<F, T>(operation: F) -> Result<T, StructError<UvsReason>>
where
    F: Fn() -> Result<T, StructError<UvsReason>>,
{
    match operation() {
        Ok(result) => Ok(result),
        Err(error) => {
            let strategy = RecoveryStrategy::for_error(error.reason());
            
            match strategy {
                RecoveryStrategy::ImmediateRetry => {
                    log::info!("Immediate retry for error: {}", error);
                    // 立即重试一次
                    operation()
                }
                RecoveryStrategy::ExponentialBackoff { max_attempts } => {
                    log::info!("Exponential backoff retry for error: {}", error);
                    // 实现指数退避重试逻辑
                    retry_with_backoff(operation, max_attempts).await
                }
                RecoveryStrategy::Fallback { fallback_service } => {
                    log::warn!("Using fallback service {} for error: {}", fallback_service, error);
                    // 调用降级服务
                    call_fallback_service(fallback_service).await
                }
                RecoveryStrategy::FailFast => {
                    log::error!("Fail fast for error: {}", error);
                    Err(error)
                }
                RecoveryStrategy::ManualIntervention => {
                    log::error!("Manual intervention required for error: {}", error);
                    // 发送告警并返回特定错误
                    send_alert(&error).await;
                    Err(StructError::from(
                        UvsReason::system_error("service unavailable - manual intervention required")
                    ))
                }
            }
        }
    }
}
```

### 4. 监控和告警配置

基于错误严重性设置不同的监控策略：

```rust
use orion_error::UvsReason;

pub struct MonitoringConfig {
    pub alert_channels: AlertChannels,
    pub metrics_config: MetricsConfig,
    pub log_levels: LogLevelConfig,
}

#[derive(Debug)]
pub struct AlertChannels {
    pub email: bool,
    pub slack: bool,
    pub pager_duty: bool,
}

#[derive(Debug)]
pub struct MetricsConfig {
    pub enable_histogram: bool,
    pub enable_counter: bool,
    pub buckets: Vec<f64>,
}

#[derive(Debug)]
pub struct LogLevelConfig {
    pub business_errors: log::Level,
    pub infrastructure_errors: log::Level,
    pub configuration_errors: log::Level,
}

impl MonitoringConfig {
    pub fn for_error(error: &UvsReason) -> Self {
        match error.error_code() {
            // 系统和资源错误 - 高优先级告警
            201 | 203 => Self {
                alert_channels: AlertChannels {
                    email: true,
                    slack: true,
                    pager_duty: true,
                },
                metrics_config: MetricsConfig {
                    enable_histogram: true,
                    enable_counter: true,
                    buckets: vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
                },
                log_levels: LogLevelConfig {
                    business_errors: log::Level::Error,
                    infrastructure_errors: log::Level::Error,
                    configuration_errors: log::Level::Error,
                },
            },
            
            // 网络和超时错误 - 中等优先级告警
            202 | 204 => Self {
                alert_channels: AlertChannels {
                    email: false,
                    slack: true,
                    pager_duty: false,
                },
                metrics_config: MetricsConfig {
                    enable_histogram: true,
                    enable_counter: true,
                    buckets: vec![0.5, 1.0, 2.0, 5.0, 10.0, 30.0],
                },
                log_levels: LogLevelConfig {
                    business_errors: log::Level::Warn,
                    infrastructure_errors: log::Level::Warn,
                    configuration_errors: log::Level::Warn,
                },
            },
            
            // 业务逻辑错误 - 正常监控
            100 | 101 | 102 | 103 => Self {
                alert_channels: AlertChannels {
                    email: false,
                    slack: false,
                    pager_duty: false,
                },
                metrics_config: MetricsConfig {
                    enable_histogram: false,
                    enable_counter: true,
                    buckets: vec![],
                },
                log_levels: LogLevelConfig {
                    business_errors: log::Level::Info,
                    infrastructure_errors: log::Level::Info,
                    configuration_errors: log::Level::Info,
                },
            },
            
            // 配置和外部错误 - 低优先级告警
            300 | 301 => Self {
                alert_channels: AlertChannels {
                    email: true,
                    slack: true,
                    pager_duty: false,
                },
                metrics_config: MetricsConfig {
                    enable_histogram: false,
                    enable_counter: true,
                    buckets: vec![],
                },
                log_levels: LogLevelConfig {
                    business_errors: log::Level::Warn,
                    infrastructure_errors: log::Level::Warn,
                    configuration_errors: log::Level::Error,
                },
            },
            
            // 数据错误 - 正常监控
            200 => Self {
                alert_channels: AlertChannels {
                    email: false,
                    slack: false,
                    pager_duty: false,
                },
                metrics_config: MetricsConfig {
                    enable_histogram: false,
                    enable_counter: true,
                    buckets: vec![],
                },
                log_levels: LogLevelConfig {
                    business_errors: log::Level::Warn,
                    infrastructure_errors: log::Level::Warn,
                    configuration_errors: log::Level::Warn,
                },
            },
        }
    }
}
```

### 5. 测试策略

为错误处理编写全面的测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        // 测试错误码分配
        let validation_error = UvsReason::validation_error("invalid input");
        assert_eq!(validation_error.error_code(), 100);

        let business_error = UvsReason::business_error("business rule violation");
        assert_eq!(business_error.error_code(), 101);

        let not_found_error = UvsReason::not_found_error("resource not found");
        assert_eq!(not_found_error.error_code(), 102);
    }

    #[test]
    fn test_retryable_errors() {
        let retryable_errors = vec![
            UvsReason::network_error("connection failed"),
            UvsReason::timeout_error("operation timeout"),
            UvsReason::system_error("disk full"),
            UvsReason::resource_error("memory exhausted"),
            UvsReason::external_error("api down"),
        ];

        for error in retryable_errors {
            assert!(error.is_retryable(), "{} should be retryable", error);
        }

        let non_retryable_errors = vec![
            UvsReason::validation_error("invalid input"),
            UvsReason::business_error("insufficient funds"),
            UvsReason::not_found_error("user not found"),
            UvsReason::permission_error("access denied"),
            UvsReason::config_error(ConfErrReason::Core("missing config".to_string())),
        ];

        for error in non_retryable_errors {
            assert!(!error.is_retryable(), "{} should not be retryable", error);
        }
    }

    #[test]
    fn test_high_severity_errors() {
        let high_severity_errors = vec![
            UvsReason::system_error("critical system failure"),
            UvsReason::resource_error("out of memory"),
            UvsReason::config_error(ConfErrReason::Core("configuration missing".to_string())),
        ];

        for error in high_severity_errors {
            assert!(error.is_high_severity(), "{} should be high severity", error);
        }

        let normal_severity_errors = vec![
            UvsReason::validation_error("invalid input"),
            UvsReason::business_error("business error"),
            UvsReason::network_error("connection failed"),
        ];

        for error in normal_severity_errors {
            assert!(!error.is_high_severity(), "{} should not be high severity", error);
        }
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(UvsReason::validation_error("test").category_name(), "validation");
        assert_eq!(UvsReason::business_error("test").category_name(), "business");
        assert_eq!(UvsReason::network_error("test").category_name(), "network");
        assert_eq!(UvsReason::system_error("test").category_name(), "system");
        assert_eq!(UvsReason::external_error("test").category_name(), "external");
    }

    #[tokio::test]
    async fn test_error_context_propagation() {
        let result = async_operation_with_context().await;
        
        match result {
            Ok(_) => panic!("Expected error, got success"),
            Err(error) => {
                assert!(!error.context().is_empty());
                assert!(error.context().len() >= 2); // 至少有两层上下文
                
                // 验证上下文内容
                let first_ctx = error.context().first().unwrap();
                assert_eq!(first_ctx.target(), Some("operation".to_string()));
                
                let second_ctx = error.context().get(1).unwrap();
                assert_eq!(second_ctx.target(), Some("sub_operation".to_string()));
            }
        }
    }

    #[test]
    fn test_error_conversion() {
        // 测试从其他错误类型的转换
        let io_error: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "connection refused"
        ));
        
        let converted_error = io_error.owe_sys().unwrap_err();
        assert_eq!(converted_error.error_code(), 201); // SystemError
        assert!(converted_error.to_string().contains("connection refused"));
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();
        
        let operation = || {
            attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if attempt_count.load(std::sync::atomic::Ordering::SeqCst) < 3 {
                Err(StructError::from(UvsReason::network_error("temporary failure")))
            } else {
                Ok("success".to_string())
            }
        };
        
        let result = smart_retry(operation, 5).await;
        assert!(result.is_ok());
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    async fn async_operation_with_context() -> Result<String, StructError<UvsReason>> {
        let mut ctx = WithContext::want("operation");
        ctx.with("param", "value");
        
        sub_operation().want("sub_operation").with(&ctx)
    }

    async fn sub_operation() -> Result<String, StructError<UvsReason>> {
        let mut ctx = WithContext::want("sub_operation");
        ctx.with("detail", "additional_info");
        
        Err(StructError::from(UvsReason::business_error("operation failed"))
            .want("sub_operation")
            .with(&ctx))
    }
}
```

## 总结

本教程详细介绍了 orion-error 库的使用方法，从基础概念到高级应用。通过合理的错误分类、丰富的上下文信息和智能的重试机制，您可以构建出更加健壮和可维护的应用程序。

关键要点：

1. **选择合适的错误类型** - 根据错误性质选择 ValidationError、BusinessError、NotFoundError 等
2. **提供丰富的上下文** - 使用 WithContext 为错误添加相关的环境信息
3. **实现智能重试** - 基于 `is_retryable()` 和错误类型实施不同的重试策略
4. **集成监控告警** - 根据错误严重性设置不同的监控级别
5. **编写全面测试** - 为错误处理逻辑编写单元测试和集成测试

通过遵循这些最佳实践，您可以构建出具有出色错误处理能力的生产级应用程序。
