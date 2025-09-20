//! 展示WithContext日志记录功能的示例
//! 此示例展示了如何在无错误情况下使用WithContext记录有价值的上下文信息

use orion_error::{ContextRecord, OperationContext};

fn main() {
    // 初始化日志系统（实际项目中需要在main函数开始处初始化）

    env_logger::init();
    println!("=== WithContext日志记录示例 ===\n");

    // 示例1: 订单处理流程中的日志记录
    process_order("order_123", 100.0, "customer_456");

    println!("\n=== 成功场景下的日志价值 ===");

    // 示例2: 成功场景下的详细日志记录
    successful_operation();
}

fn process_order(order_id: &str, amount: f64, customer_id: &str) {
    // 创建WithContext对象来收集上下文信息
    let mut ctx = OperationContext::want("process_order").with_auto_log();
    ctx.record("order_id", order_id);
    {
        let mut scope = ctx.scoped_success();
        scope.record("order_id", order_id);
        scope.record("amount", amount.to_string());
        scope.record("customer_id", customer_id);

        scope.info("开始处理订单");

        let validation_result = validate_order(amount);

        scope.record("validation_result", validation_result.to_string());
        scope.debug("订单验证完成");

        if validation_result {
            scope.info("订单处理成功");
            println!("订单 {order_id} 处理成功");
        } else {
            scope.error("订单验证失败");
            scope.mark_failure();
            println!("订单 {order_id} 验证失败");
        }
    }
}

fn validate_order(amount: f64) -> bool {
    // 简单的验证逻辑
    amount > 0.0 && amount <= 10000.0
}

fn successful_operation() {
    // 展示在成功操作中如何记录有价值的上下文信息
    let mut ctx = OperationContext::want("data_processing");
    {
        let mut scope = ctx.scoped_success();
        scope.record("batch_size", "1000");
        scope.record("processor", "worker_1");
        scope.record("start_time", "2024-01-01T10:00:00Z");

        scope.info("开始数据处理");

        for i in 0..5 {
            scope.record("current_item", i.to_string());
            scope.debug("处理数据项");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        scope.record("end_time", "2024-01-01T10:05:00Z");
        scope.record("items_processed", "5");

        scope.info("数据处理完成");
    }

    println!("数据处理操作完成，记录了完整的上下文信息");
}

// 运行此示例:
// 启用log功能: cargo run --example logging_example --features log
// 启用tracing功能: cargo run --example logging_example --features tracing
// 无日志功能: cargo run --example logging_example
