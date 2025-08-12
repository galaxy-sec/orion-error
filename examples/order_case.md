1. **领域错误定义**：
   - 使用`OrderError`枚举定义业务相关的错误类型
   - 实现`DomainReason`标记trait和`Display`用于错误展示

2. **错误处理模式**：
   - 使用`WithContext`添加上下文：
   ```rust
   .with(WithContext::want("解析订单"))
   ```
   - 使用`owe`转换底层错误：
   ```rust
   .map_err(|e| e.owe(OrderError::UserNotFound))
   ```
   - 使用`owe_sys`转换系统级错误：
   ```rust
   .map_err(|e| e.owe_sys())
   ```

3. **错误信息增强**：
   - 添加详细错误信息：
   ```rust
   .with_detail(format!("当前余额：{balance}，需要：{amount}"))
   ```

4. **错误展示**：
   - 实现`print_error`函数展示完整的错误链：
   ```rust
   [错误代码 103] sys error > Storage capacity exceeded
   Target:保存订单
   error context:
       "解析订单"
       "验证资金"
