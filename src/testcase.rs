// 测试专用断言 (无消息)
pub trait TestAssert {
    type Output;
    fn assert(self) -> Self::Output;
}

// 测试专用断言 (带消息)
pub trait TestAssertWithMsg<A> {
    type Output;
    fn assert(self, msg: A) -> Self::Output;
}

impl<T, E> TestAssert for Result<T, E>
where
    E: std::fmt::Display,
{
    type Output = T;

    fn assert(self) -> T {
        self.unwrap_or_else(|e| panic!("[TEST ASSERTION FAILED] \n Error details: {}", e))
    }
}

impl<T, E> TestAssertWithMsg<&str> for Result<T, E>
where
    E: std::fmt::Display,
{
    type Output = T;

    fn assert(self, msg: &str) -> T {
        self.unwrap_or_else(|e| panic!("[TEST ASSERTION FAILED] {} \n Error details: {}", msg, e))
    }
}

impl<T> TestAssert for Option<T> {
    type Output = T;

    fn assert(self) -> T {
        self.unwrap_or_else(|| panic!("[OPTION ASSERTION FAILED] ",))
    }
}
