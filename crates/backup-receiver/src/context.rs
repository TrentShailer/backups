use core::{fmt::Display, net::IpAddr};

use shared::Cadence;

/// Holds the context for the current connection. Used for prefixing logs.
#[derive(Default, Debug)]
pub struct Context {
    /// The connection peer.
    pub peer: Option<IpAddr>,
    /// The backup for this connection.
    pub backup: Option<(String, Cadence)>,
    /// The current context
    pub current_context: &'static str,
}

impl Display for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(peer) = &self.peer {
            write!(f, "[{peer}] ")?;
        }

        if let Some((service, cadence)) = &self.backup {
            write!(f, "[{service}/{cadence:?}] ")?;
        }

        write!(f, "[{}] ", self.current_context)?;

        Ok(())
    }
}
