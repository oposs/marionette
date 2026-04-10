use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use chrono::Utc;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use marionette_protocol::{
    ActionMessage, ErrorMessage, HelloMessage, ProtocolMessage, ValidationError,
};

use crate::db::session;
use crate::extractors::HandlerContext;
use crate::router::ActionRouter;
use crate::session::WsSession;

/// Shared application state passed to WebSocket handlers.
pub struct AppState {
    /// Action router for dispatching incoming actions.
    pub router: ActionRouter,
    /// Database connection pool.
    pub db: Arc<sea_orm::DatabaseConnection>,
    /// Optional login form to send to unauthenticated WebSocket connections.
    pub login_form: Option<ProtocolMessage>,
    /// Optional extension data (e.g., external service clients).
    pub listmonk: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

/// Axum handler that upgrades an HTTP connection to a WebSocket.
///
/// Accepts connections at the configured route (typically `/ws`) and hands
/// off to [`handle_session`] for the message loop. Extracts the session
/// cookie from the HTTP upgrade request for authentication.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl IntoResponse {
    let session_token = jar
        .get("marionette_session")
        .map(|c| c.value().to_owned());
    ws.on_upgrade(move |socket| handle_session(socket, state, session_token))
}

/// Main WebSocket session loop.
///
/// Splits the socket into reader/writer halves, sends the hello message,
/// then processes incoming actions until the connection closes.
async fn handle_session(
    socket: WebSocket,
    state: Arc<AppState>,
    session_token: Option<String>,
) {
    let (ws_sender, ws_receiver) = socket.split();
    let (tx, rx) = mpsc::channel::<ProtocolMessage>(32);
    let mut ws_session = WsSession::new();

    // Authenticate from session cookie if present
    if let Some(token) = session_token {
        match session::Entity::find()
            .filter(session::Column::SessionToken.eq(&token))
            .one(&*state.db)
            .await
        {
            Ok(Some(model)) => {
                // Check if session has expired
                let expired = chrono::NaiveDateTime::parse_from_str(
                    &model.session_expires,
                    "%Y-%m-%dT%H:%M:%SZ",
                )
                .map(|dt| dt.and_utc() < Utc::now())
                .unwrap_or(true);

                if expired {
                    debug!(session_id = %ws_session.id, "Session cookie expired");
                } else {
                    ws_session.user_id = model.session_user.map(|id| id.to_string());
                    ws_session.roles =
                        serde_json::from_str(&model.session_roles).unwrap_or_default();
                    debug!(
                        session_id = %ws_session.id,
                        user_id = ?ws_session.user_id,
                        "Authenticated from cookie"
                    );
                }
            }
            Ok(None) => {
                debug!(session_id = %ws_session.id, "Session token not found in DB");
            }
            Err(e) => {
                warn!(session_id = %ws_session.id, error = %e, "Failed to look up session");
            }
        }
    }

    debug!(session_id = %ws_session.id, "WebSocket session started");

    // Writer task: drains mpsc channel and sends to WebSocket
    let write_task = tokio::spawn(write_loop(ws_sender, rx));

    // Send hello message
    let hello = ProtocolMessage::Hello(HelloMessage {
        version: "1.1.0".into(),
    });
    if tx.send(hello).await.is_err() {
        warn!(session_id = %ws_session.id, "Failed to send hello -- writer closed");
        write_task.abort();
        return;
    }

    // Send login form to unauthenticated sessions
    if ws_session.user_id.is_none()
        && let Some(ref login_form) = state.login_form
        && tx.send(login_form.clone()).await.is_err()
    {
        warn!(session_id = %ws_session.id, "Failed to send login form");
        write_task.abort();
        return;
    }

    // Reader loop: reads WS messages, dispatches actions, sends responses through tx
    read_loop(ws_receiver, &tx, &state, &mut ws_session).await;

    // Clean up: drop sender to signal writer task, then wait for it
    drop(tx);
    let _ = write_task.await;

    debug!(session_id = %ws_session.id, "WebSocket session ended");
}

/// Reads incoming WebSocket messages and dispatches actions through the router.
async fn read_loop(
    mut receiver: SplitStream<WebSocket>,
    tx: &mpsc::Sender<ProtocolMessage>,
    state: &Arc<AppState>,
    session: &mut WsSession,
) {
    while let Some(result) = receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                debug!(session_id = %session.id, error = %e, "WebSocket receive error");
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                handle_text_message(&text, tx, state, session).await;
            }
            Message::Close(_) => {
                debug!(session_id = %session.id, "Received close frame");
                break;
            }
            // Axum handles ping/pong automatically at the protocol level,
            // but we handle Ping explicitly just in case.
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Binary(_) => {
                warn!(session_id = %session.id, "Received unexpected binary message");
            }
        }
    }
}

/// Parse a text message, check its type, and dispatch actions.
async fn handle_text_message(
    text: &str,
    tx: &mpsc::Sender<ProtocolMessage>,
    state: &Arc<AppState>,
    session: &mut WsSession,
) {
    // Parse as generic JSON first to check the message type
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            let error_msg = ProtocolMessage::Error(ErrorMessage {
                id: None,
                errors: vec![ValidationError {
                    path: None,
                    message: format!("Invalid message: {e}"),
                }],
            });
            let _ = tx.send(error_msg).await;
            return;
        }
    };

    // Check message type before dispatching
    let msg_type = value.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match msg_type {
        "action" => {
            // Parse as ActionMessage and dispatch
        }
        "hello" => {
            debug!(session_id = %session.id, "Received client hello, acknowledging");
            return;
        }
        other => {
            let error_msg = ProtocolMessage::Error(ErrorMessage {
                id: None,
                errors: vec![ValidationError {
                    path: None,
                    message: format!("Unexpected message type: {other}"),
                }],
            });
            let _ = tx.send(error_msg).await;
            return;
        }
    }

    let action: ActionMessage = match serde_json::from_value(value) {
        Ok(action) => action,
        Err(e) => {
            let error_msg = ProtocolMessage::Error(ErrorMessage {
                id: None,
                errors: vec![ValidationError {
                    path: None,
                    message: format!("Invalid action message: {e}"),
                }],
            });
            let _ = tx.send(error_msg).await;
            return;
        }
    };

    let action_id = action.id.clone();
    let action_name = action.name.clone();
    debug!(session_id = %session.id, action = %action_name, "Dispatching action");

    // Build handler context and dispatch
    let ctx = HandlerContext {
        action,
        db: Arc::clone(&state.db),
        session: session.to_session(),
    };

    let responses = state.router.dispatch(ctx).await;

    // If a login action succeeded (returned Render, not Error), update the WsSession
    // so subsequent actions on this connection pass auth checks.
    if action_name == "login"
        && responses
            .iter()
            .any(|r| matches!(r, ProtocolMessage::Render(_)))
    {
        // The login handler validated credentials. Extract user info from the
        // render response data (the handler embeds it for us).
        for r in &responses {
            if let ProtocolMessage::Render(render) = r {
                if let Some(user_id) = render
                    .data
                    .get("_auth_user_id")
                    .and_then(serde_json::Value::as_i64)
                {
                    session.user_id = Some(user_id.to_string());
                }
                if let Some(role) = render.data.get("_auth_role").and_then(|v| v.as_str()) {
                    session.roles = vec![role.to_string()];
                }
            }
        }
        if session.user_id.is_some() {
            info!(session_id = %session.id, user_id = ?session.user_id, "WebSocket session authenticated via login action");
        }
    }

    // Send each response message; if the channel is closed, stop.
    for mut response in responses {
        // Propagate the action ID to response messages if they don't have one
        if let Some(ref id) = action_id {
            propagate_id(&mut response, id);
        }
        if tx.send(response).await.is_err() {
            warn!(session_id = %session.id, "Writer channel closed during response send");
            return;
        }
    }
}

/// Propagate the action correlation ID to response messages that lack one.
fn propagate_id(msg: &mut ProtocolMessage, id: &str) {
    match msg {
        ProtocolMessage::Render(m) if m.id.is_none() => m.id = Some(id.to_owned()),
        ProtocolMessage::Patch(m) if m.id.is_none() => m.id = Some(id.to_owned()),
        ProtocolMessage::Event(m) if m.id.is_none() => m.id = Some(id.to_owned()),
        ProtocolMessage::Error(m) if m.id.is_none() => m.id = Some(id.to_owned()),
        _ => {}
    }
}

/// Drains the mpsc channel and sends each message as JSON text over the WebSocket.
async fn write_loop(
    mut sender: SplitSink<WebSocket, Message>,
    mut rx: mpsc::Receiver<ProtocolMessage>,
) {
    while let Some(msg) = rx.recv().await {
        match serde_json::to_string(&msg) {
            Ok(text) => {
                if let Err(e) = sender.send(Message::Text(text.into())).await {
                    error!(error = %e, "Failed to send WebSocket message");
                    break;
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to serialize protocol message");
            }
        }
    }
}
