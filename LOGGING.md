# WithContext 日志记录功能

orion-error 现在提供了强大的日志记录功能，允许在无错误情况下也能记录有价值的上下文信息。

## 功能特性

- ✅ 支持多种日志库：log 和 tracing
- ✅ 在成功和失败场景下都能记录上下文信息
- ✅ 可选的依赖，按需启用
- ✅ 丰富的日志级别支持
- ✅ 自动格式化上下文信息

## 启用日志功能

在 `Cargo.toml` 中添加相应的特性：

```toml
[dependencies]
orion-error = { version = "0.4", features = ["log"] }
# 或者
orion-error = { version = "0.4", features = ["tracing"] }
```

## 使用方法

### 基本日志记录

```rust
use orion_error::WithContext;

let mut ctx = WithContext::want("order_processing");
ctx.with("order_id", "123");
ctx.with("amount", "100.0");

// 记录不同级别的日志
ctx.log_info("开始处理订单");
ctx.log_debug("订单详情");
ctx.log_warn("警告信息");
ctx.log_error("错误信息");
ctx.log_trace("详细追踪");
```

### 在业务逻辑中使用

```rust
fn process_order(order: Order) -> Result<(), Error> {
    let mut ctx = WithContext::want("process_order");
    ctx.with("order_id", order.id.to_string());
    
    // 记录处理开始
    ctx.log_info("开始订单处理");
    
    // 业务逻辑...
    
    if success {
        // 记录成功信息
        ctx.log_info("订单处理成功");
        Ok(())
    } else {
        // 记录错误信息
        ctx.log_error("订单处理失败");
        Err(Error::new())
    }
}
```

### 输出示例

启用日志功能后，输出格式如下：

```
INFO 开始处理订单 - target: order_processing, context: {order_id: 123, amount: 100.0}
DEBUG 订单详情 - target: order_processing, context: {order_id: 123, amount: 100.0}
```

## 配置日志库

### 使用 log 库

```rust
fn main() {
    env_logger::init();
    // 你的代码...
}
```

### 使用 tracing 库

```rust
fn main() {
    tracing_subscriber::fmt::init();
    // 你的代码...
}
```

## 性能考虑

- 日志记录只在启用相应特性时编译
- 在成功路径上，日志调用有轻微性能开销
- 上下文格式化只在日志记录时发生
- 适合用于调试、监控和审计场景

## 示例

查看 `examples/logging_example.rs` 获取完整示例：

```bash
# 使用 log 功能运行示例
cargo run --example logging_example --features log

# 使用 tracing 功能运行示例  
cargo run --example logging_example --features tracing

# 无日志功能运行示例
cargo run --example logging_example
```

## 最佳实践

1. **适度使用**: 在关键业务路径记录有价值的信息
2. **上下文丰富**: 利用 `with()` 方法添加相关上下文
3. **级别选择**: 根据信息重要性选择合适的日志级别
4. **性能敏感**: 在高性能场景谨慎使用调试和追踪级别

## 优势

- 🎯 **零成本成功路径**: 错误处理逻辑只在错误时执行
- 📝 **丰富上下文**: 即使成功也能记录完整的操作上下文
- 🔧 **灵活配置**: 支持多种日志库和级别
- 🚀 **按需启用**: 不影响不使用日志功能的用户

通过这个功能，你可以在无错误情况下也能获得有价值的操作日志，大大提升了系统的可观测性和调试能力。