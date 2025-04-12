use core::{fmt::Display, net::IpAddr};

use shared::Cadance;

/// Holds the context for the current connection. Used for prefixing logs.
#[derive(Default, Debug)]
pub struct Context {
    /// The connection peer.
    pub peer: Option<IpAddr>,
    /// The backup for this connection.
    pub backup: Option<(String, Cadance)>,
    /// The current context
    pub current_context: &'static str,
}

impl Display for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(peer) = &self.peer {
            write!(f, "[{peer}] ")?;
        }

        if let Some((service, cadance)) = &self.backup {
            write!(f, "[{service}/{cadance:?}] ")?;
        }

        write!(f, "[{}] ", self.current_context)?;

        Ok(())
    }
}
