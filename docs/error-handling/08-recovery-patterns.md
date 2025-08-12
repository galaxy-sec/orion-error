# 错误恢复模式

## 概述

错误恢复模式是错误处理系统中的关键组成部分，定义了系统在面对各种错误情况时的自动恢复和容错机制。通过合理的恢复模式，可以显著提高系统的可靠性、可用性和用户体验，减少人工干预的需求。

## 恢复模式分类

### 自动恢复模式
自动恢复模式是指系统能够自动检测到错误并执行恢复操作，无需人工干预的模式。

#### 重试模式
重试模式是最常用的自动恢复模式，适用于临时性故障。

```rust
use std::time::Duration;
use backoff::{ExponentialBackoff, backoff::Future};
use tokio::time::sleep;
use std::future::Future;
use std::pin::Pin;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Retryable {
    fn is_retryable(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }
    
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> BoxFuture<'_, Result<T, E>>,
        E: Retryable + std::fmt::Debug,
    {
        let mut attempt = 0;
        
        loop {
            attempt += 1;
            let result = operation().await;
            
            match result {
                Ok(success) => return Ok(success),
                Err(error) if error.is_retryable() && attempt < self.config.max_attempts => {
                    let delay = self.calculate_delay(attempt);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }
    
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.config.base_delay.as_millis() as f64
            * self.config.multiplier.powi(attempt as i32 - 1);
        
        let mut delay = Duration::from_millis(delay_ms as u64);
        
        if delay > self.config.max_delay {
            delay = self.config.max_delay;
        }
        
        if self.config.jitter {
            let jitter_ms = (delay.as_millis() as f64 * 0.1) as u64;
            let jitter = rand::random::<u64>() % (2 * jitter_ms + 1);
            delay = Duration::from_millis(delay.as_millis() as u64 + jitter - jitter_ms);
        }
        
        delay
    }
}

// 使用示例
pub async fn fetch_data_with_retry(url: &str) -> Result<String, NetworkError> {
    let retry_config = RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(10),
        ..Default::default()
    };
    
    let executor = RetryExecutor::new(retry_config);
    
    executor.execute(|| {
        Box::pin(async {
            fetch_data(url).await
        })
    }).await
}
```

#### 指数退避重试
指数退避重试是在重试模式基础上增加智能延迟的策略。

```rust
pub struct ExponentialBackoffRetry {
    config: RetryConfig,
    backoff_strategy: BackoffStrategy,
}

#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    Exponential,
    Linear,
    Fibonacci,
    Custom(Box<dyn Fn(u32) -> Duration + Send + Sync>),
}

impl ExponentialBackoffRetry {
    pub fn new(config: RetryConfig, strategy: BackoffStrategy) -> Self {
        Self {
            config,
            backoff_strategy: strategy,
        }
    }
    
    pub async fn execute_with_backoff<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> BoxFuture<'_, Result<T, E>>,
        E: Retryable + std::fmt::Debug,
    {
        let mut attempt = 0;
        
        loop {
            attempt += 1;
            let result = operation().await;
            
            match result {
                Ok(success) => return Ok(success),
                Err(error) if error.is_retryable() && attempt < self.config.max_attempts => {
                    let delay = self.calculate_backoff_delay(attempt);
                    
                    // 记录重试日志
                    info!(
                        "操作失败，第 {} 次重试，延迟 {:?}: {:?}",
                        attempt, delay, error
                    );
                    
                    sleep(delay).await;
                    continue;
                }
                Err(error) => {
                    error!("操作最终失败: {:?}", error);
                    return Err(error);
                }
            }
        }
    }
    
    fn calculate_backoff_delay(&self, attempt: u32) -> Duration {
        match &self.backoff_strategy {
            BackoffStrategy::Exponential => {
                let delay = self.config.base_delay * 2u32.pow(attempt - 1);
                std::cmp::min(delay, self.config.max_delay)
            },
            BackoffStrategy::Linear => {
                let delay = self.config.base_delay * attempt;
                std::cmp::min(delay, self.config.max_delay)
            },
            BackoffStrategy::Fibonacci => {
                let delay = self.config.base_delay * self.fibonacci(attempt);
                std::cmp::min(delay, self.config.max_delay)
            },
            BackoffStrategy::Custom(func) => {
                std::cmp::min(func(attempt), self.config.max_delay)
            }
        }
    }
    
    fn fibonacci(&self, n: u32) -> u32 {
        match n {
            0 => 0,
            1 => 1,
            _ => {
                let mut a = 0;
                let mut b = 1;
                for _ in 2..=n {
                    let next = a + b;
                    a = b;
                    b = next;
                }
                b
            }
        }
    }
}
```

### 隔离模式
隔离模式通过将系统资源进行隔离，防止错误在组件间传播。

#### 舱壁模式
舱壁模式通过限制资源使用，防止单个组件的错误影响整个系统。

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use tokio::sync::Semaphore;
use std::time::Duration;

pub struct Bulkhead {
    semaphore: Arc<Semaphore>,
    max_concurrent: u32,
    timeout: Duration,
    active_requests: AtomicU32,
    circuit_breaker: Arc<CircuitBreaker>,
}

#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    pub max_concurrent: u32,
    pub timeout: Duration,
    pub circuit_breaker_config: CircuitBreakerConfig,
}

impl Bulkhead {
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent as usize)),
            max_concurrent: config.max_concurrent,
            timeout: config.timeout,
            active_requests: AtomicU32::new(0),
            circuit_breaker: Arc::new(CircuitBreaker::new(config.circuit_breaker_config)),
        }
    }
    
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    where
        F: FnOnce() -> BoxFuture<'_, Result<T, E>>,
        E: std::fmt::Debug,
    {
        // 检查断路器状态
        if self.circuit_breaker.is_open() {
            return Err(BulkheadError::CircuitOpen);
        }
        
        // 获取许可
        let permit = tokio::time::timeout(
            self.timeout,
            self.semaphore.acquire()
        ).await;
        
        let permit = match permit {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Err(BulkheadError::SemaphoreClosed),
            Err(_) => return Err(BulkheadError::Timeout),
        };
        
        self.active_requests.fetch_add(1, Ordering::Relaxed);
        
        // 执行操作
        let result = tokio::time::timeout(
            self.timeout,
            operation()
        ).await;
        
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
        drop(permit);
        
        match result {
            Ok(Ok(success)) => {
                self.circuit_breaker.record_success();
                Ok(success)
            },
            Ok(Err(error)) => {
                self.circuit_breaker.record_failure();
                Err(BulkheadError::OperationError(error))
            },
            Err(_) => {
                self.circuit_breaker.record_failure();
                Err(BulkheadError::Timeout)
            }
        }
    }
    
    pub fn active_requests(&self) -> u32 {
        self.active_requests.load(Ordering::Relaxed)
    }
    
    pub fn available_permits(&self) -> u32 {
        self.semaphore.available_permits() as u32
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BulkheadError<E> {
    #[error("断路器处于开启状态")]
    CircuitOpen,
    
    #[error("信号量已关闭")]
    SemaphoreClosed,
    
    #[error("操作超时")]
    Timeout,
    
    #[error("操作执行失败: {0}")]
    OperationError(E),
}

impl<E> Retryable for BulkheadError<E> {
    fn is_retryable(&self) -> bool {
        match self {
            BulkheadError::CircuitOpen => true,
            BulkheadError::Timeout => true,
            _ => false,
        }
    }
}
```

### 断路器模式
断路器模式在系统连续失败时暂时停止请求，避免资源浪费。

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU8, AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub expected_exception_predicate: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    pub ring_buffer_size_in_half_open_state: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            expected_exception_predicate: None,
            ring_buffer_size_in_half_open_state: 10,
        }
    }
}

pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU32,
    last_failure_time: AtomicU64,
    success_count: AtomicU32,
    config: CircuitBreakerConfig,
    ring_buffer_half_open: Arc<std::sync::Mutex<VecDeque<bool>>>,
}

const STATE_CLOSED: u8 = 0;
const STATE_OPEN: u8 = 1;
const STATE_HALF_OPEN: u8 = 2;

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: AtomicU8::new(STATE_CLOSED),
            failure_count: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            success_count: AtomicU32::new(0),
            config,
            ring_buffer_half_open: Arc::new(std::sync::Mutex::new(VecDeque::new())),
        }
    }
    
    pub fn is_open(&self) -> bool {
        self.state.load(Ordering::Relaxed) == STATE_OPEN
    }
    
    pub fn record_success(&self) {
        let current_state = self.state.load(Ordering::Relaxed);
        
        match current_state {
            STATE_CLOSED => {
                self.failure_count.store(0, Ordering::Relaxed);
            },
            STATE_HALF_OPEN => {
                let mut buffer = self.ring_buffer_half_open.lock().unwrap();
                buffer.push_back(true);
                
                if buffer.len() >= self.config.ring_buffer_size_in_half_open_state as usize {
                    if buffer.iter().all(|&success| success) {
                        self.transition_to_closed_state();
                    }
                    buffer.clear();
                }
                
                self.success_count.fetch_add(1, Ordering::Relaxed);
            },
            _ => {}
        }
    }
    
    pub fn record_failure(&self) {
        let current_state = self.state.load(Ordering::Relaxed);
        let now = Instant::now();
        
        match current_state {
            STATE_CLOSED => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                self.last_failure_time.store(
                    now.duration_since(Instant::UNIX_EPOCH).unwrap().as_secs(),
                    Ordering::Relaxed
                );
                
                if failures >= self.config.failure_threshold {
                    self.transition_to_open_state();
                }
            },
            STATE_HALF_OPEN => {
                let mut buffer = self.ring_buffer_half_open.lock().unwrap();
                buffer.push_back(false);
                
                if buffer.len() >= self.config.ring_buffer_size_in_half_open_state as usize {
                    let success_rate = buffer.iter().filter(|&&success| success).count() as f64 / buffer.len() as f64;
                    if success_rate < 0.5 {
                        self.transition_to_open_state();
                    }
                    buffer.clear();
                }
            },
            _ => {}
        }
    }
    
    fn transition_to_open_state(&self) {
        self.state.store(STATE_OPEN, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        
        // 启动恢复定时器
        let config = self.config.clone();
        tokio::spawn(async move {
            sleep(config.recovery_timeout).await;
            // 这里需要访问self，但因为在不同的async block中，需要使用其他机制
        });
    }
    
    fn transition_to_closed_state(&self) {
        self.state.store(STATE_CLOSED, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
    }
    
    pub async fn attempt_reset(&self) -> bool {
        let current_state = self.state.load(Ordering::Relaxed);
        
        if current_state == STATE_OPEN {
            let last_failure = self.last_failure_time.load(Ordering::Relaxed);
            let now = Instant::now().duration_since(Instant::UNIX_EPOCH).unwrap().as_secs();
            
            if now.saturating_sub(last_failure) >= self.config.recovery_timeout.as_secs() {
                self.state.store(STATE_HALF_OPEN, Ordering::Relaxed);
                self.ring_buffer_half_open.lock().unwrap().clear();
                return true;
            }
        }
        
        false
    }
    
    pub fn get_state(&self) -> CircuitBreakerState {
        match self.state.load(Ordering::Relaxed) {
            STATE_CLOSED => CircuitBreakerState::Closed,
            STATE_OPEN => CircuitBreakerState::Open,
            STATE_HALF_OPEN => CircuitBreakerState::HalfOpen,
            _ => CircuitBreakerState::Closed,
        }
    }
    
    pub fn get_metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            state: self.get_state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            failure_rate: self.calculate_failure_rate(),
        }
    }
    
    fn calculate_failure_rate(&self) -> f64 {
        let failures = self.failure_count.load(Ordering::Relaxed);
        let successes = self.success_count.load(Ordering::Relaxed);
        let total = failures + successes;
        
        if total == 0 {
            0.0
        } else {
            failures as f64 / total as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub failure_rate: f64,
}
```

## 降级恢复模式

### 降级策略
降级策略在系统资源不足或外部服务不可用时，提供简化的功能。

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait DegradableService<T, E> {
    async fn execute_full(&self) -> Result<T, E>;
    async fn execute_degraded(&self) -> Result<T, E>;
    async fn should_degrade(&self) -> bool;
}

pub struct DegradationManager<T, E> {
    services: HashMap<String, Arc<dyn DegradableService<T, E> + Send + Sync>>,
    degradation_config: DegradationConfig,
    current_mode: Arc<RwLock<DegradationMode>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DegradationMode {
    Normal,
    Partial(HashMap<String, bool>), // service_name -> is_degraded
    Full,
}

#[derive(Debug, Clone)]
pub struct DegradationConfig {
    pub auto_degradation_enabled: bool,
    pub degradation_threshold: f64, // 错误率阈值
    pub recovery_threshold: f64,    // 恢复阈值
    pub monitoring_interval: Duration,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            auto_degradation_enabled: true,
            degradation_threshold: 0.1, // 10%错误率
            recovery_threshold: 0.05,  // 5%错误率
            monitoring_interval: Duration::from_secs(30),
        }
    }
}

impl<T, E: std::fmt::Debug> DegradationManager<T, E> {
    pub fn new(config: DegradationConfig) -> Self {
        Self {
            services: HashMap::new(),
            degradation_config: config,
            current_mode: Arc::new(RwLock::new(DegradationMode::Normal)),
        }
    }
    
    pub fn register_service<S>(&mut self, name: String, service: S)
    where
        S: DegradableService<T, E> + Send + Sync + 'static,
    {
        self.services.insert(name, Arc::new(service));
    }
    
    pub async fn execute_service(&self, service_name: &str) -> Result<T, DegradationError<E>> {
        let service = self.services.get(service_name)
            .ok_or_else(|| DegradationError::ServiceNotFound(service_name.to_string()))?;
        
        let mode = self.current_mode.read().await;
        
        let should_degrade = match *mode {
            DegradationMode::Normal => service.should_degrade().await,
            DegradationMode::Partial(ref degraded_services) => {
                degraded_services.get(service_name).copied().unwrap_or(false)
            },
            DegradationMode::Full => true,
        };
        
        drop(mode);
        
        if should_degrade {
            service.execute_degraded().await
                .map_err(DegradationError::DegradedExecution)
        } else {
            service.execute_full().await
                .map_err(DegradationError::FullExecution)
        }
    }
    
    pub async fn start_monitoring(&self) {
        if !self.degradation_config.auto_degradation_enabled {
            return;
        }
        
        let services = self.services.clone();
        let current_mode = self.current_mode.clone();
        let config = self.degradation_config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.monitoring_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::monitor_and_adjust(&services, &current_mode, &config).await {
                    error!("降级监控失败: {}", e);
                }
            }
        });
    }
    
    async fn monitor_and_adjust(
        services: &HashMap<String, Arc<dyn DegradableService<T, E> + Send + Sync>>,
        current_mode: &Arc<RwLock<DegradationMode>>,
        config: &DegradationConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut service_errors = HashMap::new();
        
        // 收集各服务的错误率
        for (name, service) in services {
            let should_degrade = service.should_degrade().await;
            service_errors.insert(name.clone(), should_degrade);
        }
        
        let mut current_mode_guard = current_mode.write().await;
        
        match &*current_mode_guard {
            DegradationMode::Normal => {
                // 检查是否需要降级
                let degrade_count = service_errors.values().filter(|&&should_degrade| should_degrade).count();
                let total_services = service_errors.len();
                
                if total_services > 0 && (degrade_count as f64 / total_services as f64) > config.degradation_threshold {
                    let mut degraded_services = HashMap::new();
                    for (name, &should_degrade) in &service_errors {
                        degraded_services.insert(name.clone(), should_degrade);
                    }
                    
                    *current_mode_guard = DegradationMode::Partial(degraded_services);
                    warn!("系统进入部分降级模式");
                }
            },
            DegradationMode::Partial(ref degraded_services) => {
                // 检查是否可以恢复
                let error_count = degraded_services.values().filter(|&&should_degrade| should_degrade).count();
                let total_services = degraded_services.len();
                
                if total_services > 0 && (error_count as f64 / total_services as f64) < config.recovery_threshold {
                    *current_mode_guard = DegradationMode::Normal;
                    info!("系统恢复正常模式");
                } else {
                    // 更新降级服务列表
                    let mut new_degraded_services = HashMap::new();
                    for (name, _) in degraded_services {
                        let should_degrade = service_errors.get(name).copied().unwrap_or(false);
                        new_degraded_services.insert(name.clone(), should_degrade);
                    }
                    
                    *current_mode_guard = DegradationMode::Partial(new_degraded_services);
                }
            },
            DegradationMode::Full => {
                // 检查是否可以恢复到部分降级
                let healthy_count = service_errors.values().filter(|&&should_degrade| !should_degrade).count();
                
                if healthy_count > 0 {
                    let mut degraded_services = HashMap::new();
                    for (name, &should_degrade) in &service_errors {
                        degraded_services.insert(name.clone(), should_degrade);
                    }
                    
                    *current_mode_guard = DegradationMode::Partial(degraded_services);
                    warn!("系统从全降级恢复到部分降级模式");
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn force_degradation(&self, mode: DegradationMode) {
        let mut current_mode = self.current_mode.write().await;
        *current_mode = mode;
    }
    
    pub async fn get_current_mode(&self) -> DegradationMode {
        self.current_mode.read().await.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DegradationError<E> {
    #[error("服务未找到: {0}")]
    ServiceNotFound(String),
    
    #[error("完整模式执行失败: {0}")]
    FullExecution(E),
    
    #[error("降级模式执行失败: {0}")]
    DegradedExecution(E),
}

// 示例降级服务实现
pub struct RecommendationService {
    full_client: Arc<dyn RecommendationClient>,
    cache_client: Arc<dyn CacheClient>,
    error_rate: Arc<RwLock<f64>>,
}

impl DegradableService<Vec<String>, RecommendationError> for RecommendationService {
    async fn execute_full(&self) -> Result<Vec<String>, RecommendationError> {
        self.full_client.get_recommendations().await
            .map_err(|e| {
                self.update_error_rate(true).await;
                RecommendationError::FullServiceError(e)
            })
    }
    
    async fn execute_degraded(&self) -> Result<Vec<String>, RecommendationError> {
        // 尝试从缓存获取热门推荐
        if let Ok(cached_recommendations) = self.cache_client.get_popular_items().await {
            self.update_error_rate(false).await;
            Ok(cached_recommendations)
        } else {
            // 如果缓存也没有，返回空列表
            self.update_error_rate(false).await;
            Ok(vec![])
        }
    }
    
    async fn should_degrade(&self) -> bool {
        let error_rate = self.error_rate.read().await;
        *error_rate > 0.1 // 错误率超过10%时降级
    }
}

impl RecommendationService {
    async fn update_error_rate(&self, is_error: bool) {
        let mut rate = self.error_rate.write().await;
        // 简单的移动平均计算
        *rate = if is_error {
            (*rate * 0.9) + 0.1
        } else {
            *rate * 0.9
        };
    }
}
```

### 缓存降级
缓存降级通过使用缓存数据替代实时数据，在服务不可用时提供基本功能。

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct CacheDegradationStrategy<T> {
    cache: Arc<dyn Cache<T>>,
    fallback_data: Arc<RwLock<HashMap<String, T>>>,
    cache_ttl: Duration,
    fallback_ttl: Duration,
    metrics: Arc<CacheDegradationMetrics>,
}

#[derive(Debug, Clone)]
pub struct CacheDegradationMetrics {
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub fallback_hits: AtomicU64,
    pub fallback_misses: AtomicU64,
    pub degradation_events: AtomicU64,
}

impl<T: Clone> CacheDegradationStrategy<T> {
    pub fn new(
        cache: Arc<dyn Cache<T>>,
        cache_ttl: Duration,
        fallback_ttl: Duration,
    ) -> Self {
        Self {
            cache,
            fallback_data: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            fallback_ttl,
            metrics: Arc::new(CacheDegradationMetrics::new()),
        }
    }
    
    pub async fn get_data(&self, key: &str) -> Result<T, CacheDegradationError> {
        // 首先尝试从缓存获取
        match self.cache.get(key).await {
            Ok(Some(data)) => {
                self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(data);
            },
            Ok(None) => {
                self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
            },
            Err(e) => {
                self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
                warn!("缓存访问失败: {}", e);
            }
        }
        
        // 缓存未命中，尝试从备用数据获取
        let fallback_data = self.fallback_data.read().await;
        if let Some(data) = fallback_data.get(key) {
            self.metrics.fallback_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(data.clone());
        }
        self.metrics.fallback_misses.fetch_add(1, Ordering::Relaxed);
        
        Err(CacheDegradationError::DataNotFound)
    }
    
    pub async fn set_fallback_data(&self, key: String, data: T) {
        let mut fallback_data = self.fallback_data.write().await;
        fallback_data.insert(key, data);
    }
    
    pub async fn preload_cache(&self, keys: Vec<String>) {
        info!("开始预加载缓存，键数量: {}", keys.len());
        
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for key in keys {
            match self.cache.get(&key).await {
                Ok(Some(_)) => {
                    success_count += 1;
                },
                Ok(None) | Err(_) => {
                    failure_count += 1;
                }
            }
        }
        
        info!("缓存预加载完成，成功: {}, 失败: {}", success_count, failure_count);
    }
    
    pub async fn get_metrics(&self) -> CacheDegradationMetricsSnapshot {
        CacheDegradationMetricsSnapshot {
            cache_hits: self.metrics.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.metrics.cache_misses.load(Ordering::Relaxed),
            fallback_hits: self.metrics.fallback_hits.load(Ordering::Relaxed),
            fallback_misses: self.metrics.fallback_misses.load(Ordering::Relaxed),
            degradation_events: self.metrics.degradation_events.load(Ordering::Relaxed),
            cache_hit_rate: self.calculate_cache_hit_rate(),
        }
    }
    
    fn calculate_cache_hit_rate(&self) -> f64 {
        let hits = self.metrics.cache_hits.load(Ordering::Relaxed);
        let misses = self.metrics.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

impl CacheDegradationMetrics {
    fn new() -> Self {
        Self {
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            fallback_hits: AtomicU64::new(0),
            fallback_misses: AtomicU64::new(0),
            degradation_events: AtomicU64::new(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheDegradationMetricsSnapshot {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub fallback_hits: u64,
    pub fallback_misses: u64,
    pub degradation_events: u64,
    pub cache_hit_rate: f64,
}

#[async_trait::async_trait]
pub trait Cache<T: Send + Sync + Clone>: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>;
    async fn set(&self, key: &str, value: T, ttl: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait::async_trait]
pub trait RecommendationClient: Send + Sync {
    async fn get_recommendations(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait::async_trait]
pub trait CacheClient: Send + Sync {
    async fn get_popular_items(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug, thiserror::Error)]
pub enum CacheDegradationError {
    #[error("数据未找到")]
    DataNotFound,
    
    #[error("缓存操作失败: {0}")]
    CacheError(String),
    
    #[error("降级操作失败: {0}")]
    DegradationError(String),
}
```

## 超时控制模式

### 智能超时
智能超时根据系统负载和历史响应时间动态调整超时阈值。

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::VecDeque;

pub struct SmartTimeout {
    base_timeout: Duration,
    max_timeout: Duration,
    min_timeout: Duration,
    adaptive_factor: f64,
    response_times: Arc<RwLock<VecDeque<Duration>>>,
    window_size: usize,
    load_factor: Arc<AtomicU64>,
}

impl SmartTimeout {
    pub fn new(base_timeout: Duration) -> Self {
        Self {
            base_timeout,
            max_timeout: Duration::from_secs(30),
            min_timeout: Duration::from_millis(100),
            adaptive_factor: 2.0,
            response_times: Arc::new(RwLock::new(VecDeque::new())),
            window_size: 100,
            load_factor: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub async fn execute_with_timeout<F, T, E>(&self, operation: F) -> Result<T, TimeoutError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let timeout = self.calculate_timeout().await;
        let start_time = Instant::now();
        
        match tokio::time::timeout(timeout, operation).await {
            Ok(Ok(result)) => {
                let actual_duration = start_time.elapsed();
                self.record_response_time(actual_duration).await;
                Ok(result)
            },
            Ok(Err(error)) => {
                let actual_duration = start_time.elapsed();
                self.record_response_time(actual_duration).await;
                Err(TimeoutError::OperationError(error))
            },
            Err(_) => {
                self.record_timeout().await;
                Err(TimeoutError::Timeout(timeout))
            }
        }
    }
    
    async fn calculate_timeout(&self) -> Duration {
        let load = self.load_factor.load(Ordering::Relaxed);
        let response_times = self.response_times.read().await;
        
        let avg_response_time = if response_times.is_empty() {
            self.base_timeout
        } else {
            let total: Duration = response_times.iter().sum();
            total / response_times.len() as u32
        };
        
        let load_factor = if load > 80 { 1.5 } else if load > 60 { 1.2 } else { 1.0 };
        let calculated_timeout = avg_response_time * self.adaptive_factor * load_factor;
        
        calculated_timeout.clamp(self.min_timeout, self.max_timeout)
    }
    
    async fn record_response_time(&self, duration: Duration) {
        let mut response_times = self.response_times.write().await;
        response_times.push_back(duration);
        
        if response_times.len() > self.window_size {
            response_times.pop_front();
        }
    }
    
    async fn record_timeout(&self) {
        // 记录超时事件，可以触发告警或调整策略
        warn!("操作超时，当前响应时间窗口可能需要调整");
    }
    
    pub async fn update_load_factor(&self, load: u64) {
        self.load_factor.store(load, Ordering::Relaxed);
    }
    
    pub async fn get_stats(&self) -> TimeoutStats {
        let response_times = self.response_times.read().await;
        
        if response_times.is_empty() {
            return TimeoutStats {
                avg_response_time: Duration::from_millis(0),
                min_response_time: Duration::from_millis(0),
                max_response_time: Duration::from_millis(0),
                timeout_count: 0,
                current_load: self.load_factor.load(Ordering::Relaxed),
            };
        }
        
        let min_time = *response_times.iter().min().unwrap();
        let max_time = *response_times.iter().max().unwrap();
        let avg_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
        
        TimeoutStats {
            avg_response_time: avg_time,
            min_response_time: min_time,
            max_response_time: max_time,
            timeout_count: 0, // 需要额外统计
            current_load: self.load_factor.load(Ordering::Relaxed),
        }
    }
}

pub struct TimeoutStats {
    pub avg_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub timeout_count: u64,
    pub current_load: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum TimeoutError<E> {
    #[error("操作超时，超时时间: {0:?}")]
    Timeout(Duration),
    
    #[error("操作执行失败: {0}")]
    OperationError(E),
}
```

### 阶梯超时
阶梯超时为不同类型的操作设置不同的超时策略。

```rust
pub struct TieredTimeout {
    timeout_tiers: Vec<TimeoutTier>,
    fallback_strategy: FallbackStrategy,
}

#[derive(Debug, Clone)]
pub struct TimeoutTier {
    pub operation_type: String,
    pub base_timeout: Duration,
    pub max_retries: u32,
    pub backoff_multiplier: f64,
}

#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    ImmediateError,
    BestEffort,
    GracefulDegradation,
    QueueAndRetry,
}

impl TieredTimeout {
    pub fn new() -> Self {
        Self {
            timeout_tiers: vec![
                TimeoutTier {
                    operation_type: "database_query".to_string(),
                    base_timeout: Duration::from_secs(5),
                    max_retries: 2,
                    backoff_multiplier: 2.0,
                },
                TimeoutTier {
                    operation_type: "external_api".to_string(),
                    base_timeout: Duration::from_secs(10),
                    max_retries: 3,
                    backoff_multiplier: 1.5,
                },
                TimeoutTier {
                    operation_type: "cache_operation".to_string(),
                    base_timeout: Duration::from_millis(100),
                    max_retries: 1,
                    backoff_multiplier: 1.0,
                },
            ],
            fallback_strategy: FallbackStrategy::GracefulDegradation,
        }
    }
    
    pub fn add_tier(mut self, tier: TimeoutTier) -> Self {
        self.timeout_tiers.push(tier);
        self
    }
    
    pub fn with_fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }
    
    pub async fn execute_operation<F, T, E>(&self, operation_type: &str, operation: F) -> Result<T, TieredTimeoutError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let tier = self.timeout_tiers.iter()
            .find(|t| t.operation_type == operation_type)
            .unwrap_or(&self.timeout_tiers[0]); // 使用默认配置
        
        self.execute_with_tier(tier, operation).await
    }
    
    async fn execute_with_tier<F, T, E>(&self, tier: &TimeoutTier, operation: F) -> Result<T, TieredTimeoutError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=tier.max_retries {
            let timeout = if attempt == 0 {
                tier.base_timeout
            } else {
                let delay = tier.base_timeout * tier.backoff_multiplier.powi(attempt as i32);
                tokio::time::sleep(delay).await;
                delay
            };
            
            match tokio::time::timeout(timeout, &mut operation).await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(error)) => {
                    last_error = Some(TieredTimeoutError::OperationError(error));
                    if attempt == tier.max_retries {
                        break;
                    }
                    continue;
                },
                Err(_) => {
                    last_error = Some(TieredTimeoutError::Timeout);
                    if attempt == tier.max_retries {
                        break;
                    }
                    continue;
                }
            }
        }
        
        // 所有重试都失败，应用降级策略
        self.apply_fallback_strategy(operation_type, last_error).await
    }
    
    async fn apply_fallback_strategy<T, E>(&self, operation_type: &str, error: Option<TieredTimeoutError<E>>) -> Result<T, TieredTimeoutError<E>> {
        match self.fallback_strategy {
            FallbackStrategy::ImmediateError => {
                Err(error.unwrap_or(TieredTimeoutError::UnknownError))
            },
            FallbackStrategy::BestEffort => {
                // 返回最佳可能的结果或空结果
                // 这里需要根据具体业务逻辑实现
                Err(TieredTimeoutError::BestEffortFallback)
            },
            FallbackStrategy::GracefulDegradation => {
                // 尝试降级处理
                info!("应用降级策略处理超时操作: {}", operation_type);
                Err(TieredTimeoutError::GracefulDegradation)
            },
            FallbackStrategy::QueueAndRetry => {
                // 将操作放入队列异步重试
                info!("将超时操作放入重试队列: {}", operation_type);
                Err(TieredTimeoutError::QueuedForRetry)
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TieredTimeoutError<E> {
    #[error("操作超时")]
    Timeout,
    
    #[error("操作执行失败: {0}")]
    OperationError(E),
    
    #[error("未知错误")]
    UnknownError,
    
    #[error("最佳努力降级")]
    BestEffortFallback,
    
    #[error("优雅降级")]
    GracefulDegradation,
    
    #[error("已加入重试队列")]
    QueuedForRetry,
}
```

## 恢复编排器

### 恢复流程编排
恢复编排器协调整个系统的恢复过程，确保恢复操作按正确顺序执行。

```rust
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<RecoveryStep>,
    pub rollback_steps: Vec<RecoveryStep>,
    pub dependencies: HashMap<String, Vec<String>>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStep {
    pub id: String,
    pub name: String,
    pub step_type: RecoveryStepType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub rollback_step_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStepType {
    DatabaseRestore,
    ServiceRestart,
    CacheClear,
    ConfigurationUpdate,
    DataValidation,
    HealthCheck,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub backoff_multiplier: f64,
}

pub struct RecoveryOrchestrator {
    plans: Arc<RwLock<HashMap<String, RecoveryPlan>>>,
    execution_history: Arc<RwLock<Vec<RecoveryExecution>>>,
    step_executors: HashMap<String, Arc<dyn RecoveryStepExecutor>>,
    event_bus: Arc<dyn RecoveryEventBus>,
}

#[derive(Debug, Clone)]
pub struct RecoveryExecution {
    pub plan_id: String,
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub step_results: HashMap<String, StepResult>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    RollingBack,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub status: ExecutionStatus,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub attempts: u32,
    pub error_message: Option<String>,
    pub output: Option<serde_json::Value>,
}

#[async_trait::async_trait]
pub trait RecoveryStepExecutor: Send + Sync {
    async fn execute(&self, step: &RecoveryStep, context: &RecoveryContext) -> Result<serde_json::Value, StepExecutionError>;
    async fn rollback(&self, step: &RecoveryStep, context: &RecoveryContext) -> Result<(), StepExecutionError>;
    fn can_execute(&self, step_type: &RecoveryStepType) -> bool;
}

#[derive(Debug, Clone)]
pub struct RecoveryContext {
    pub execution_id: String,
    pub plan_id: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub step_results: HashMap<String, serde_json::Value>,
}

#[async_trait::async_trait]
pub trait RecoveryEventBus: Send + Sync {
    async fn publish(&self, event: RecoveryEvent);
    async fn subscribe(&self, handler: Box<dyn RecoveryEventHandler>);
}

#[async_trait::async_trait]
pub trait RecoveryEventHandler: Send + Sync {
    async fn handle(&self, event: &RecoveryEvent);
}

#[derive(Debug, Clone)]
pub struct RecoveryEvent {
    pub event_type: RecoveryEventType,
    pub execution_id: String,
    pub plan_id: String,
    pub step_id: Option<String>,
    pub timestamp: Instant,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum RecoveryEventType {
    ExecutionStarted,
    StepStarted,
    StepCompleted,
    StepFailed,
    ExecutionCompleted,
    ExecutionFailed,
    RollbackStarted,
    RollbackCompleted,
    RollbackFailed,
}

impl RecoveryOrchestrator {
    pub fn new(event_bus: Arc<dyn RecoveryEventBus>) -> Self {
        Self {
            plans: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            step_executors: HashMap::new(),
            event_bus,
        }
    }
    
    pub fn register_step_executor<T>(&mut self, step_type: RecoveryStepType, executor: T)
    where
        T: RecoveryStepExecutor + 'static,
    {
        let type_str = match &step_type {
            RecoveryStepType::DatabaseRestore => "database_restore".to_string(),
            RecoveryStepType::ServiceRestart => "service_restart".to_string(),
            RecoveryStepType::CacheClear => "cache_clear".to_string(),
            RecoveryStepType::ConfigurationUpdate => "configuration_update".to_string(),
            RecoveryStepType::DataValidation => "data_validation".to_string(),
            RecoveryStepType::HealthCheck => "health_check".to_string(),
            RecoveryStepType::Custom(name) => name.clone(),
        };
        
        self.step_executors.insert(type_str, Arc::new(executor));
    }
    
    pub async fn register_plan(&self, plan: RecoveryPlan) -> Result<(), RecoveryError> {
        let mut plans = self.plans.write().await;
        plans.insert(plan.id.clone(), plan);
        Ok(())
    }
    
    pub async fn execute_plan(&self, plan_id: String, parameters: HashMap<String, serde_json::Value>) -> Result<String, RecoveryError> {
        let plans = self.plans.read().await;
        let plan = plans.get(&plan_id)
            .ok_or(RecoveryError::PlanNotFound(plan_id.clone()))?;
        
        let execution_id = uuid::Uuid::new_v4().to_string();
        
        // 创建执行记录
        let execution = RecoveryExecution {
            plan_id: plan_id.clone(),
            execution_id: execution_id.clone(),
            status: ExecutionStatus::Pending,
            started_at: Instant::now(),
            completed_at: None,
            step_results: HashMap::new(),
            error_message: None,
        };
        
        let mut history = self.execution_history.write().await;
        history.push(execution.clone());
        drop(history);
        
        // 发布开始事件
        self.event_bus.publish(RecoveryEvent {
            event_type: RecoveryEventType::ExecutionStarted,
            execution_id: execution_id.clone(),
            plan_id: plan_id.clone(),
            step_id: None,
            timestamp: Instant::now(),
            data: None,
        }).await;
        
        // 执行恢复计划
        let result = self.execute_recovery_plan(&plan, &execution_id, parameters).await;
        
        // 更新执行状态
        let mut history = self.execution_history.write().await;
        if let Some(exec) = history.iter_mut().find(|e| e.execution_id == execution_id) {
            exec.completed_at = Some(Instant::now());
            exec.status = match &result {
                Ok(_) => ExecutionStatus::Completed,
                Err(_) => ExecutionStatus::Failed,
            };
            exec.error_message = result.as_ref().err().map(|e| e.to_string());
        }
        
        // 发布完成事件
        self.event_bus.publish(RecoveryEvent {
            event_type: match result {
                Ok(_) => RecoveryEventType::ExecutionCompleted,
                Err(_) => RecoveryEventType::ExecutionFailed,
            },
            execution_id: execution_id.clone(),
            plan_id: plan_id.clone(),
            step_id: None,
            timestamp: Instant::now(),
            data: None,
        }).await;
        
        result.map(|_| execution_id)
    }
    
    async fn execute_recovery_plan(
        &self,
        plan: &RecoveryPlan,
        execution_id: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<(), RecoveryError> {
        let context = RecoveryContext {
            execution_id: execution_id.to_string(),
            plan_id: plan.id.clone(),
            parameters,
            step_results: HashMap::new(),
        };
        
        // 构建执行顺序
        let execution_order = self.build_execution_order(plan)?;
        
        // 执行恢复步骤
        for step_id in execution_order {
            let step = plan.steps.iter()
                .find(|s| s.id == step_id)
                .ok_or(RecoveryError::StepNotFound(step_id.clone()))?;
            
            if let Err(e) = self.execute_recovery_step(step, &context).await {
                // 步骤失败，执行回滚
                self.execute_rollback(plan, &context, step_id).await?;
                return Err(e);
            }
        }
        
        Ok(())
    }
    
    async fn execute_recovery_step(&self, step: &RecoveryStep, context: &RecoveryContext) -> Result<(), RecoveryError> {
        let executor = self.step_executors.get(&step.step_type.to_string())
            .ok_or(RecoveryError::NoExecutor(step.step_type.clone()))?;
        
        self.event_bus.publish(RecoveryEvent {
            event_type: RecoveryEventType::StepStarted,
            execution_id: context.execution_id.clone(),
            plan_id: context.plan_id.clone(),
            step_id: Some(step.id.clone()),
            timestamp: Instant::now(),
            data: None,
        }).await;
        
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts <= step.retry_policy.max_attempts {
            attempts += 1;
            
            match executor.execute(step, context).await {
                Ok(output) => {
                    self.event_bus.publish(RecoveryEvent {
                        event_type: RecoveryEventType::StepCompleted,
                        execution_id: context.execution_id.clone(),
                        plan_id: context.plan_id.clone(),
                        step_id: Some(step.id.clone()),
                        timestamp: Instant::now(),
                        data: Some(output),
                    }).await;
                    return Ok(());
                },
                Err(e) => {
                    last_error = Some(e);
                    if attempts < step.retry_policy.max_attempts {
                        let delay = step.retry_policy.base_delay * step.retry_policy.backoff_multiplier.powi(attempts as i32 - 1);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        self.event_bus.publish(RecoveryEvent {
            event_type: RecoveryEventType::StepFailed,
            execution_id: context.execution_id.clone(),
            plan_id: context.plan_id.clone(),
            step_id: Some(step.id.clone()),
            timestamp: Instant::now(),
            data: None,
        }).await;
        
        Err(last_error.unwrap().into())
    }
    
    async fn execute_rollback(&self, plan: &RecoveryPlan, context: &RecoveryContext, failed_step_id: String) -> Result<(), RecoveryError> {
        self.event_bus.publish(RecoveryEvent {
            event_type: RecoveryEventType::RollbackStarted,
            execution_id: context.execution_id.clone(),
            plan_id: context.plan_id.clone(),
            step_id: None,
            timestamp: Instant::now(),
            data: None,
        }).await;
        
        // 获取已执行的步骤，按相反顺序执行回滚
        let executed_steps: Vec<&RecoveryStep> = plan.steps.iter()
            .filter(|step| {
                // 这里需要检查步骤是否已经执行成功
                // 简化实现，实际应该从执行历史中获取
                true
            })
            .collect();
        
        for step in executed_steps.iter().rev() {
            if let Some(rollback_step_id) = &step.rollback_step_id {
                if let Some(rollback_step) = plan.rollback_steps.iter().find(|s| s.id == *rollback_step_id) {
                    let executor = self.step_executors.get(&rollback_step.step_type.to_string())
                        .ok_or(RecoveryError::NoExecutor(rollback_step.step_type.clone()))?;
                    
                    if let Err(e) = executor.rollback(rollback_step, context).await {
                        error!("回滚步骤 {} 失败: {}", rollback_step.id, e);
                        // 继续执行其他回滚步骤
                    }
                }
            }
        }
        
        self.event_bus.publish(RecoveryEvent {
            event_type: RecoveryEventType::RollbackCompleted,
            execution_id: context.execution_id.clone(),
            plan_id: context.plan_id.clone(),
            step_id: None,
            timestamp: Instant::now(),
            data: None,
        }).await;
        
        Ok(())
    }
    
    fn build_execution_order(&self, plan: &RecoveryPlan) -> Result<Vec<String>, RecoveryError> {
        // 拓扑排序，处理步骤依赖关系
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();
        
        for step in &plan.steps {
            if !visited.contains(&step.id) {
                self.visit_step(&step.id, plan, &mut visited, &mut temp_visited, &mut order)?;
            }
        }
        
        Ok(order)
    }
    
    fn visit_step(
        &self,
        step_id: &str,
        plan: &RecoveryPlan,
        visited: &mut std::collections::HashSet<String>,
        temp_visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), RecoveryError> {
        if temp_visited.contains(step_id) {
            return Err(RecoveryError::CircularDependency(step_id.to_string()));
        }
        
        if visited.contains(step_id) {
            return Ok(());
        }
        
        temp_visited.insert(step_id.to_string());
        
        if let Some(dependencies) = plan.dependencies.get(step_id) {
            for dep_id in dependencies {
                self.visit_step(dep_id, plan, visited, temp_visited, order)?;
            }
        }
        
        temp_visited.remove(step_id);
        visited.insert(step_id.to_string());
        order.push(step_id.to_string());
        
        Ok(())
    }
    
    pub async fn get_execution_status(&self, execution_id: &str) -> Option<RecoveryExecution> {
        let history = self.execution_history.read().await;
        history.iter()
            .find(|e| e.execution_id == execution_id)
            .cloned()
    }
    
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), RecoveryError> {
        let mut history = self.execution_history.write().await;
        if let Some(execution) = history.iter_mut().find(|e| e.execution_id == execution_id) {
            if execution.status == ExecutionStatus::Running {
                execution.status = ExecutionStatus::Cancelled;
                execution.completed_at = Some(Instant::now());
                
                self.event_bus.publish(RecoveryEvent {
                    event_type: RecoveryEventType::ExecutionFailed,
                    execution_id: execution_id.to_string(),
                    plan_id: execution.plan_id.clone(),
                    step_id: None,
                    timestamp: Instant::now(),
                    data: None,
                }).await;
                
                return Ok(());
            }
        }
        
        Err(RecoveryError::ExecutionNotFound(execution_id.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("恢复计划未找到: {0}")]
    PlanNotFound(String),
    
    #[error("恢复步骤未找到: {0}")]
    StepNotFound(String),
    
    #[error("未找到步骤执行器: {0:?}")]
    NoExecutor(RecoveryStepType),
    
    #[error("发现循环依赖: {0}")]
    CircularDependency(String),
    
    #[error("步骤执行错误: {0}")]
    StepExecutionError(String),
    
    #[error("执行记录未找到: {0}")]
    ExecutionNotFound(String),
    
    #[error("操作超时")]
    Timeout,
}

impl From<StepExecutionError> for RecoveryError {
    fn from(error: StepExecutionError) -> Self {
        RecoveryError::StepExecutionError(error.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StepExecutionError {
    #[error("数据库操作失败: {0}")]
    DatabaseError(String),
    
    #[error("服务操作失败: {0}")]
    ServiceError(String),
    
    #[error("缓存操作失败: {0}")]
    CacheError(String),
    
    #[error("配置操作失败: {0}")]
    ConfigurationError(String),
    
    #[error("验证失败: {0}")]
    ValidationError(String),
    
    #[error("超时错误")]
    Timeout,
    
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl RecoveryStepType {
    pub fn to_string(&self) -> String {
        match self {
            RecoveryStepType::DatabaseRestore => "database_restore".to_string(),
            RecoveryStepType::ServiceRestart => "service_restart".to_string(),
            RecoveryStepType::CacheClear => "cache_clear".to_string(),
            RecoveryStepType::ConfigurationUpdate => "configuration_update".to_string(),
            RecoveryStepType::DataValidation => "data_validation".to_string(),
            RecoveryStepType::HealthCheck => "health_check".to_string(),
            RecoveryStepType::Custom(name) => name.clone(),
        }
    }
}
```

## 恢复测试框架

### 自动化恢复测试
恢复测试框架用于验证各种恢复模式的有效性。

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Arc;
use serde::{Deserialize, Serialize};

pub struct RecoveryTestFramework {
    test_scenarios: Vec<RecoveryTestScenario>,
    test_results: Arc<std::sync::Mutex<Vec<RecoveryTestResult>>>,
    metrics_collector: Arc<dyn RecoveryMetricsCollector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryTestScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub error_injection: ErrorInjectionConfig,
    pub expected_recovery: ExpectedRecoveryBehavior,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInjectionConfig {
    pub injection_type: ErrorInjectionType,
    pub injection_timing: InjectionTiming,
    pub error_rate: f64,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorInjectionType {
    NetworkFailure,
    DatabaseTimeout,
    ServiceUnavailable,
    ResourceExhaustion,
    ConfigurationError,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InjectionTiming {
    Immediate,
    Delayed(Duration),
    Random,
    ConditionBased(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedRecoveryBehavior {
    pub recovery_mode: RecoveryMode,
    pub max_recovery_time: Duration,
    pub success_criteria: SuccessCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryMode {
    Automatic,
    Manual,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub success_rate_threshold: f64,
    pub response_time_threshold: Duration,
    pub data_consistency_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryTestResult {
    pub scenario_id: String,
    pub execution_id: String,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub status: TestStatus,
    pub recovery_time: Option<Duration>,
    pub success_rate: f64,
    pub average_response_time: Duration,
    pub data_consistent: bool,
    pub error_logs: Vec<String>,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Timeout,
    Inconclusive,
}

#[async_trait::async_trait]
pub trait RecoveryMetricsCollector: Send + Sync {
    async fn collect_metrics(&self, scenario_id: &str) -> Result<HashMap<String, f64>, Box<dyn std::error::Error + Send + Sync>>;
    async fn reset_metrics(&self, scenario_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl RecoveryTestFramework {
    pub fn new(metrics_collector: Arc<dyn RecoveryMetricsCollector>) -> Self {
        Self {
            test_scenarios: Vec::new(),
            test_results: Arc::new(std::sync::Mutex::new(Vec::new())),
            metrics_collector,
        }
    }
    
    pub fn add_scenario(&mut self, scenario: RecoveryTestScenario) {
        self.test_scenarios.push(scenario);
    }
    
    pub async fn run_all_tests(&self) -> Vec<RecoveryTestResult> {
        let mut results = Vec::new();
        
        for scenario in &self.test_scenarios {
            match self.run_test_scenario(scenario).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("测试场景 {} 执行失败: {}", scenario.id, e);
                    // 创建失败结果
                    results.push(RecoveryTestResult {
                        scenario_id: scenario.id.clone(),
                        execution_id: uuid::Uuid::new_v4().to_string(),
                        started_at: Instant::now(),
                        completed_at: Some(Instant::now()),
                        status: TestStatus::Failed,
                        recovery_time: None,
                        success_rate: 0.0,
                        average_response_time: Duration::from_secs(0),
                        data_consistent: false,
                        error_logs: vec![e.to_string()],
                        metrics: HashMap::new(),
                    });
                }
            }
        }
        
        results
    }
    
    pub async fn run_test_scenario(&self, scenario: &RecoveryTestScenario) -> Result<RecoveryTestResult, Box<dyn std::error::Error + Send + Sync>> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let started_at = Instant::now();
        
        info!("开始执行恢复测试: {}", scenario.name);
        
        // 重置指标
        self.metrics_collector.reset_metrics(&scenario.id).await?;
        
        // 注入错误
        let injector = ErrorInjector::new(scenario.error_injection.clone());
        injector.inject().await?;
        
        // 执行测试
        let result = tokio::time::timeout(scenario.timeout, async {
            self.execute_test_logic(scenario).await
        }).await;
        
        let completed_at = Instant::now();
        let recovery_time = Some(completed_at.duration_since(started_at));
        
        // 收集指标
        let metrics = self.metrics_collector.collect_metrics(&scenario.id).await?;
        
        // 评估结果
        let test_result = match result {
            Ok(Ok(test_result)) => {
                let evaluated_result = self.evaluate_test_result(scenario, test_result, metrics).await;
                evaluated_result
            },
            Ok(Err(e)) => {
                RecoveryTestResult {
                    scenario_id: scenario.id.clone(),
                    execution_id,
                    started_at,
                    completed_at: Some(completed_at),
                    status: TestStatus::Failed,
                    recovery_time,
                    success_rate: 0.0,
                    average_response_time: Duration::from_secs(0),
                    data_consistent: false,
                    error_logs: vec![e.to_string()],
                    metrics,
                }
            },
            Err(_) => {
                RecoveryTestResult {
                    scenario_id: scenario.id.clone(),
                    execution_id,
                    started_at,
                    completed_at: Some(completed_at),
                    status: TestStatus::Timeout,
                    recovery_time,
                    success_rate: 0.0,
                    average_response_time: Duration::from_secs(0),
                    data_consistent: false,
                    error_logs: vec!["Test execution timeout".to_string()],
                    metrics,
                }
            }
        };
        
        // 记录结果
        let mut results = self.test_results.lock().unwrap();
        results.push(test_result.clone());
        
        info!("恢复测试完成: {}, 状态: {:?}", scenario.name, test_result.status);
        
        Ok(test_result)
    }
    
    async fn execute_test_logic(&self, scenario: &RecoveryTestScenario) -> Result<RecoveryTestResult, Box<dyn std::error::Error + Send + Sync>> {
        // 这里应该实现具体的测试逻辑
        // 根据场景类型执行不同的测试
        
        Ok(RecoveryTestResult {
            scenario_id: scenario.id.clone(),
            execution_id: uuid::Uuid::new_v4().to_string(),
            started_at: Instant::now(),
            completed_at: None,
            status: TestStatus::Passed,
            recovery_time: None,
            success_rate: 1.0,
            average_response_time: Duration::from_millis(100),
            data_consistent: true,
            error_logs: Vec::new(),
            metrics: HashMap::new(),
        })
    }
    
    async fn evaluate_test_result(
        &self,
        scenario: &RecoveryTestScenario,
        mut test_result: RecoveryTestResult,
        metrics: HashMap<String, f64>,
    ) -> RecoveryTestResult {
        test_result.metrics = metrics;
        test_result.completed_at = Some(Instant::now());
        
        // 评估成功标准
        let expected = &scenario.expected_recovery;
        
        // 检查恢复时间
        let recovery_time_ok = test_result.recovery_time
            .map_or(false, |rt| rt <= expected.max_recovery_time);
        
        // 检查成功率
        let success_rate_ok = test_result.success_rate >= expected.success_criteria.success_rate_threshold;
        
        // 检查响应时间
        let response_time_ok = test_result.average_response_time <= expected.success_criteria.response_time_threshold;
        
        // 检查数据一致性
        let data_consistency_ok = !expected.success_criteria.data_consistency_required || test_result.data_consistent;
        
        // 综合评估
        test_result.status = if recovery_time_ok && success_rate_ok && response_time_ok && data_consistency_ok {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };
        
        test_result
    }
    
    pub fn get_test_results(&self) -> Vec<RecoveryTestResult> {
        self.test_results.lock().unwrap().clone()
    }
    
    pub fn generate_test_report(&self) -> TestReport {
        let results = self.get_test_results();
        
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed_tests = results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let timeout_tests = results.iter().filter(|r| r.status == TestStatus::Timeout).count();
        
        let average_recovery_time = results.iter()
            .filter_map(|r| r.recovery_time)
            .sum::<Duration>() / std::cmp::max(1, total_tests) as u32;
        
        let average_success_rate = results.iter()
            .map(|r| r.success_rate)
            .sum::<f64>() / std::cmp::max(1, total_tests) as f64;
        
        TestReport {
            total_tests,
            passed_tests,
            failed_tests,
            timeout_tests,
            pass_rate: (passed_tests as f64 / total_tests as f64) * 100.0,
            average_recovery_time,
            average_success_rate,
            generated_at: Instant::now(),
            results,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestReport {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub timeout_tests: usize,
    pub pass_rate: f64,
    pub average_recovery_time: Duration,
    pub average_success_rate: f64,
    pub generated_at: Instant,
    pub results: Vec<RecoveryTestResult>,
}

pub struct ErrorInjector {
    config: ErrorInjectionConfig,
}

impl ErrorInjector {
    pub fn new(config: ErrorInjectionConfig) -> Self {
        Self { config }
    }
    
    pub async fn inject(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.config.injection_timing {
            InjectionTiming::Immediate => self.inject_immediately().await,
            InjectionTiming::Delayed(delay) => {
                tokio::time::sleep(delay).await;
                self.inject_immediately().await
            },
            InjectionTiming::Random => {
                let delay = Duration::from_millis(rand::random::<u64>() % 5000);
                tokio::time::sleep(delay).await;
                self.inject_immediately().await
            },
            InjectionTiming::ConditionBased(_) => {
                // 条件注入逻辑
                self.inject_immediately().await
            }
        }
    }
    
    async fn inject_immediately(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.config.injection_type {
            ErrorInjectionType::NetworkFailure => {
                // 模拟网络故障
                info!("注入网络故障");
                Ok(())
            },
            ErrorInjectionType::DatabaseTimeout => {
                // 模拟数据库超时
                info!("注入数据库超时");
                Ok(())
            },
            ErrorInjectionType::ServiceUnavailable => {
                // 模拟服务不可用
                info!("注入服务不可用");
                Ok(())
            },
            ErrorInjectionType::ResourceExhaustion => {
                // 模拟资源耗尽
                info!("注入资源耗尽");
                Ok(())
            },
            ErrorInjectionType::ConfigurationError => {
                // 模拟配置错误
                info!("注入配置错误");
                Ok(())
            },
            ErrorInjectionType::Custom(ref name) => {
                info!("注入自定义错误: {}", name);
                Ok(())
            }
        }
    }
}
```

## 最佳实践

### 恢复模式选择指南

#### 不同场景的恢复模式选择

| 错误类型 | 推荐恢复模式 | 适用条件 | 注意事项 |
|----------|--------------|----------|----------|
| **临时网络故障** | 指数退避重试 | 网络超时、连接重置 | 设置最大重试次数，避免无限重试 |
| **服务过载** | 断路器 + 降级 | 高并发、资源受限 | 合理配置断路器阈值 |
| **数据库故障** | 故障转移 + 重试 | 主从数据库架构 | 确保数据一致性 |
| **缓存失效** | 缓存降级 | 高频访问数据 | 提供备用数据源 |
| **外部API故障** | 超时控制 + 降级 | 第三方服务依赖 | 实现优雅降级 |
| **资源不足** | 舱壁隔离 + 降级 | 系统资源限制 | 防止雪崩效应 |

#### 恢复策略配置最佳实践

```rust
pub struct RecoveryStrategyConfig {
    pub retry_config: RetryConfig,
    pub circuit_breaker_config: CircuitBreakerConfig,
    pub degradation_config: DegradationConfig,
    pub timeout_config: TimeoutConfig,
    pub bulkhead_config: BulkheadConfig,
}

impl Default for RecoveryStrategyConfig {
    fn default() -> Self {
        Self {
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                multiplier: 2.0,
                jitter: true,
            },
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 5,
                recovery_timeout: Duration::from_secs(60),
                expected_exception_predicate: None,
                ring_buffer_size_in_half_open_state: 10,
            },
            degradation_config: DegradationConfig {
                auto_degradation_enabled: true,
                degradation_threshold: 0.1,
                recovery_threshold: 0.05,
                monitoring_interval: Duration::from_secs(30),
            },
            timeout_config: TimeoutConfig {
                connect_timeout: Duration::from_secs(5),
                read_timeout: Duration::from_secs(30),
                write_timeout: Duration::from_secs(30),
                total_timeout: Duration::from_secs(60),
            },
            bulkhead_config: BulkheadConfig {
                max_concurrent: 100,
                timeout: Duration::from_secs(30),
                circuit_breaker_config: CircuitBreakerConfig::default(),
            },
        }
    }
}

impl RecoveryStrategyConfig {
    pub fn for_database() -> Self {
        Self {
            retry_config: RetryConfig {
                max_attempts: 2,
                base_delay: Duration::from_millis(50),
                max_delay: Duration::from_secs(5),
                multiplier: 1.5,
                jitter: false,
            },
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 3,
                recovery_timeout: Duration::from_secs(30),
                expected_exception_predicate: None,
                ring_buffer_size_in_half_open_state: 5,
            },
            ..Default::default()
        }
    }
    
    pub fn for_external_api() -> Self {
        Self {
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(30),
                multiplier: 2.5,
                jitter: true,
            },
            timeout_config: TimeoutConfig {
                connect_timeout: Duration::from_secs(3),
                read_timeout: Duration::from_secs(10),
                write_timeout: Duration::from_secs(10),
                total_timeout: Duration::from_secs(15),
            },
            ..Default::default()
        }
    }
    
    pub fn for_cache() -> Self {
        Self {
            retry_config: RetryConfig {
                max_attempts: 1,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                multiplier: 1.0,
                jitter: false,
            },
            degradation_config: DegradationConfig {
                auto_degradation_enabled: true,
                degradation_threshold: 0.05,
                recovery_threshold: 0.02,
                monitoring_interval: Duration::from_secs(10),
            },
            ..Default::default()
        }
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.retry_config.max_attempts == 0 {
            return Err("最大重试次数必须大于0".to_string());
        }
        
        if self.retry_config.base_delay > self.retry_config.max_delay {
            return Err("基础延迟不能大于最大延迟".to_string());
        }
        
        if self.circuit_breaker_config.failure_threshold == 0 {
            return Err("断路器失败阈值必须大于0".to_string());
        }
        
        if self.degradation_config.degradation_threshold <= self.degradation_config.recovery_threshold {
            return Err("降级阈值必须大于恢复阈值".to_string());
        }
        
        Ok(())
    }
}
```

### 监控和告警

#### 恢复模式监控指标

```rust
use prometheus::{Counter, Histogram, Gauge, Registry};

pub struct RecoveryMetrics {
    // 重试指标
    pub retry_attempts_total: Counter,
    pub retry_success_total: Counter,
    pub retry_failure_total: Counter,
    pub retry_latency: Histogram,
    
    // 断路器指标
    pub circuit_breaker_state: Gauge,
    pub circuit_breaker_failures_total: Counter,
    pub circuit_breaker_successes_total: Counter,
    pub circuit_breaker_state_changes_total: Counter,
    
    // 降级指标
    pub degradation_events_total: Counter,
    pub degradation_mode_duration: Histogram,
    pub degradation_success_rate: Gauge,
    
    // 超时指标
    pub timeout_events_total: Counter,
    pub timeout_duration: Histogram,
    
    // 舱壁指标
    pub bulkhead_rejections_total: Counter,
    pub bulkhead_active_connections: Gauge,
    pub bulkhead_queue_size: Gauge,
}

impl RecoveryMetrics {
    pub fn new(registry: &Registry) -> Self {
        let retry_attempts_total = Counter::new(
            "recovery_retry_attempts_total",
            "Total number of retry attempts"
        ).expect("Failed to create retry_attempts_total counter");

        let retry_success_total = Counter::new(
            "recovery_retry_success_total",
            "Total number of successful retries"
        ).expect("Failed to create retry_success_total counter");

        let retry_failure_total = Counter::new(
            "recovery_retry_failure_total",
            "Total number of failed retries"
        ).expect("Failed to create retry_failure_total counter");

        let retry_latency = Histogram::new(
            "recovery_retry_latency_seconds",
            "Time spent on retry attempts"
        ).expect("Failed to create retry_latency histogram");

        let circuit_breaker_state = Gauge::new(
            "recovery_circuit_breaker_state",
            "Current circuit breaker state (0=Closed, 1=Open, 2=Half-Open)"
        ).expect("Failed to create circuit_breaker_state gauge");

        let circuit_breaker_failures_total = Counter::new(
            "recovery_circuit_breaker_failures_total",
            "Total number of circuit breaker failures"
        ).expect("Failed to create circuit_breaker_failures_total counter");

        let circuit_breaker_successes_total = Counter::new(
            "recovery_circuit_breaker_successes_total",
            "Total number of circuit breaker successes"
        ).expect("Failed to create circuit_breaker_successes_total counter");

        let circuit_breaker_state_changes_total = Counter::new(
            "recovery_circuit_breaker_state_changes_total",
            "Total number of circuit breaker state changes"
        ).expect("Failed to create circuit_breaker_state_changes_total counter");

        let degradation_events_total = Counter::new(
            "recovery_degradation_events_total",
            "Total number of degradation events"
        ).expect("Failed to create degradation_events_total counter");

        let degradation_mode_duration = Histogram::new(
            "recovery_degradation_mode_duration_seconds",
            "Time spent in degradation mode"
        ).expect("Failed to create degradation_mode_duration histogram");

        let degradation_success_rate = Gauge::new(
            "recovery_degradation_success_rate",
            "Success rate during degradation mode"
        ).expect("Failed to create degradation_success_rate gauge");

        let timeout_events_total = Counter::new(
            "recovery_timeout_events_total",
            "Total number of timeout events"
        ).expect("Failed to create timeout_events_total counter");

        let timeout_duration = Histogram::new(
            "recovery_timeout_duration_seconds",
            "Timeout duration"
        ).expect("Failed to create timeout_duration histogram");

        let bulkhead_rejections_total = Counter::new(
            "recovery_bulkhead_rejections_total",
            "Total number of bulkhead rejections"
        ).expect("Failed to create bulkhead_rejections_total counter");

        let bulkhead_active_connections = Gauge::new(
            "recovery_bulkhead_active_connections",
            "Number of active bulkhead connections"
        ).expect("Failed to create bulkhead_active_connections gauge");

        let bulkhead_queue_size = Gauge::new(
            "recovery_bulkhead_queue_size",
            "Current bulkhead queue size"
        ).expect("Failed to create bulkhead_queue_size gauge");

        // 注册所有指标
        registry.register(Box::new(retry_attempts_total.clone())).unwrap();
        registry.register(Box::new(retry_success_total.clone())).unwrap();
        registry.register(Box::new(retry_failure_total.clone())).unwrap();
        registry.register(Box::new(retry_latency.clone())).unwrap();
        registry.register(Box::new(circuit_breaker_state.clone())).unwrap();
        registry.register(Box::new(circuit_breaker_failures_total.clone())).unwrap();
        registry.register(Box::new(circuit_breaker_successes_total.clone())).unwrap();
        registry.register(Box::new(circuit_breaker_state_changes_total.clone())).unwrap();
        registry.register(Box::new(degradation_events_total.clone())).unwrap();
        registry.register(Box::new(degradation_mode_duration.clone())).unwrap();
        registry.register(Box::new(degradation_success_rate.clone())).unwrap();
        registry.register(Box::new(timeout_events_total.clone())).unwrap();
        registry.register(Box::new(timeout_duration.clone())).unwrap();
        registry.register(Box::new(bulkhead_rejections_total.clone())).unwrap();
        registry.register(Box::new(bulkhead_active_connections.clone())).unwrap();
        registry.register(Box::new(bulkhead_queue_size.clone())).unwrap();

        Self {
            retry_attempts_total,
            retry_success_total,
            retry_failure_total,
            retry_latency,
            circuit_breaker_state,
            circuit_breaker_failures_total,
            circuit_breaker_successes_total,
            circuit_breaker_state_changes_total,
            degradation_events_total,
            degradation_mode_duration,
            degradation_success_rate,
            timeout_events_total,
            timeout_duration,
            bulkhead_rejections_total,
            bulkhead_active_connections,
            bulkhead_queue_size,
        }
    }
    
    pub fn record_retry_attempt(&self, success: bool, latency: f64) {
        self.retry_attempts_total.inc();
        if success {
            self.retry_success_total.inc();
        } else {
            self.retry_failure_total.inc();
        }
        self.retry_latency.observe(latency);
    }
    
    pub fn record_circuit_breaker_state(&self, state: u8) {
        self.circuit_breaker_state.set(state as f64);
    }
    
    pub fn record_circuit_breaker_event(&self, success: bool) {
        if success {
            self.circuit_breaker_successes_total.inc();
        } else {
            self.circuit_breaker_failures_total.inc();
        }
        self.circuit_breaker_state_changes_total.inc();
    }
    
    pub fn record_degradation_event(&self, duration: f64) {
        self.degradation_events_total.inc();
        self.degradation_mode_duration.observe(duration);
    }
    
    pub fn update_degradation_success_rate(&self, rate: f64) {
        self.degradation_success_rate.set(rate);
    }
    
    pub fn record_timeout_event(&self, duration: f64) {
        self.timeout_events_total.inc();
        self.timeout_duration.observe(duration);
    }
    
    pub fn record_bulkhead_rejection(&self) {
        self.bulkhead_rejections_total.inc();
    }
    
    pub fn update_bulkhead_metrics(&self, active_connections: u64, queue_size: u64) {
        self.bulkhead_active_connections.set(active_connections as f64);
        self.bulkhead_queue_size.set(queue_size as f64);
    }
}
```

### 总结

错误恢复模式是构建高可用、高可靠性系统的关键技术。通过合理组合重试、断路器、降级、超时控制、舱壁隔离等模式，可以显著提升系统的容错能力和用户体验。

#### 关键要点

1. **模式组合**: 单一恢复模式往往不够，需要根据业务场景组合使用多种模式
2. **配置调优**: 恢复模式的参数配置需要根据实际业务特点和系统负载进行调整
3. **监控告警**: 建立完善的监控体系，及时发现恢复模式触发和效果
4. **测试验证**: 定期进行恢复测试，验证恢复策略的有效性
5. **持续优化**: 根据运行数据和故障分析，持续优化恢复策略

#### 实施建议

1. **渐进式实施**: 从关键服务开始，逐步推广到全系统
2. **文档化**: 详细记录恢复策略的配置、使用场景和操作流程
3. **培训**: 对运维和开发人员进行恢复模式相关培训
4. **演练**: 定期进行故障演练，提高团队应急响应能力
5. **自动化**: 尽可能实现恢复模式的自动化配置和调优

通过合理应用这些恢复模式，可以构建出更加健壮、可靠的错误处理系统，为用户提供更好的服务体验。

## 相关文档

- [错误分类体系](./01-error-classification.md) - 不同类型错误的分类方法
- [错误处理策略](./02-handling-strategies.md) - 不同类型错误的处理策略
- [错误归集机制](./03-error-aggregation.md) - 跨层错误转换方法
- [错误处理层级](./04-handling-layers.md) - 分层处理模型
- [错误日志规范](./05-logging-standards.md) - 标准化错误日志字段格式
- [最佳实践指南](./06-best-practices.md) - 综合最佳实践指南
- [监控指标体系](./07-monitoring-metrics.md) - 关键指标定义和告警阈值