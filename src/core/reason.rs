/// Represents the root cause of an error, which can be either
/// a domain-specific reason or a universal system reason.

pub trait ErrorCode {
    fn error_code(&self) -> i32 {
        500
    }
}
