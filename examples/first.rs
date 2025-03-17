use orion_error::{
    DomainFrom, DomainReason, ErrorCode, ErrorOwe, ErrorWith, StructError, UseTarget, WithContext,
};
use std::fmt::{self, Display};

// 1. 定义领域错误类型
#[derive(Debug, PartialEq, Clone)]
enum UserReason {
    InvalidInput,
    PermissionDenied,
}
impl ErrorCode for UserReason {
    fn error_code(&self) -> i32 {
        500
    }
}

impl DomainReason for UserReason {}
impl Display for UserReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidInput => write!(f, "Invalid user input"),
            Self::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

type UserError = StructError<UserReason>;
// 2. 业务函数示例
fn validate_user(age: i32) -> Result<(), UserError> {
    // 使用 WithContext 构建上下文
    let mut ctx = WithContext::want("user_service");
    ctx = ctx.with("validate_age()").with("input_check");

    if age < 0 {
        // 创建领域错误
        return StructError::from_domain(UserReason::InvalidInput)
            .with_detail(format!("Invalid age: {}", age))
            .err();
        //.with_context(ctx.context().clone())
        //.with(&ctx)
        //.err()
    } else if age < 18 {
        // 转换第三方错误
        let txt = std::fs::read_to_string("license.txt")
            // 使用 ErrorOwe 转换标准库错误
            .owe_biz()
            .want("license_check")?;
        //.with_detail("Age verification failed")?;
        let data = txt.parse::<i32>().owe_sys()?;
        return Ok(());
    } else {
        Ok(())
    }
}

// 3. 错误处理示例
fn handle_error(result: Result<(), UserError>) {
    match result {
        Ok(_) => println!("Operation succeeded"),
        Err(e) => {
            println!("Error Code: {}", e.error_code());
            println!("Error Details:\n{}", e);

            // 错误转换示例
            //let converted_err = orion_error::core::stc_err_conv(e);
            //println!("Converted Error: {:?}", converted_err);
        }
    }
}

fn main() {
    // 测试用例
    let test_ages = vec![-5, 16, 25];

    for age in test_ages {
        println!("\nTesting age: {}", age);
        let result = validate_user(age);
        handle_error(result);
    }
}
