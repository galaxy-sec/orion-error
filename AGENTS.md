# Repository Guidelines

## 项目结构与模块组织
- `src/`: 库代码。`core/`（error、reason、context、domain、universal）；`traits/` 为转换与助手；`lib.rs` 统一导出；`testcase.rs` 含测试辅助。
- `tests/`: 集成测试（如 `tests/test_error_owe.rs`）。
- `examples/`: 可运行示例（如 `order_case`、`logging_example`）。
- `docs/`: 文档与设计说明（教程、错误分类、日志等）。
- `.github/workflows/`: CI（fmt、clippy、tests、coverage、audit）。
- `tasks/`: 任务记录与 TODO（非代码）。

## 构建、测试与本地开发
- `cargo build`: 构建库。
- `cargo test --all-features -- --test-threads=1`: 运行测试（与 CI 一致）。
- `cargo fmt --all` / `cargo fmt --all -- --check`: 格式化 / 校验格式。
- `cargo clippy --all-targets --all-features -- -D warnings`: Lint 并将告警视为错误。
- 示例：`cargo run --example order_case`；日志特性：`cargo run --example logging_example --features log` 或 `--features tracing`。
- 覆盖率（可选）：`cargo llvm-cov --all-features --workspace --html`。

## 代码风格与命名规范
- Rust 2021；遵循 rustfmt 默认；CI 要求 clippy 零告警。
- 命名：模块/函数用 `snake_case`；类型/枚举/trait 用 `PascalCase`。
- 倡导安全代码；库中避免 `panic!`；合理使用 `thiserror`、`derive_more`、`serde` 派生。
- 公共 API 通过 `lib.rs` 统一导出，保持边界清晰、粒度小。

## 测试指南
- 单元测试靠近实现；集成测试放在 `tests/`（命名 `test_*.rs`）。
- 关注语义：断言 `ErrorCode`、`StructError`、`detail()/context()` 等行为。
- 保持示例可编译运行，必要时新增 `examples/` 用例。

## Commit 与 Pull Request
- 提交信息简明祈使式（示例：`fmt`、`cargo clippy`），聚合相关改动。
- PR 需含：变更摘要与动机、关联 issue、API 变更前后对照、同步更新文档/示例；本地通过 `fmt`/`clippy`/`test`。必要时附覆盖率或截图。

## 架构概览
- 核心：`StructError<R>` + `UvsReason`（分层错误分类，支持 `is_retryable`/`is_high_severity`/`category_name`）。
- 上下文：`WithContext`/`OperationContext` 富化错误；转换与处理位于 `traits::*`（如 `ErrorOwe`、`ErrorConv`）。
