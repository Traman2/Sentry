//! The push channel to the agent: a WebSocket at `/events` alongside the MCP endpoint.
//!
//! MCP is request/response — the agent calls a tool, the app answers. That leaves no way for
//! the app to tell the agent something happened, so without this the agent's only option is
//! to ask repeatedly. This carries the other direction: the app publishes when a user sends
//! a message or changes the model, and the agent reacts.
//!
//! The socket doubles as a liveness signal. It closes the instant the app dies — including
//! on a crash or force-kill, where no shutdown hook runs — so the agent learns the app is
//! gone immediately instead of inferring it from failed requests.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

/// Buffered events per subscriber. Generous: a subscriber only falls behind while it's busy
/// running an agent turn, and dropping a chat notification would strand that message.
const CAPACITY: usize = 256;

/// Something the agent should react to.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// A user sent a message. The agent re-reads the space and answers it.
    ChatMessage { chat_space_id: i64 },
    /// The user hit stop. The agent abandons the turn it's running for this space.
    CancelChat { chat_space_id: i64 },
    /// The model picker changed, or this is the snapshot sent on connect.
    AgentConfig { model: String },
}

/// Fan-out to every connected agent.
///
/// Cloneable and cheap; held as Tauri state so command handlers can publish.
#[derive(Clone)]
pub struct EventBus(broadcast::Sender<String>);

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CAPACITY);
        Self(tx)
    }

    /// Publishes to all subscribers. A send with no subscribers is not an error — the agent
    /// simply isn't connected, and it reconciles on its next connect.
    pub fn publish(&self, event: &Event) {
        match serde_json::to_string(event) {
            Ok(json) => {
                let _ = self.0.send(json);
            }
            Err(e) => eprintln!("sentry-events: could not serialize {event:?}: {e}"),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.0.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Emitted to the frontend when an agent connects or disconnects.
///
/// This connection is the app's liveness signal for the agent, which is why nothing polls
/// for it: the socket opens when the agent is ready and closes the moment it stops — on a
/// clean exit, a crash, or a kill alike.
pub const EVENT_AGENT_CONNECTION: &str = "agent://connection";

/// `GET /events` — upgrades to a WebSocket carrying [`Event`]s as JSON text frames.
pub async fn handler(
    ws: WebSocketUpgrade,
    bus: EventBus,
    hello: Event,
    app: AppHandle,
) -> Response {
    ws.on_upgrade(move |socket| pump(socket, bus, hello, app))
}

async fn pump(mut socket: WebSocket, bus: EventBus, hello: Event, app: AppHandle) {
    let _ = app.emit(EVENT_AGENT_CONNECTION, true);
    run(&mut socket, bus, hello).await;
    let _ = app.emit(EVENT_AGENT_CONNECTION, false);
}

async fn run(socket: &mut WebSocket, bus: EventBus, hello: Event) {
    // Subscribe before sending the greeting so nothing published in between is missed.
    let mut events = bus.subscribe();

    // The current config up front, so a freshly connected agent is correct immediately
    // rather than after the first change.
    if let Ok(json) = serde_json::to_string(&hello) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    loop {
        tokio::select! {
            received = events.recv() => match received {
                Ok(json) => {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        return; // client gone
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    // The agent was busy long enough to overflow its buffer. It reconciles
                    // by re-scanning on the next event, so this is worth reporting but not
                    // worth dropping the connection over.
                    eprintln!("sentry-events: subscriber lagged, {skipped} event(s) dropped");
                }
                Err(broadcast::error::RecvError::Closed) => return,
            },
            // Drain the inbound half so close frames and pings are handled; the agent never
            // sends us anything meaningful.
            inbound = socket.recv() => match inbound {
                Some(Ok(_)) => continue,
                _ => return,
            },
        }
    }
}
