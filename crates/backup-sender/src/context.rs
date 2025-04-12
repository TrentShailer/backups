//! Context for the current backup
//!

use core::fmt;

use shared::Cadance;

/// Context for the current backup
pub struct Context {
    /// The service being backed up's name.
    pub service_name: String,

    /// The cadace for the current backup.
    pub cadance: Cadance,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}/{:?}]", self.service_name, self.cadance)
    }
}
