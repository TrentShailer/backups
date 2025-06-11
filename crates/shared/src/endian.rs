#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The endian of a given system.
pub enum Endian {
    /// Big Endian
    Big = 0,

    /// Little Endian
    Little = 1,
}

impl Endian {
    /// Gets the current endian.
    pub fn current() -> Self {
        if cfg!(target_endian = "little") {
            Self::Little
        } else {
            Self::Big
        }
    }

    /// If this instance of Endian matches the system running the application.
    pub fn is_current(&self) -> bool {
        self.eq(&Self::current())
    }

    /// Try to convert a `u8` to an instance of self.
    pub fn try_from_u8(value: u8) -> Option<Self> {
        match value {
            0..=1 => Some(unsafe { core::mem::transmute::<u8, Self>(value) }),
            _ => None,
        }
    }
}

impl From<Endian> for u8 {
    #[allow(clippy::as_conversions)]
    fn from(value: Endian) -> Self {
        value as Self
    }
}
