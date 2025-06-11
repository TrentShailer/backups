//! Context for the current backup
//!

use core::fmt;

use shared::Cadence;

/// Context for the current backup
pub struct Context {
    /// The service being backed up's name.
    pub service_name: String,

    /// The cadence for the current backup.
    pub cadence: Cadence,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}/{:?}] ", self.service_name, self.cadence)
    }
}
