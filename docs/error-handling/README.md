fn place_order() -> Result<Order, StructError<UvsReason>> {
    let mut ctx = OperationContext::want("place_order");
    ctx.with("order_text", order_txt);

    parse_order()
        .with_context(OperationContext::want("解析订单"))
        .map_err(|e| StructError::from(UvsReason::business_error("订单解析失败")))?
}
```

### 错误处理策略示例
```rust
fn process_with_retry<T>(operation: impl Fn() -> Result<T, StructError<UvsReason>>) -> Result<T, StructError<UvsReason>> {
    match operation() {
        Ok(result) => Ok(result),
        Err(error) => {
            // 根据错误类型选择处理策略
            match error.reason() {
                UvsReason::NetworkError(_) => {
                    // 网络错误使用重试策略
                    // 实际的重试逻辑需要根据具体场景实现
                    Err(error)
                }
                UvsReason::ValidationError(_) => {
                    // 验证错误直接抛出
                    Err(error)
                }
                _ => Err(error),
            }
        }
    }
}
```

## 设计闭环

该设计文档与示例代码形成完整闭环，通过：

1. **UvsReason 错误分类** → 对应[错误分类体系](./01-error-classification.md)
2. **StructError 错误结构** → 对应[错误处理策略](./02-handling-strategies.md)
3. **OperationContext 上下文管理** → 对应[错误处理层次](./04-handling-layers.md)
4. **ErrStrategy 处理策略** → 对应[错误处理策略](./02-handling-strategies.md)

实现从设计到落地的完整错误处理方案。

## 相关链接

- [主项目 README](../../README.md)
- [示例代码](../../examples/)
- [API 文档](../../src/)
- [原始设计文档](../../error-design.md)