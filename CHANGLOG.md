# 更新日志 (CHANGELOG)

## 版本 0.5.5 (2024-9-20)

### ✨ 新增与优化
- **结构化错误构建器**：`StructError::builder` 支持链式设置 detail、position、context，并与 `context_ref` 共享上下文栈，避免重复分配。
- **上下文作用域 Guard**：新增 `OperationContext::scope()` / `scoped_success()` 与 `OperationScope`，在成功路径自动标记 `mark_suc()`，降低遗漏风险。
- **错误转换性能**：`ErrorOwe::owe_*` 系列仅序列化一次底层错误消息，减少 `to_string()` 开销；上下文内部改用 `Arc<Vec<_>>` 共用堆栈。
- **示例更新**：`examples/order_case.rs`、`logging_example.rs` 演示 builder 与作用域 guard 的推荐写法。

### 📚 文档更新
- `docs/tutorial.md` 增补 builder/OperationScope 的实践示例。
- `docs/error-handling/05-logging-standards.md`、`06-best-practices.md` 补充 guard 与构建器的使用建议。
- `docs/README.md` 在亮点更新中突出上述能力。

### 🧪 兼容性
- 接口向后兼容；若手动调用 `mark_suc()`，仍可与作用域 guard 共存。

## [v0.5.0] - 2025-08-25

### 新增功能
- **日志支持**: 为错误上下文添加了完整的日志记录功能，包括 `info`、`debug`、`warn`、`error`、`trace` 方法
- **自动日志记录**: 新增 `with_exit_log` 和 `mark_suc` 方法，支持在对象销毁时自动记录日志
- **PathContext包装器**: 添加了 `PathContext<V: AsRef<Path>` 包装类型，用于区分路径和字符串类型

### 重大变更
- **结构体重命名**: 将内部结构体从 `WithContext` 重命名为 `OperationContext`，提高代码清晰度
- **ContextTake trait重构**: 解决了 trait 实现冲突问题，移除了 `&PathBuf` 的特定实现，改为使用 `PathContext` 包装器

### 依赖更新
- 升级 `thiserror` 从 2.0.12 到 2.0.16
- 升级 `serde_json` 从 1.0.140 到 1.0.143
- 添加 `log` 和 `env_logger` 依赖以支持日志功能

### 文档和示例
- 新增 `LOGGING.md` 文档，详细说明日志功能的使用方法
- 添加 `examples/logging_example.rs` 示例文件
- 更新现有示例以适配新的 API
- 完善了错误处理的最佳实践文档

### 测试改进
- 新增了完整的 `ContextRecord` trait 测试用例，覆盖字符串类型、数字类型、路径类型等各种场景
- 改进了测试覆盖率，确保所有新功能的稳定性

### CI/CD 优化
- 升级 GitHub Actions 从 `actions/checkout@v4` 到 `v5`
- 升级 `rustsec/audit-check` 从 1.4.1 到 2.0.0
- 改进了 CI 流程中的覆盖率处理

### 错误修复
- 修复了 ContextTake trait 的实现冲突问题 (E0119)
- 修复了路径类型处理的编译错误
- 改进了错误消息的格式化和显示

## [v0.4.0] - 2025-08-24

### 初始发布
- 基础错误处理框架
- 支持结构化错误类型
- 提供错误上下文管理
- 基本的序列化和反序列化支持
