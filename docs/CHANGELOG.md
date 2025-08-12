# 更新日志 (CHANGELOG)

## 版本 0.4.0 (2024-01-XX)

### 🎯 主要变更
- **全新错误分类体系**: 重新设计了 `UvsReason` 的分类架构，采用三层分层设计
- **错误码重新分配**: 按错误层次重新分配错误码范围 (100-399)
- **新增实用功能**: 内置错误重试性判断、严重程度评估、错误类别识别
- **修复实现错误**: 修正了网络错误错误映射到业务错误的问题
- **命名统一**: 统一了错误类型的命名规范

### 🔄 重大变更 (Breaking Changes)

#### 错误类型重构

**移除的错误类型:**
- `UvsReason::LogicError` → 使用 `ValidationError` 替代
- `UvsReason::BizError` → 使用 `BusinessError` 替代
- `UvsReason::RuleError` → 使用 `ValidationError` 替代
- `UvsReason::PrivacyError` → 使用 `PermissionError` 替代

**重命名的错误类型:**
- `UvsReason::Timeout` → `UvsReason::TimeoutError`
- `UvsReason::ConfError` → `UvsReason::ConfigError`

**新增的错误类型:**
- `UvsReason::ValidationError` (代码 100) - 输入验证错误
- `UvsReason::NotFoundError` (代码 102) - 资源不存在错误
- `UvsReason::PermissionError` (代码 103) - 权限认证错误
- `UvsReason::ExternalError` (代码 301) - 第三方服务错误

#### 错误码变更
```
旧代码 -> 新代码 -> 错误类型
100   -> 100    -> ValidationError
101   -> 101    -> BusinessError
102   -> 200    -> DataError
103   -> 201    -> SystemError
104   -> 203    -> ResourceError
105   -> 300    -> ConfigError
106   -> (移除) -> 合并到 ValidationError
107   -> (移除) -> 合并到 PermissionError
108   -> 202    -> NetworkError
109   -> 204    -> TimeoutError
```

#### Trait 变更

**移除的 Trait:**
- `UvsRuleFrom` → 使用 `UvsValidationFrom` 替代
- `UvsLogicFrom` → 使用 `UvsValidationFrom` 替代

**新增的 Trait:**
- `UvsValidationFrom` - 验证错误转换
- `UvsNotFoundFrom` - 资源不存在错误转换
- `UvsPermissionFrom` - 权限错误转换
- `UvsExternalFrom` - 外部服务错误转换

**修改的 Trait 方法:**
- `ErrorOwe::owe_rule()` → `ErrorOwe::owe_validation()`

#### 构造函数变更
```rust
// 旧方式
let error = UvsReason::BizError("msg".into());
let error = UvsReason::LogicError("msg".into());
let error = UvsReason::Timeout("msg".into());

// 新方式
let error = UvsReason::business_error("msg");
let error = UvsReason::validation_error("msg");
let error = UvsReason::timeout_error("msg");
```

### ✨ 新增功能

#### 错误分析功能
```rust
let error = UvsReason::network_error("connection failed");

// 检查是否可重试
assert!(error.is_retryable());

// 检查严重程度
assert!(!error.is_high_severity());

// 获取错误类别
assert_eq!(error.category_name(), "network");
```

#### 重试性判断
内置的重试性判断逻辑：
- **可重试错误**: NetworkError, TimeoutError, ResourceError, SystemError, ExternalError
- **不可重试错误**: ValidationError, BusinessError, NotFoundError, PermissionError, ConfigError, DataError

#### 严重程度判断
高严重性错误标记：
- `SystemError` - 系统级故障
- `ResourceError` - 资源耗尽
- `ConfigError` - 配置问题

### 🐛 修复的问题
- 修复了 `UvsNetFrom` 错误地将网络错误映射为业务错误的问题
- 修复了错误类型命名不一致的问题
- 统一了错误消息格式

### 📚 文档更新
- 完整重写了 README.md
- 新增了错误分类指南 (`docs/error_classification.md`)
- 新增了 API 参考文档 (`docs/api_reference.md`)
- 新增了使用教程 (`docs/tutorial.md`)
- 新增了更新日志 (`docs/CHANGELOG.md`)

## 版本 0.2.0 (2023-XX-XX)

### 🎯 主要变更
- 初始版本发布
- 基础错误处理框架
- 结构化错误类型支持
- 错误上下文跟踪

### ✨ 功能特性
- `StructError<T>` 结构化错误类型
- `WithContext` 错误上下文支持
- 基本的错误转换 trait
- 错误码系统

### 📋 已知限制
- 错误分类体系较为简单
- 缺少重试性和严重程度判断
- 错误类型边界不清晰

## 版本历史
- **0.1.0** - 初始概念验证版本
- **0.2.0** - 第一个稳定版本
- **0.3.0** - 重大重构，分层错误分类体系

## 升级指南

### 从 0.2.x 升级到 0.3.0

#### 步骤 1: 更新依赖
```toml
[dependencies]
# 旧版本
orion-error = "0.2"

# 新版本
orion-error = "0.3"
```

#### 步骤 2: 更新错误类型
```rust
// 旧代码
use orion_error::UvsReason;

let error = UvsReason::BizError("business logic failed".into());
let error = UvsReason::LogicError("validation failed".into());
let error = UvsReason::Timeout("timeout occurred".into());

// 新代码
use orion_error::UvsReason;

let error = UvsReason::business_error("business logic failed");
let error = UvsReason::validation_error("validation failed");
let error = UvsReason::timeout_error("timeout occurred");
```

#### 步骤 3: 更新 Trait 使用
```rust
// 旧代码
let result = some_operation().owe_rule()?;

// 新代码
let result = some_operation().owe_validation()?;
```

#### 步骤 4: 更新错误码依赖
```rust
// 旧代码 - 硬编码错误码
if error.error_code() == 101 {
    // 处理业务错误
}

// 新代码 - 使用类别或语义判断
match error.category_name() {
    "business" => { /* 处理业务错误 */ }
    "validation" => { /* 处理验证错误 */ }
    _ => { /* 其他错误 */ }
}
```

#### 步骤 5: 利用新功能
```rust
// 新增的重试逻辑
if error.is_retryable() {
    // 实现重试
}

// 新增的严重程度判断
if error.is_high_severity() {
    // 发送高优先级告警
}
```

### 升级检查清单
- [ ] 更新 Cargo.toml 依赖版本
- [ ] 替换所有已移除的错误类型
- [ ] 更新 trait 方法调用
- [ ] 检查错误码依赖的逻辑
- [ ] 利用新的错误分析功能
- [ ] 更新测试用例中的错误码期望
- [ ] 验证错误消息格式的变化

## 未来计划

### 计划中的功能 (0.4.0)
- 错误指标收集和统计
- 分布式追踪集成
- 错误恢复策略框架
- 更丰富的监控集成

### 长期路线图
- 错误模式识别
- 智能错误推荐
- 错误预防机制
- 跨语言错误处理

---

*遵循 [语义化版本 2.0.0](https://semver.org/spec/v2.0.0.html)*
