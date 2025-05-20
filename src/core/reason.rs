pub trait ErrorCode {
    fn error_code(&self) -> i32 {
        500
    }
}
