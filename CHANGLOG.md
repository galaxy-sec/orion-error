# 更新日志

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