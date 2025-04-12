/// Possible responses from the server.
#[repr(u64)]
#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    /// The sender sucesfully saved the backup.
    Success = 0,

    /// The sender encountered an unexpected error.
    Error = 1,

    /// The client sent bad data.
    BadData = 2,

    /// The sender has exceeded the rate limit.
    ExceededRateLimit = 3,

    /// The sender tried to send something that exeeded the size limit.
    TooLarge = 4,

    /// The payload took too long to receive.
    Timeout = 5,
}

impl Response {
    /// Converts the response to big endian bytes.
    pub fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        let value: u64 = unsafe { core::mem::transmute(self) };
        value.to_be_bytes()
    }

    /// Try convert a u64 value to a response.
    pub fn try_from_u64(value: u64) -> Option<Self> {
        match value {
            0..=5 => Some(unsafe { core::mem::transmute::<u64, Self>(value) }),
            _ => None,
        }
    }
}
