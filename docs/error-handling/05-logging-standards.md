# 错误日志规范

## 概述

错误日志是错误处理系统的重要组成部分，标准化的日志格式和字段定义能够确保错误信息的一致性、可读性和可分析性。本规范定义了系统错误日志的标准格式、字段定义和最佳实践。

## 日志字段标准

### 核心字段定义

#### 基础信息字段
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLog {
    // ===== 基础标识信息 =====
    pub trace_id: String,           // 全链路追踪ID
    pub span_id: String,            // 当前调用段ID
    pub request_id: String,         // 请求唯一标识
    
    // ===== 时间信息 =====
    pub timestamp: DateTime<Utc>,   // 错误发生时间 (ISO 8601格式)
    pub duration_ms: Option<u64>,   // 操作持续时间(毫秒)
    
    // ===== 错误分类信息 =====
    pub error_code: String,         // 错误代码 (格式: DOMAIN.TYPE.SUBTYPE.CODE)
    pub error_type: ErrorType,      // 错误类型枚举
    pub severity: LogLevel,         // 日志级别
    pub category: ErrorCategory,    // 错误分类
    
    // ===== 错误内容信息 =====
    pub message: String,            // 用户友好的错误信息
    pub detail: String,             // 技术详细信息
    pub stack_trace: Option<String>, // 调用堆栈信息
    pub root_cause: Option<String>, // 根本原因分析
    
    // ===== 上下文信息 =====
    pub context: serde_json::Value, // 业务上下文数据
    pub user_context: UserContext,  // 用户相关上下文
    pub system_context: SystemContext, // 系统相关上下文
    
    // ===== 环境信息 =====
    pub environment: String,        // 运行环境 (dev/test/prod)
    pub service_name: String,       // 服务名称
    pub service_version: String,    // 服务版本
    pub host_name: String,          // 主机名称
    pub pod_name: Option<String>,   // Kubernetes Pod名称
    
    // ===== 处理信息 =====
    pub retry_count: Option<u32>,   // 重试次数
    pub handled: bool,              // 是否已处理
    pub recovery_action: Option<String>, // 恢复动作
}
```

#### 错误类型枚举
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    // 业务错误
    Validation,                     // 数据验证失败
    BusinessRule,                   // 业务规则违反
    InsufficientResources,          // 资源不足
    NotFound,                       // 资源不存在
    
    // 系统错误
    SystemInternal,                 // 系统内部错误
    Timeout,                        // 超时错误
    ResourceExhausted,              // 资源耗尽
    
    // 依赖错误
    Database,                       // 数据库相关错误
    ExternalService,                // 外部服务错误
    Network,                        // 网络相关错误
    
    // 逻辑错误
    Programming,                    // 编程逻辑错误
    Configuration,                  // 配置错误
    Security,                       // 安全相关错误
}
```

#### 错误分类枚举
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    // 按影响范围分类
    UserFacing,                     // 面向用户的错误
    Internal,                       // 内部系统错误
    Dependency,                     // 依赖服务错误
    
    // 按处理方式分类
    Recoverable,                    // 可恢复错误
    NonRecoverable,                 // 不可恢复错误
    
    // 按严重程度分类
    Critical,                       // 严重错误
    Major,                          // 主要错误
    Minor,                          // 次要错误
}
```

#### 日志级别枚举
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,                          // 错误级别，需要立即处理
    Warn,                           // 警告级别，需要注意
    Info,                           // 信息级别，正常流程信息
    Debug,                          // 调试级别，开发调试信息
    Trace,                          // 追踪级别，详细执行信息
}
```

#### 用户上下文
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Option<String>,        // 用户ID
    pub user_type: Option<String>,      // 用户类型
    pub user_role: Option<String>,      // 用户角色
    pub session_id: Option<String>,     // 会话ID
    pub client_ip: Option<String>,      // 客户端IP
    pub user_agent: Option<String>,     // 用户代理
    pub tenant_id: Option<String>,      // 租户ID
    pub department: Option<String>,     // 部门信息
}
```

#### 系统上下文
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub thread_id: String,              // 线程ID
    pub thread_name: Option<String>,    // 线程名称
    pub coroutine_id: Option<String>,   // 协程ID
    pub memory_usage: Option<u64>,      // 内存使用量(字节)
    pub cpu_usage: Option<f64>,         // CPU使用率
    pub disk_usage: Option<u64>,        // 磁盘使用量(字节)
    pub network_io: Option<NetworkIo>,  // 网络IO信息
    pub database_connections: Option<u32>, // 数据库连接数
}
```

## 日志格式规范

### JSON 格式标准

#### 标准格式
```json
{
  "@timestamp": "2024-01-15T10:30:45.123Z",
  "@version": "1",
  "message": "订单创建失败：余额不足",
  "level": "ERROR",
  "logger_name": "com.example.order.OrderService",
  "thread_name": "order-service-thread-1",
  "error": {
    "trace_id": "abc123-def456-ghi789",
    "span_id": "span001",
    "request_id": "req-001",
    "error_code": "ORDER.BUSINESS.INSUFFICIENT_BALANCE",
    "error_type": "BusinessRule",
    "severity": "Error",
    "category": "UserFacing",
    "detail": "用户 user_123 创建订单时余额不足，需要 100.00，可用 50.00",
    "root_cause": "数据库查询显示用户账户余额不足",
    "stack_trace": "com.example.order.OrderService.createOrder(OrderService.java:45)",
    "retry_count": 0,
    "handled": true,
    "recovery_action": "返回错误信息给用户"
  },
  "context": {
    "order_id": "order_001",
    "user_id": "user_123",
    "order_amount": 100.00,
    "currency": "CNY",
    "payment_method": "credit_card"
  },
  "user_context": {
    "user_id": "user_123",
    "user_type": "customer",
    "client_ip": "192.168.1.100",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
  },
  "system_context": {
    "thread_id": "47",
    "memory_usage": 104857600,
    "cpu_usage": 25.5,
    "database_connections": 5
  },
  "environment": {
    "service_name": "order-service",
    "service_version": "1.2.3",
    "environment": "production",
    "host_name": "order-service-pod-001",
    "region": "us-west-2"
  },
  "tags": ["order", "payment", "business"],
  "metrics": {
    "processing_time_ms": 45,
    "database_query_time_ms": 12
  }
}
```

#### 简化格式 (针对高频日志)
```json
{
  "@timestamp": "2024-01-15T10:30:45.123Z",
  "level": "ERROR",
  "trace_id": "abc123-def456-ghi789",
  "error_code": "ORDER.BUSINESS.INSUFFICIENT_BALANCE",
  "message": "订单创建失败：余额不足",
  "user_id": "user_123",
  "order_id": "order_001",
  "service": "order-service",
  "environment": "production"
}
```

### 文本格式标准

#### 结构化文本格式
```log
[2024-01-15T10:30:45.123Z] [ERROR] [order-service] [trace_id=abc123-def456-ghi789] [request_id=req-001] 
[error_code=ORDER.BUSINESS.INSUFFICIENT_BALANCE] [user_id=user_123] [order_id=order_001] 
订单创建失败：余额不足 | detail: 用户 user_123 创建订单时余额不足，需要 100.00，可用 50.00 | 
context: {\"order_amount\":100.00,\"currency\":\"CNY\"} | 
system: {\"memory_usage\":104857600,\"cpu_usage\":25.5}
```

#### 简化文本格式
```log
2024-01-15T10:30:45.123Z ERROR order-service trace_id=abc123-def456-ghi789 error_code=ORDER.BUSINESS.INSUFFICIENT_BALANCE message=订单创建失败：余额不足 user_id=user_123
```

## 日志级别使用指南

### ERROR 级别
**使用场景：**
- 系统无法继续执行的严重错误
- 业务流程完全失败的错误
- 数据一致性问题的错误
- 安全相关的严重错误

**示例：**
```rust
// 数据库连接失败，无法继续处理
log::error!(
    error_code = "DATABASE.CONNECTION.FAILED",
    trace_id = %trace_id,
    service = "order-service",
    "无法连接到数据库，服务不可用"
);

// 支付处理失败，影响核心业务
log::error!(
    error_code = "PAYMENT.PROCESSING.FAILED",
    trace_id = %trace_id,
    order_id = %order_id,
    "支付处理失败，订单创建中止"
);
```

### WARN 级别
**使用场景：**
- 系统可以继续执行但需要注意的问题
- 性能降级的情况
- 即将达到资源限制的警告
- 业务规则接近边界的情况

**示例：**
```rust
// 缓存命中率低，可能影响性能
log::warn!(
    error_code = "CACHE.LOW_HIT_RATE",
    trace_id = %trace_id,
    cache_name = "user_cache",
    hit_rate = 0.45,
    "缓存命中率过低，建议检查缓存策略"
);

// 数据库连接池接近满载
log::warn!(
    error_code = "DATABASE.POOL.HIGH_USAGE",
    trace_id = %trace_id,
    pool_usage = 0.85,
    max_connections = 100,
    "数据库连接池使用率过高"
);
```

### INFO 级别
**使用场景：**
- 正常业务流程的重要节点
- 系统状态变更
- 关键操作的成功执行
- 业务规则处理的决策点

**示例：**
```rust
// 订单成功创建
log::info!(
    error_code = "ORDER.CREATED",
    trace_id = %trace_id,
    order_id = %order_id,
    user_id = %user_id,
    amount = %amount,
    "订单创建成功"
);

// 用户登录成功
log::info!(
    error_code = "USER.LOGIN.SUCCESS",
    trace_id = %trace_id,
    user_id = %user_id,
    client_ip = %client_ip,
    "用户登录成功"
);
```

### DEBUG 级别
**使用场景：**
- 详细的处理过程信息
- 中间计算结果
- 条件分支的执行路径
- 数据处理步骤

**示例：**
```rust
// 数据验证过程
log::debug!(
    error_code = "VALIDATION.PROCESSING",
    trace_id = %trace_id,
    field = "email",
    value = %email,
    validation_result = "valid",
    "字段验证完成"
);

// 数据库查询详情
log::debug!(
    error_code = "DATABASE.QUERY.EXECUTED",
    trace_id = %trace_id,
    query = %query_sql,
    execution_time_ms = 15,
    rows_affected = 1,
    "数据库查询执行完成"
);
```

### TRACE 级别
**使用场景：**
- 函数调用的详细轨迹
- 复杂算法的执行步骤
- 网络请求的完整生命周期
- 并发操作的时序信息

**示例：**
```rust
// 函数调用轨迹
log::trace!(
    error_code = "FUNCTION.CALL.ENTER",
    trace_id = %trace_id,
    function = "process_order",
    parameters = %serde_json::to_string(&params).unwrap_or_default(),
    "进入函数处理"
);

// 网络请求详情
log::trace!(
    error_code = "HTTP.REQUEST.COMPLETE",
    trace_id = %trace_id,
    method = "POST",
    url = %url,
    request_size = 256,
    response_size = 1024,
    duration_ms = 45,
    status_code = 200,
    "HTTP请求完成"
);
```

## 结构化日志实现

### 日志记录器配置
```rust
use serde_json::json;
use tracing::{error, warn, info, debug, trace, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn setup_logging() {
    // 标准输出层
    let stdout_layer = tracing_subscriber::fmt::layer()
        .json() // JSON格式输出
        .with_current_span(true)
        .with_span_list(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true);

    // 文件输出层
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(std::io::BufWriter::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs/application.log")
                .expect("Failed to open log file"),
        ));

    // 错误专用文件层
    let error_file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(std::io::BufWriter::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs/error.log")
                .expect("Failed to open error log file"),
        ))
        .with_filter(tracing_subscriber::filter::LevelFilter::ERROR);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(error_file_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
```

### 错误日志宏封装
```rust
#[macro_export]
macro_rules! log_error {
    (error_code = $error_code:expr, $($field:tt = $value:expr),* $(,)?) => {
        {
            tracing::error!(
                error_code = $error_code,
                $($field = $value,)*
                error_type = tracing::field::Empty,
                severity = "ERROR",
                handled = false,
                recovery_action = tracing::field::Empty,
            );
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    (error_code = $error_code:expr, $($field:tt = $value:expr),* $(,)?) => {
        {
            tracing::warn!(
                error_code = $error_code,
                $($field = $value,)*
                error_type = tracing::field::Empty,
                severity = "WARN",
                handled = false,
                recovery_action = tracing::field::Empty,
            );
        }
    };
}

#[macro_export]
macro_rules! log_info {
    (error_code = $error_code:expr, $($field:tt = $value:expr),* $(,)?) => {
        {
            tracing::info!(
                error_code = $error_code,
                $($field = $value,)*
                error_type = tracing::field::Empty,
                severity = "INFO",
                handled = false,
                recovery_action = tracing::field::Empty,
            );
        }
    };
}
```

### 上下文追踪实现
```rust
use tracing::{Span, span, Level};

pub struct ErrorContext {
    pub trace_id: String,
    pub span_id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub service_name: String,
    pub environment: String,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            request_id: generate_request_id(),
            user_id: None,
            service_name: env!("CARGO_PKG_NAME").to_string(),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        }
    }
    
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn create_span(&self, operation: &str) -> Span {
        span!(
            Level::INFO,
            "operation",
            trace_id = %self.trace_id,
            span_id = %self.span_id,
            request_id = %self.request_id,
            operation = operation,
            service_name = %self.service_name,
            environment = %self.environment,
            user_id = ?self.user_id,
        )
    }
}

// 使用示例
pub fn process_order(ctx: &ErrorContext, order_request: &OrderRequest) -> Result<OrderResponse, Error> {
    let span = ctx.create_span("process_order");
    let _guard = span.enter();
    
    log_info!(
        error_code = "ORDER.PROCESS.START",
        order_id = %order_request.id,
        user_id = %order_request.user_id,
        "开始处理订单"
    );
    
    // 业务逻辑处理
    match validate_order(order_request) {
        Ok(_) => {
            log_info!(
                error_code = "ORDER.PROCESS.SUCCESS",
                order_id = %order_request.id,
                "订单处理成功"
            );
            Ok(create_order_response(order_request))
        }
        Err(e) => {
            log_error!(
                error_code = "ORDER.PROCESS.FAILED",
                order_id = %order_request.id,
                error_detail = %e,
                "订单处理失败"
            );
            Err(e)
        }
    }
}
```

## 日志关联与追踪

### 全链路追踪实现
```rust
use opentelemetry::{
    trace::{TraceContextExt, TraceId, SpanId},
    Context,
};

pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub baggage: std::collections::HashMap<String, String>,
}

impl TraceContext {
    pub fn current() -> Self {
        let context = Context::current();
        let span_context = context.span().span_context();
        
        Self {
            trace_id: span_context.trace_id(),
            span_id: span_context.span_id(),
            parent_span_id: None, // 父级Span需要从Context中获取
            baggage: std::collections::HashMap::new(),
        }
    }
    
    pub fn from_headers(headers: &http::HeaderMap) -> Option<Self> {
        // 从HTTP头中提取追踪信息
        headers.get("traceparent")
            .and_then(|value| value.to_str().ok())
            .and_then(|trace_parent| Self::from_traceparent(trace_parent))
    }
    
    fn from_traceparent(trace_parent: &str) -> Option<Self> {
        // 解析W3C Trace-Parent格式
        let parts: Vec<&str> = trace_parent.split('-').collect();
        if parts.len() != 4 {
            return None;
        }
        
        let trace_id = TraceId::from_hex(parts[1]).ok()?;
        let span_id = SpanId::from_hex(parts[2]).ok()?;
        
        Some(Self {
            trace_id,
            span_id,
            parent_span_id: None,
            baggage: std::collections::HashMap::new(),
        })
    }
    
    pub fn to_traceparent(&self) -> String {
        format!("00-{:032x}-{:016x}-{:02x}", 
            self.trace_id, 
            self.span_id, 
            0x01 // sampled
        )
    }
    
    pub fn inject_to_headers(&self, headers: &mut http::HeaderMap) {
        headers.insert("traceparent", 
            self.to_traceparent().parse().unwrap());
    }
}
```

### 日志关联查询实现
```rust
use serde_json::Value;
use std::collections::HashMap;

pub struct LogCorrelator {
    pub trace_id: String,
    pub related_logs: Vec<Value>,
    pub error_chain: Vec<String>,
}

impl LogCorrelator {
    pub fn new(trace_id: String) -> Self {
        Self {
            trace_id,
            related_logs: Vec::new(),
            error_chain: Vec::new(),
        }
    }
    
    pub fn add_log(&mut self, log_entry: Value) {
        if let Some(log_trace_id) = log_entry.get("trace_id")
            .and_then(|v| v.as_str()) 
        {
            if log_trace_id == self.trace_id {
                self.related_logs.push(log_entry);
            }
        }
    }
    
    pub fn build_error_chain(&mut self) {
        // 按时间戳排序日志
        self.related_logs.sort_by(|a, b| {
            let time_a = a.get("@timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let time_b = b.get("@timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            time_a.cmp(time_b)
        });
        
        // 构建错误链
        for log in &self.related_logs {
            if let Some(error_code) = log.get("error_code")
                .and_then(|v| v.as_str()) 
            {
                if error_code.starts_with("ERROR") || 
                   log.get("level")
                    .and_then(|v| v.as_str()) 
                    .map(|level| level == "ERROR")
                    .unwrap_or(false) 
                {
                    self.error_chain.push(error_code.to_string());
                }
            }
        }
    }
    
    pub fn get_correlation_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for log in &self.related_logs {
            if let Some(service) = log.get("service")
                .and_then(|v| v.as_str()) 
            {
                *summary.entry(service.to_string()).or_insert(0) += 1;
            }
        }
        
        summary
    }
}
```

## 性能考虑

### 日志性能优化策略
```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::Mutex;

pub struct LogBuffer {
    buffer: Arc<Mutex<Vec<String>>>,
    max_size: usize,
    flush_threshold: usize,
    counter: Arc<AtomicUsize>,
}

impl LogBuffer {
    pub fn new(max_size: usize, flush_threshold: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
            flush_threshold,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    pub fn write(&self, message: String) -> Result<(), String> {
        let current_count = self.counter.fetch_add(1, Ordering::Relaxed);
        
        if current_count >= self.flush_threshold {
            // 触发异步刷新
            self.trigger_async_flush();
        }
        
        let mut buffer = self.buffer.lock();
        if buffer.len() >= self.max_size {
            buffer.clear();
        }
        
        buffer.push(message);
        Ok(())
    }
    
    fn trigger_async_flush(&self) {
        let buffer_clone = self.buffer.clone();
        let counter_clone = self.counter.clone();
        
        tokio::spawn(async move {
            let messages: Vec<String> = {
                let mut buffer = buffer_clone.lock();
                buffer.drain(..).collect()
            };
            
            if !messages.is_empty() {
                if let Err(e) = Self::flush_to_disk(&messages).await {
                    eprintln!("Failed to flush logs: {}", e);
                }
            }
            
            counter_clone.store(0, Ordering::Relaxed);
        });
    }
    
    async fn flush_to_disk(messages: &[String]) -> std::io::Result<()> {
        // 实现日志写入磁盘的逻辑
        Ok(())
    }
}
```

### 异步日志配置
```rust
use tokio::sync::mpsc;
use tracing_appender::{non_blocking, rolling};

pub fn setup_async_logging() {
    // 设置异步文件输出
    let file_appender = rolling::daily("logs", "application.log");
    let (non_blocking, _guard) = non_blocking(file_appender);
    
    // 设置异步错误文件输出
    let error_file_appender = rolling::daily("logs", "error.log");
    let (error_non_blocking, _error_guard) = non_blocking(error_file_appender);
    
    // 配置不同级别的输出目标
    let stdout_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(false)
        .with_thread_ids(true)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
    
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG);
    
    let error_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(error_non_blocking)
        .with_filter(tracing_subscriber::filter::LevelFilter::ERROR);
    
    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(error_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // 保持guard不被释放
    // _guard和_error_guard需要在应用生命周期内保持
}
```

### 日志采样策略
```rust
use rand::Rng;

pub struct LogSampler {
    sample_rate: f64,
    always_sample_codes: Vec<String>,
    rng: rand::rngs::ThreadRng,
}

impl LogSampler {
    pub fn new(sample_rate: f64, always_sample_codes: Vec<String>) -> Self {
        Self {
            sample_rate,
            always_sample_codes,
            rng: rand::thread_rng(),
        }
    }
    
    pub fn should_sample(&mut self, error_code: &str, severity: &str) -> bool {
        // 总是采样的错误代码
        if self.always_sample_codes.contains(&error_code.to_string()) {
            return true;
        }
        
        // 总是采样的严重级别
        if severity == "ERROR" {
            return true;
        }
        
        // 根据采样率决定
        let sample_value: f64 = self.rng.gen();
        sample_value < self.sample_rate
    }
}

// 使用示例
pub fn sampled_log_info(
    sampler: &mut LogSampler,
    error_code: &str,
    fields: &str,
) {
    if sampler.should_sample(error_code, "INFO") {
        log_info!(
            error_code = error_code,
            details = fields,
            "采样日志记录"
        );
    }
}
```

## 最佳实践

### 日志记录最佳实践

#### 1. 结构化优先
```rust
// 好的做法：结构化日志
log_error!(
    error_code = "ORDER.VALIDATION.FAILED",
    trace_id = %trace_id,
    order_id = %order_id,
    field = "email",
    provided_value = %invalid_email,
    expected_format = "user@domain.com",
    "订单验证失败：邮箱格式错误"
);

// 避免：非结构化日志
log::error!("订单验证失败，订单ID: {}, 邮箱: {}, 格式错误", order_id, invalid_email);
```

#### 2. 上下文完整性
```rust
pub fn process_payment(ctx: &ErrorContext, payment: &Payment) -> Result<PaymentResult, Error> {
    let span = ctx.create_span("process_payment");
    let _guard = span.enter();
    
    log_info!(
        error_code = "PAYMENT.PROCESS.START",
        payment_id = %payment.id,
        amount = %payment.amount,
        currency = %payment.currency,
        user_id = %payment.user_id,
        "开始处理支付"
    );
    
    match payment_gateway.process(payment).await {
        Ok(result) => {
            log_info!(
                error_code = "PAYMENT.PROCESS.SUCCESS",
                payment_id = %payment.id,
                transaction_id = %result.transaction_id,
                processing_time_ms = result.processing_time_ms,
                "支付处理成功"
            );
            Ok(result)
        }
        Err(e) => {
            log_error!(
                error_code = "PAYMENT.PROCESS.FAILED",
                payment_id = %payment.id,
                error_type = %e.error_type(),
                error_detail = %e,
                retry_count = %e.retry_count(),
                recovery_action = "建议用户稍后重试或联系客服",
                "支付处理失败"
            );
            Err(e)
        }
    }
}
```

#### 3. 性能感知
```rust
use std::time::Instant;

pub fn handle_request(ctx: &ErrorContext, request: &Request) -> Result<Response, Error> {
    let start_time = Instant::now();
    let span = ctx.create_span("handle_request");
    let _guard = span.enter();
    
    log_info!(
        error_code = "REQUEST.RECEIVED",
        request_id = %request.id,
        method = %request.method,
        path = %request.path,
        "接收到请求"
    );
    
    let result = process_request(request).await;
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    
    match &result {
        Ok(response) => {
            log_info!(
                error_code = "REQUEST.COMPLETED",
                request_id = %request.id,
                status_code = %response.status_code,
                duration_ms = duration_ms,
                response_size = %response.size,
                "请求处理完成"
            );
        }
        Err(e) => {
            log_error!(
                error_code = "REQUEST.FAILED",
                request_id = %request.id,
                error_type = %e.error_type(),
                duration_ms = duration_ms,
                "请求处理失败"
            );
        }
    }
    
    result
}
```

#### 4. 敏感信息处理
```rust
pub fn log_user_action(ctx: &ErrorContext, user: &User, action: &str) {
    // 屏蔽敏感信息
    let masked_phone = mask_phone_number(&user.phone);
    let masked_email = mask_email(&user.email);
    
    log_info!(
        error_code = "USER.ACTION",
        trace_id = %ctx.trace_id,
        user_id = %user.id,
        masked_phone = %masked_phone,
        masked_email = %masked_email,
        action = action,
        timestamp = %Utc::now(),
        "用户操作记录"
    );
}

fn mask_phone_number(phone: &str) -> String {
    if phone.len() >= 8 {
        format!("{}****{}", &phone[..3], &phone[phone.len()-4..])
    } else {
        "****".to_string()
    }
}

fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let username = &email[..at_pos];
        let domain = &email[at_pos..];
        
        if username.len() > 2 {
            format!("{}**{}", &username[..1], domain)
        } else {
            format!("**{}", domain)
        }
    } else {
        "****".to_string()
    }
}
```

### 日志配置最佳实践

#### 1. 环境相关配置
```rust
#[derive(Debug, serde::Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
    pub file_output: bool,
    pub max_file_size_mb: u64,
    pub max_file_count: u32,
    pub async_logging: bool,
    pub sample_rate: f64,
    pub sensitive_fields: Vec<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            file_output: true,
            max_file_size_mb: 100,
            max_file_count: 10,
            async_logging: true,
            sample_rate: 1.0,
            sensitive_fields: vec![
                "password".to_string(),
                "credit_card".to_string(),
                "token".to_string(),
            ],
        }
    }
}

impl LogConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(level) = env::var("LOG_LEVEL") {
            config.level = level;
        }
        
        if let Ok(format) = env::var("LOG_FORMAT") {
            config.format = format;
        }
        
        if let Ok(sample_rate) = env::var("LOG_SAMPLE_RATE") {
            if let Ok(rate) = sample_rate.parse::<f64>() {
                config.sample_rate = rate;
            }
        }
        
        config
    }
    
    pub fn setup_logging(&self) {
        // 根据配置设置日志系统
        let level_filter = match self.level.as_str() {
            "error" => tracing_subscriber::filter::LevelFilter::ERROR,
            "warn" => tracing_subscriber::filter::LevelFilter::WARN,
            "info" => tracing_subscriber::filter::LevelFilter::INFO,
            "debug" => tracing_subscriber::filter::LevelFilter::DEBUG,
            "trace" => tracing_subscriber::filter::LevelFilter::TRACE,
            _ => tracing_subscriber::filter::LevelFilter::INFO,
        };
        
        if self.async_logging {
            setup_async_logging();
        } else {
            setup_sync_logging();
        }
    }
}
```

#### 2. 动态日志级别调整
```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;

pub struct LogManager {
    config: Arc<RwLock<LogConfig>>,
    reload_signal: Arc<AtomicBool>,
}

impl LogManager {
    pub fn new(config: LogConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            reload_signal: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub async fn reload_config(&self, new_config: LogConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
        self.reload_signal.store(true, Ordering::Relaxed);
        
        // 重新设置日志系统
        config.setup_logging();
    }
    
    pub async fn get_config(&self) -> LogConfig {
        self.config.read().await.clone()
    }
    
    pub async fn watch_config_changes(&self) {
        let config = self.config.clone();
        let reload_signal = self.reload_signal.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                
                if reload_signal.load(Ordering::Relaxed) {
                    let current_config = config.read().await;
                    current_config.setup_logging();
                    reload_signal.store(false, Ordering::Relaxed);
                    
                    log_info!(
                        error_code = "LOG.CONFIG.RELOADED",
                        "日志配置已重新加载"
                    );
                }
            }
        });
    }
}
```

## 监控集成

### 监控指标定义
```rust
use prometheus::{Counter, Histogram, Gauge, Registry};

pub struct LogMetrics {
    pub log_total: Counter,
    pub log_by_level: prometheus::Vec<Counter>,
    pub log_by_error_code: prometheus::Vec<Counter>,
    pub log_processing_time: Histogram,
    pub log_buffer_size: Gauge,
    pub log_flush_failures: Counter,
}

impl LogMetrics {
    pub fn new(registry: &Registry) -> Self {
        let log_total = Counter::new("log_total", "Total number of log entries")
            .expect("Failed to create log_total counter");
        
        let log_by_level = prometheus::Vec::new(
            prometheus::Opts::new("log_by_level", "Number of logs by level"),
            &["level"]
        ).expect("Failed to create log_by_level counter");
        
        let log_by_error_code = prometheus::Vec::new(
            prometheus::Opts::new("log_by_error_code", "Number of logs by error code"),
            &["error_code"]
        ).expect("Failed to create log_by_error_code counter");
        
        let log_processing_time = Histogram::new(
            "log_processing_time_seconds",
            "Time spent processing log entries"
        ).expect("Failed to create log_processing_time histogram");
        
        let log_buffer_size = Gauge::new("log_buffer_size", "Current log buffer size")
            .expect("Failed to create log_buffer_size gauge");
        
        let log_flush_failures = Counter::new(
            "log_flush_failures_total", 
            "Total number of log flush failures"
        ).expect("Failed to create log_flush_failures counter");
        
        registry.register(Box::new(log_total.clone())).unwrap();
        registry.register(Box::new(log_by_level.clone())).unwrap();
        registry.register(Box::new(log_by_error_code.clone())).unwrap();
        registry.register(Box::new(log_processing_time.clone())).unwrap();
        registry.register(Box::new(log_buffer_size.clone())).unwrap();
        registry.register(Box::new(log_flush_failures.clone())).unwrap();
        
        Self {
            log_total,
            log_by_level,
            log_by_error_code,
            log_processing_time,
            log_buffer_size,
            log_flush_failures,
        }
    }
    
    pub fn record_log(&self, level: &str, error_code: &str, processing_time: f64) {
        self.log_total.inc();
        self.log_by_level.with_label_values(&[level]).inc();
        self.log_by_error_code.with_label_values(&[error_code]).inc();
        self.log_processing_time.observe(processing_time);
    }
    
    pub fn set_buffer_size(&self, size: f64) {
        self.log_buffer_size.set(size);
    }
    
    pub fn record_flush_failure(&self) {
        self.log_flush_failures.inc();
    }
}
```

### 告警规则定义
```yaml
# prometheus-alerts.yml
groups:
  - name: log_monitoring
    rules:
      # 高错误率告警
      - alert: HighErrorRate
        expr: rate(log_by_level{level="ERROR"}[5m]) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per minute"
      
      # 日志处理延迟告警
      - alert: LogProcessingLatency
        expr: histogram_quantile(0.95, rate(log_processing_time_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High log processing latency"
          description: "95th percentile log processing time is {{ $value }} seconds"
      
      # 日志缓冲区大小告警
      - alert: LogBufferSizeHigh
        expr: log_buffer_size > 10000
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Log buffer size is high"
          description: "Log buffer size is {{ $value }} entries"
      
      # 特定错误类型告警
      - alert: SpecificErrorTypeHigh
        expr: rate(log_by_error_code{error_code="DATABASE.CONNECTION.FAILED"}[5m]) > 5
        for: 3m
        labels:
          severity: critical
        annotations:
          summary: "High rate of database connection failures"
          description: "Database connection failures rate is {{ $value }} per minute"
```

## 相关文档

- [错误分类体系](./01-error-classification.md) - 不同类型错误的分类方法
- [错误处理策略](./02-handling-strategies.md) - 不同类型错误的处理策略
- [错误归集机制](./03-error-aggregation.md) - 跨层错误转换方法
- [错误处理层级](./04-handling-layers.md) - 分层处理模型
- [最佳实践指南](./06-best-practices.md) - 综合最佳实践指南