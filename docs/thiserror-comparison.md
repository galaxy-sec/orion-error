# 与 thiserror 的差异与协作指南

本页系统对比 `thiserror` 与本库（orion-error），并给出协作使用的最佳实践。

## 定位与目标
- thiserror：专注“定义错误类型”的派生宏，自动实现 `Error`/`Display`，零运行时开销，适合轻量场景。
- orion-error：面向“错误治理”的结构化库，提供统一分类（`UvsReason`）、错误码（`ErrorCode`）、上下文（`OperationContext`）、转换与传播策略（`ErrorOwe`/`ErrorConv`），适合中大型工程。

## 能力对比（概览）
| 能力 | thiserror | orion-error | 组合使用 |
|---|---|---|---|
| 错误类型定义 | 宏派生定义优雅 | 接入 `DomainReason` 协议 | 用 thiserror 定义，orion-error 承载治理 |
| Display/Error 实现 | 自动 | 支持，但侧重结构化展示 | thiserror 负责显示，orion-error 负责治理 |
| 错误码与分类 | 无 | `ErrorCode` + `UvsReason` 分层 | 由领域错误转发 `UvsReason` 码 |
| 上下文与链路 | 无 | `WithContext`/`OperationContext` 堆栈 | 在关键路径记录上下文 |
| 转换与传播 | 无 | `.owe_*()`/`err_conv()` | 将 `Result<T,E:Display>` 归一化 |
| 序列化/观测 | 无 | 可序列化、`category_name()` 等 | 统一输出指标/日志/HTTP |

## 推荐协作模式
- 用 thiserror 定义领域枚举；实现 `From<UvsReason>`、`ErrorCode`，自动满足 `DomainReason`；在业务中用 `.owe_*()` 归一转换，叠加 `WithContext`。

```rust
use derive_more::From;
use thiserror::Error;
use orion_error::{StructError, ErrorOwe, ErrorCode, UvsReason, DomainReason};

#[derive(Debug, Error, From, serde::Serialize, PartialEq)]
enum AppError {
    #[error("{0}")]
    Uvs(UvsReason),
    #[error("parse failed")]
    Parse,
}
impl ErrorCode for AppError { fn error_code(&self)->i32 { match self { AppError::Uvs(r)=>r.error_code(), AppError::Parse=>100 } } }
impl DomainReason for AppError {}

fn handle() -> Result<(), StructError<AppError>> {
    read_io().owe_sys()?;          // 系统类错误
    parse_cfg().owe_validation()?; // 校验类错误
    Ok(())
}
```

## 何时选择
- 仅需定义错误并优雅显示：thiserror + anyhow/eyre 足够。
- 需要跨模块/服务统一治理（错误码、分类、可观测、策略化）：orion-error，并与 thiserror 组合最佳。

## 性能与依赖
- thiserror：编译期派生，零运行时成本。
- orion-error：`StructError` 保存 detail/position/context，带来可控的小幅对象与格式化开销；建议仅在需要处添加上下文。
- 可选特性建议：将日志/序列化做成 feature（如 `log`/`tracing`/`serde`），在生产按需启用。

## 实践建议
- 统一错误码：服务边界对外仅暴露 `ErrorCode` 与 `category_name()`。
- Web 映射：将 `UvsReason` 分类映射为 HTTP 状态码；中间件统一处理。
- 重试与告警：基于 `is_retryable()` 与 `is_high_severity()` 制定自动化策略。

如需更完整示例，请查看 `examples/` 与顶层 README 的“与 thiserror 的差异与配合”。
