#[cfg(test)]
mod tests {

    use std::fmt::Display;

    use crate::{
        DomainReason, ErrorCode, ErrorWith, StructError, StructReason, TestAssertWithMsg,
        UvsReason, WithContext, convert_error_type,
    };

    // 测试用领域原因类型
    #[derive(Debug, PartialEq, Clone)]
    struct TestDomainReason;

    impl DomainReason for TestDomainReason {}
    impl Display for TestDomainReason {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestDomainReason")
        }
    }

    // 另一个领域原因类型用于转换测试
    #[derive(Debug, PartialEq, Clone)]
    struct OtherDomainReason;

    impl Display for OtherDomainReason {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "OtherDomainReason")
        }
    }

    impl DomainReason for OtherDomainReason {}

    impl ErrorCode for TestDomainReason {
        fn error_code(&self) -> i32 {
            200
        }
    }

    impl From<TestDomainReason> for StructReason<OtherDomainReason> {
        fn from(_: TestDomainReason) -> Self {
            StructReason::Domain(OtherDomainReason)
        }
    }

    #[test]
    fn test_domain_error_creation() {
        let err = StructError::domain(TestDomainReason);

        assert_eq!(err.reason(), &StructReason::Domain(TestDomainReason));
        assert_eq!(err.error_code(), 200);
    }

    #[test]
    fn test_error_with_details() {
        let err = StructError::<TestDomainReason>::domain(TestDomainReason)
            .with_detail("detailed message");

        assert_eq!(err.detail(), &Some("detailed message".to_string()));
    }

    #[test]
    fn test_error_context() {
        let mut ctx = WithContext::want("user_profile");
        ctx.with("user_id", "12345");

        let err = StructError::<TestDomainReason>::domain(TestDomainReason).with(ctx);

        assert_eq!(err.target(), &Some("user_profile".to_string()));
        assert!(
            err.context()
                .items
                .contains(&("user_id".into(), "12345".into()))
        );
    }

    #[test]
    fn test_error_conversion() {
        let original = StructError::<TestDomainReason>::domain(TestDomainReason)
            .with_detail("conversion test")
            .with_position("test.rs:1")
            .with_context(WithContext::want("ctx").context().clone());

        let converted: StructError<OtherDomainReason> = convert_error_type(original);

        assert_eq!(converted.reason(), &StructReason::Domain(OtherDomainReason));
        assert_eq!(converted.detail(), &Some("conversion test".into()));
    }

    #[test]
    fn test_error_display() {
        let mut ctx = WithContext::new();
        ctx.with("step", "initialization");
        ctx.with("resource", "database");

        let err =
            StructError::<TestDomainReason>::universal(UvsReason::core_conf("config missing"))
                .with_detail("missing db config")
                .position("src/config.rs:42")
                .want("database_config")
                .with(ctx);

        let display_output = format!("{}", err);
        println!("{}", display_output);

        assert!(display_output.contains("[105]")); // ConfError的error code
        assert!(display_output.contains("conf error << core config > config missing"));
        assert!(display_output.contains("-> At: src/config.rs:42"));
        assert!(display_output.contains("-> Want: database_config"));
        assert!(display_output.contains("-> Details: missing db config"));
        assert!(display_output.contains("Context stack:"));
        assert!(display_output.contains("1. step:initialization"));
        assert!(display_output.contains("2. resource:database"));
    }

    #[test]
    #[should_panic]
    fn test_error_assertions() {
        let result: Result<(), _> = StructError::<TestDomainReason>::domain(TestDomainReason).err();

        // 使用自定义断言trait
        result.assert("This should panic with domain error");
    }

    #[test]
    #[should_panic]
    fn test_context_assert_panic() {
        let result: Result<(), _> = StructError::<TestDomainReason>::domain(TestDomainReason).err();
        result.assert("test context error");
    }
}
