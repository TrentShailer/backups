use tracing::error;

pub fn log_and_panic<Err: core::fmt::Display>(error: Err, message: &str) -> ! {
    error!("{message}: {error}");

    panic!("{message}: {error}");
}

/// Extension trait for results.
#[allow(unused)]
pub trait Failure<T> {
    /// Log an error an panic.
    fn or_log_and_panic(self, message: &str) -> T;
}

impl<T, E: core::fmt::Display> Failure<T> for Result<T, E> {
    fn or_log_and_panic(self, message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => log_and_panic(error, message),
        }
    }
}
