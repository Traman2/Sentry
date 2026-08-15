//! Resolving a loopback TCP peer to the OS process on the other end of it.
//!
//! The MCP server only ever sees `127.0.0.1:<ephemeral port>` for a connecting client — never
//! a pid. Both ends of a loopback connection are local sockets, though, each owned by some
//! process on this machine, so the OS's own TCP table can answer "which process holds the
//! socket whose local port is the ephemeral port I saw as the client's remote port". That's
//! what this module does; nothing here is MCP-specific.

use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, get_sockets_info};

/// Finds the pid of the local process whose TCP socket is `client_port` talking to
/// `server_port` on loopback.
///
/// `server_port` is this app's own MCP listener (a fixed, known port). `client_port` is the
/// ephemeral port `axum`'s `ConnectInfo` saw as the remote peer of an inbound request — i.e.
/// from the client process's own socket, its *local* port. Returns `None` on any lookup
/// failure or if no matching row is found; a caller should treat that as "process unknown"
/// rather than an error, since usage logging must never block or fail a tool call over this.
pub fn pid_for_loopback_client(server_port: u16, client_port: u16) -> Option<u32> {
    let sockets = get_sockets_info(AddressFamilyFlags::IPV4, ProtocolFlags::TCP).ok()?;

    sockets
        .into_iter()
        .find(|socket| match &socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => {
                tcp.local_port == client_port && tcp.remote_port == server_port
            }
            _ => false,
        })
        .and_then(|socket| socket.associated_pids.first().copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_for_a_port_pair_nothing_is_using() {
        // Port 1 is never a real ephemeral client port paired with server port 1 — this just
        // exercises the "no matching row" path without depending on any real connection
        // existing at test time.
        assert_eq!(pid_for_loopback_client(1, 1), None);
    }
}
