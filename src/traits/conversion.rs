use crate::{core::convert_error, DomainReason, StructError};

pub trait ErrorConv<T, R: DomainReason>: Sized {
    fn err_conv(self) -> Result<T, StructError<R>>;
}

pub trait ConvStructError<R: DomainReason>: Sized {
    fn conv(self) -> StructError<R>;
}

impl<T, R1, R2> ErrorConv<T, R2> for Result<T, StructError<R1>>
where
    R1: DomainReason,
    R2: DomainReason + From<R1>,
{
    fn err_conv(self) -> Result<T, StructError<R2>> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(convert_error::<R1, R2>(e)),
        }
    }
}

impl<R1, R2> ConvStructError<R2> for StructError<R1>
where
    R1: DomainReason,
    R2: DomainReason + From<R1>,
{
    fn conv(self) -> StructError<R2> {
        convert_error::<R1, R2>(self)
    }
}

pub trait ToStructError<R>
where
    R: DomainReason,
{
    fn to_err(self) -> StructError<R>;
    fn err_result<T>(self) -> Result<T, StructError<R>>;
}
impl<R> ToStructError<R> for R
where
    R: DomainReason,
{
    fn to_err(self) -> StructError<R> {
        StructError::from(self)
    }
    fn err_result<T>(self) -> Result<T, StructError<R>> {
        Err(StructError::from(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ErrorCode, StructError, UvsReason};

    // 定义测试用的 DomainReason
    #[derive(Debug, Clone, PartialEq, serde::Serialize, thiserror::Error)]
    enum TestReason {
        #[error("test error")]
        TestError,
        #[error("{0}")]
        Uvs(UvsReason),
    }

    impl ErrorCode for TestReason {
        fn error_code(&self) -> i32 {
            match self {
                TestReason::TestError => 1001,
                TestReason::Uvs(uvs) => uvs.error_code(),
            }
        }
    }

    impl From<UvsReason> for TestReason {
        fn from(uvs: UvsReason) -> Self {
            TestReason::Uvs(uvs)
        }
    }

    // 定义另一个 DomainReason 用于测试转换
    #[derive(Debug, Clone, PartialEq, serde::Serialize, thiserror::Error)]
    enum AnotherReason {
        #[error("another error")]
        AnotherError,
        #[error("{0}")]
        Uvs(UvsReason),
    }

    impl ErrorCode for AnotherReason {
        fn error_code(&self) -> i32 {
            match self {
                AnotherReason::AnotherError => 2001,
                AnotherReason::Uvs(uvs) => uvs.error_code(),
            }
        }
    }

    impl From<UvsReason> for AnotherReason {
        fn from(uvs: UvsReason) -> Self {
            AnotherReason::Uvs(uvs)
        }
    }

    impl From<TestReason> for AnotherReason {
        fn from(test: TestReason) -> Self {
            match test {
                TestReason::TestError => AnotherReason::AnotherError,
                TestReason::Uvs(uvs) => AnotherReason::Uvs(uvs),
            }
        }
    }

    #[test]
    fn test_error_conv_trait() {
        // 测试 ErrorConv trait 的 err_conv 方法
        let original_result: Result<i32, StructError<TestReason>> =
            Err(TestReason::TestError.to_err());

        let converted_result: Result<i32, StructError<AnotherReason>> = original_result.err_conv();

        assert!(converted_result.is_err());
        let converted_error = converted_result.unwrap_err();
        assert_eq!(converted_error.error_code(), 2001);

        // 测试成功情况下的转换
        let success_result: Result<i32, StructError<TestReason>> = Ok(42);
        let converted_success: Result<i32, StructError<AnotherReason>> = success_result.err_conv();

        assert!(converted_success.is_ok());
        assert_eq!(converted_success.unwrap(), 42);
    }

    #[test]
    fn test_conv_struct_error_trait() {
        // 测试 ConvStructError trait 的 conv 方法
        let original_error: StructError<TestReason> = TestReason::TestError.to_err();

        let converted_error: StructError<AnotherReason> = original_error.conv();

        assert_eq!(converted_error.error_code(), 2001);

        // 测试带有 UvsReason 的转换
        let uvs_error: StructError<TestReason> =
            TestReason::Uvs(UvsReason::network_error("network failed")).to_err();

        let converted_uvs_error: StructError<AnotherReason> = uvs_error.conv();

        assert_eq!(converted_uvs_error.error_code(), 202);
    }

    #[test]
    fn test_to_struct_error_trait() {
        // 测试 ToStructError trait 的 to_err 方法
        let reason = TestReason::TestError;
        let error: StructError<TestReason> = reason.to_err();

        assert_eq!(error.error_code(), 1001);

        // 测试 ToStructError trait 的 err_result 方法
        let reason2 = TestReason::TestError;
        let result: Result<String, StructError<TestReason>> = reason2.err_result();

        assert!(result.is_err());
        let error_from_result = result.unwrap_err();
        assert_eq!(error_from_result.error_code(), 1001);

        // 测试使用 UvsReason
        let uvs_reason1 = UvsReason::validation_error("invalid input");
        let uvs_error: StructError<UvsReason> = uvs_reason1.to_err();

        assert_eq!(uvs_error.error_code(), 100);

        let uvs_reason2 = UvsReason::validation_error("invalid input");
        let uvs_result: Result<i32, StructError<UvsReason>> = uvs_reason2.err_result();
        assert!(uvs_result.is_err());
        assert_eq!(uvs_result.unwrap_err().error_code(), 100);
    }
}
