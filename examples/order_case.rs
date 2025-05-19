// 创造一个客户订单的用例,展示orion-error。
// 用例包括: order_service, order_store, order_txt 相关概念。
// 产生错语的原因可能： order_txt 格式错语， 客户帐户资金不足、 order_store 的空间不足等

use derive_more::From;
use orion_error::{
    print_error, ErrorCode, ErrorConv, ErrorOwe, ErrorWith, StructError, UvsBizFrom, UvsReason,
    UvsSysFrom, WithContext,
};
use serde::Serialize;
use std::{
    fmt::{Display, Formatter},
    sync::atomic::Ordering,
};
use thiserror::Error;

// ========== 领域错误定义 ==========
#[derive(Debug, PartialEq, Clone, Serialize, From)]
pub enum OrderReason {
    FormatError,
    InsufficientFunds,
    StorageFull,
    UserNotFound,
    Uvs(UvsReason),
}
impl ErrorCode for OrderReason {
    fn error_code(&self) -> i32 {
        match self {
            Self::Uvs(uvs_reason) => uvs_reason.error_code(),
            _ => 500,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Error, From)]
pub enum StoreReason {
    #[error("storeage full")]
    StorageFull,
    #[error("{0}")]
    Uvs(UvsReason),
}
impl ErrorCode for StoreReason {
    fn error_code(&self) -> i32 {
        match self {
            Self::Uvs(uvs_reason) => uvs_reason.error_code(),
            _ => 500,
        }
    }
}

#[derive(Debug, Error, PartialEq, Clone, Serialize, From)]
pub enum ParseReason {
    #[error("format error")]
    FormatError,
    #[error("{0}")]
    Uvs(UvsReason),
}
impl ErrorCode for ParseReason {
    fn error_code(&self) -> i32 {
        500
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Error, From)]
pub enum UserReason {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Uvs(UvsReason),
}

impl From<UserReason> for OrderReason {
    fn from(value: UserReason) -> Self {
        match value {
            UserReason::NotFound => Self::Uvs(UvsReason::from_biz("logic fail".to_string())),
            UserReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
        }
    }
}

impl From<StoreReason> for OrderReason {
    fn from(value: StoreReason) -> Self {
        match value {
            StoreReason::StorageFull => Self::Uvs(UvsReason::from_sys("sys fail".to_string())),
            StoreReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
        }
    }
}
pub type OrderError = StructError<OrderReason>;
pub type StoreError = StructError<StoreReason>;
pub type ParseError = StructError<ParseReason>;
pub type UserError = StructError<UserReason>;

impl Display for OrderReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderReason::FormatError => write!(f, "订单格式错误"),
            OrderReason::InsufficientFunds => write!(f, "账户余额不足"),
            OrderReason::StorageFull => write!(f, "订单存储空间不足"),
            OrderReason::UserNotFound => write!(f, "用户不存在"),
            OrderReason::Uvs(uvs_reason) => write!(f, "{}", uvs_reason),
        }
    }
}

// ========== 数据层 ==========
pub mod storage {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    use super::*;

    #[derive(Clone)]
    pub struct Order {
        pub user_id: u32,
        pub amount: f64,
    }

    pub static STORAGE_CAPACITY: AtomicUsize = AtomicUsize::new(2);
    static ORDERS: Mutex<Vec<Order>> = Mutex::new(Vec::new());
    pub fn save(order: Order) -> Result<(), StoreError> {
        save_db_impl(order).owe_sys()
    }

    fn save_db_impl(order: Order) -> Result<(), std::io::Error> {
        let capacity = STORAGE_CAPACITY.load(Ordering::Relaxed);
        let mut orders = ORDERS.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to lock orders mutex")
        })?;

        if orders.len() >= capacity {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "Storage capacity exceeded",
            ));
        }
        orders.push(order);
        Ok(())
    }
}

// ========== 业务逻辑层 ==========
struct OrderService;

impl OrderService {
    /// 创建订单完整流程
    pub fn place_order(
        user_id: u32,
        amount: f64,
        order_txt: &str,
    ) -> Result<storage::Order, OrderError> {
        let mut ctx = WithContext::want("place_order");
        ctx.with("order", order_txt);
        let order = Self::parse_order(order_txt, amount)
            .want("解析订单")
            .with(&ctx)
            .with(("key", "value".to_string()))
            .owe_biz()?;

        Self::validate_funds(user_id, order.amount)
            .want("验证资金")
            .with(&ctx)?;

        let order = storage::Order { user_id, amount };
        Self::save_order(order).want("保存订单")
    }

    fn parse_order(txt: &str, amount: f64) -> Result<storage::Order, ParseError> {
        if txt.is_empty() {
            return ParseError::from(ParseReason::FormatError)
                .with_detail("订单文本不能为空")
                .err();
        }

        // 模拟解析逻辑
        Ok(storage::Order {
            user_id: 123,
            amount,
        })
    }

    fn validate_funds(user_id: u32, amount: f64) -> Result<(), OrderError> {
        //let balance = Self::get_balance(user_id).map_err(stc_err_conv)?;
        let balance = Self::get_balance(user_id).err_conv()?;

        if balance < amount {
            StructError::from(OrderReason::InsufficientFunds)
                .with_detail(format!("当前余额：{}，需要：{}", balance, amount))
                .err()
        } else {
            Ok(())
        }
    }

    fn get_balance(user_id: u32) -> Result<f64, UserError> {
        if user_id != 123 {
            UserError::from(UserReason::NotFound)
                .with_detail(format!("uid:{}", user_id))
                .err()
        } else {
            Ok(500.0)
        }
    }

    fn save_order(order: storage::Order) -> Result<storage::Order, OrderError> {
        storage::save(order.clone()).err_conv()?;
        Ok(order)
    }
}

// ========== 展示错误处理 ==========

fn main() {
    // 测试用例 1: 空订单文本
    let case1 = OrderService::place_order(123, 200.0, "");
    if let Err(e) = case1 {
        print_error(&e);
    }

    // 测试用例 2: 用户不存在
    let case2 = OrderService::place_order(456, 200.0, "valid_order");
    if let Err(e) = case2 {
        print_error(&e);
    }

    // 测试用例 3: 余额不足
    let case3 = OrderService::place_order(123, 600.0, "valid_order");
    if let Err(e) = case3 {
        print_error(&e);
    }

    // 测试用例 4: 存储空间不足
    storage::STORAGE_CAPACITY.store(0, Ordering::Relaxed);
    let case4 = OrderService::place_order(123, 200.0, "valid_order");
    if let Err(e) = case4 {
        print_error(&e);
    }
}
