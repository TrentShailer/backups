use bytemuck::{CheckedBitPattern, NoUninit};

/// Possible responses from the server.
#[repr(u64)]
#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq, CheckedBitPattern, NoUninit)]
pub enum Response {
    /// The sender sucesfully saved the backup.
    Success,

    /// The sender encountered an unexpected error.
    Error,

    /// The client sent bad data.
    BadData,

    /// The sender has exceeded the rate limit.
    ExceededRateLimit,

    /// The sender tried to send something that exeeded the size limit.
    TooLarge,

    /// The payload took too long to receive.
    Timeout,
}
