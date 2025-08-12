fn place_order() -> Result<Order> {
    let mut ctx = WithContext::want("place_order");
    ctx.with(order_txt);

    parse_order()
        .want("解析订单")
        .with(&ctx)
        .owe_biz()?
}
```

### 重试机制示例
```rust
retry!(
    max_retries = 3,
    backoff = Exponential::new(Duration::from_secs(1)),
    condition = |e: &OrderError| e.is_retryable()
) {
    Self::place_order(user_id, amount, order_txt)
}
```

## 设计闭环

该设计文档与示例代码形成完整闭环，通过：

1. **领域错误定义** → 对应[错误分类体系](./01-error-classification.md)
2. **错误转换实现** → 对应[错误归集机制](./03-error-aggregation.md)
3. **WithContext 使用** → 对应[上下文规范](./05-logging-standards.md)
4. **print_error 展示** → 对应[日志标准](./05-logging-standards.md)

实现从设计到落地的完整错误处理方案。

## 相关链接

- [主项目 README](../../README.md)
- [示例代码](../../examples/)
- [API 文档](../../src/)
- [原始设计文档](../../error-design.md)