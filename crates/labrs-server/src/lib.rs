//! Web server for the labrs notebook UI.

mod protocol;
mod static_files;

use crate::protocol::{ClientMessage, ServerMessage};
use anyhow::Result;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use labrs_core::fmt::rustfmt_cell_source;
use labrs_core::graph::transitive_dependents;
use labrs_core::{strip_labrs_attrs, with_labrs_attr, AddKind, MoveDirection, Session};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    session: Arc<Mutex<Session>>,
}

/// Serve the notebook UI on the given port.
pub async fn serve(file: PathBuf, port: u16) -> Result<()> {
    serve_with_options(file, port, true).await
}

/// Serve with explicit auto-reactivity default.
pub async fn serve_with_options(file: PathBuf, port: u16, auto_react: bool) -> Result<()> {
    let mut session = Session::open(&file)?;
    session.set_auto_react(auto_react);
    let state = AppState {
        session: Arc::new(Mutex::new(session)),
    };

    let app = Router::new()
        .route("/", get(static_files::index))
        .route("/app.js", get(static_files::app_js))
        .route("/app.css", get(static_files::app_css))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("labrs UI: http://{addr}");
    tracing::info!("notebook: {}", file.display());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    {
        let session = state.session.lock().await;
        if send_msg(&mut socket, &full_state(&session)).await.is_err() {
            return;
        }
    }

    while let Some(Ok(msg)) = socket.recv().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                let _ = send_msg(
                    &mut socket,
                    &ServerMessage::Error {
                        message: format!("invalid message: {e}"),
                    },
                )
                .await;
                continue;
            }
        };

        if let Err(e) = handle_client_msg(&state, client_msg, &mut socket).await {
            let _ = send_msg(
                &mut socket,
                &ServerMessage::Error {
                    message: e.to_string(),
                },
            )
            .await;
        }
    }
}

async fn handle_client_msg(
    state: &AppState,
    msg: ClientMessage,
    socket: &mut WebSocket,
) -> Result<(), anyhow::Error> {
    match msg {
        ClientMessage::GetState => {
            let session = state.session.lock().await;
            send_msg(socket, &full_state(&session)).await?;
        }
        ClientMessage::EditCell { name, source } => {
            let mut session = state.session.lock().await;
            let formatted = session.edit_cell(&name, &source)?;
            send_msg(
                socket,
                &ServerMessage::CellFormatted {
                    name: name.clone(),
                    source: formatted,
                },
            )
            .await?;
            send_msg(socket, &full_state(&session)).await?;
        }
        ClientMessage::EditHelper { name, source } => {
            let mut session = state.session.lock().await;
            let formatted = session.edit_helper(&name, &source)?;
            send_msg(
                socket,
                &ServerMessage::HelperFormatted {
                    name: name.clone(),
                    source: formatted,
                },
            )
            .await?;
            if session.auto_react {
                run_dirty_streaming(&mut session, socket).await?;
            } else {
                send_msg(socket, &full_state(&session)).await?;
            }
        }
        ClientMessage::EditDefinition { name, source } => {
            let mut session = state.session.lock().await;
            let formatted = session.edit_definition(&name, &source)?;
            send_msg(
                socket,
                &ServerMessage::DefinitionFormatted {
                    name: name.clone(),
                    source: formatted,
                },
            )
            .await?;
            if session.auto_react {
                run_dirty_streaming(&mut session, socket).await?;
            } else {
                send_msg(socket, &full_state(&session)).await?;
            }
        }
        ClientMessage::EditPreamble { source } => {
            let mut session = state.session.lock().await;
            let formatted = session.edit_preamble(&source)?;
            send_msg(
                socket,
                &ServerMessage::PreambleFormatted { source: formatted },
            )
            .await?;
            if session.auto_react {
                run_dirty_streaming(&mut session, socket).await?;
            } else {
                send_msg(socket, &full_state(&session)).await?;
            }
        }
        ClientMessage::EditMarkdown { name, content } => {
            let mut session = state.session.lock().await;
            session.edit_markdown(&name, &content)?;
            send_msg(socket, &full_state(&session)).await?;
        }
        ClientMessage::AddItem {
            kind,
            after_kind,
            after_name,
        } => {
            let mut session = state.session.lock().await;
            let add = AddKind::parse(&kind)?;
            let after = match (after_kind.as_deref(), after_name.as_deref()) {
                (None, None) => None,
                (Some("__start__"), _) | (_, Some("__start__")) => {
                    Some((AddKind::Cell, "__start__".into()))
                }
                (Some(k), Some(n)) => Some((AddKind::parse(k)?, n.to_string())),
                _ => anyhow::bail!("after_kind and after_name must both be set or both omitted"),
            };
            let _name = session.add_item(add, after)?;
            send_msg(socket, &full_state(&session)).await?;
        }
        ClientMessage::ChangeKind { name, from, to } => {
            let mut session = state.session.lock().await;
            session.change_kind(&name, AddKind::parse(&from)?, AddKind::parse(&to)?)?;
            send_msg(socket, &full_state(&session)).await?;
        }
        ClientMessage::DeleteItem { kind, name } => {
            let mut session = state.session.lock().await;
            session.delete_item(AddKind::parse(&kind)?, &name)?;
            send_msg(socket, &full_state(&session)).await?;
        }
        ClientMessage::MoveItem {
            kind,
            name,
            direction,
        } => {
            let mut session = state.session.lock().await;
            let dir = match direction.as_str() {
                "up" => MoveDirection::Up,
                "down" => MoveDirection::Down,
                other => anyhow::bail!("unknown direction `{other}`"),
            };
            session.move_item(AddKind::parse(&kind)?, &name, dir)?;
            send_msg(socket, &full_state(&session)).await?;
        }
        ClientMessage::RunCell { name } => {
            let mut session = state.session.lock().await;
            if let Some(cell) = session.notebook.cell(&name).cloned() {
                let bare = strip_labrs_attrs(&cell.source);
                let for_disk = with_labrs_attr(&bare, "cell");
                let formatted = rustfmt_cell_source(&for_disk);
                if strip_labrs_attrs(&formatted) != bare {
                    session.edit_cell(&name, &bare)?;
                }
            }
            run_reactive_streaming(&mut session, &name, socket).await?;
        }
        ClientMessage::SetAuto { enabled } => {
            let mut session = state.session.lock().await;
            session.set_auto_react(enabled);
            send_msg(socket, &full_state(&session)).await?;
        }
        ClientMessage::RunAll => {
            let mut session = state.session.lock().await;
            let names: Vec<String> = session
                .notebook
                .cells
                .iter()
                .map(|c| c.name.clone())
                .collect();
            for name in names {
                session.dirty.insert(name, true);
            }
            run_dirty_streaming(&mut session, socket).await?;
        }
        ClientMessage::Reload => {
            let mut session = state.session.lock().await;
            session.reload()?;
            send_msg(socket, &full_state(&session)).await?;
        }
    }
    Ok(())
}

async fn run_reactive_streaming(
    session: &mut Session,
    name: &str,
    socket: &mut WebSocket,
) -> Result<(), anyhow::Error> {
    send_msg(
        socket,
        &ServerMessage::CellRunning {
            name: name.to_string(),
        },
    )
    .await?;

    let (first, changed) = session.run_cell_once(name)?;
    send_msg(
        socket,
        &ServerMessage::CellOutput {
            output: first.clone(),
        },
    )
    .await?;

    if session.auto_react && first.success && changed {
        let cascade = transitive_dependents(&session.graph, name);
        for dep_name in cascade {
            if !session.dirty.get(&dep_name).copied().unwrap_or(false) {
                continue;
            }
            let cell = match session.notebook.cell(&dep_name) {
                Some(c) => c.clone(),
                None => continue,
            };
            let ready = cell.params.iter().all(|p| {
                session
                    .outputs
                    .get(&p.name)
                    .map(|o| o.success)
                    .unwrap_or(false)
            });
            if !ready {
                continue;
            }

            send_msg(
                socket,
                &ServerMessage::CellRunning {
                    name: dep_name.clone(),
                },
            )
            .await?;

            match session.run_cell_once(&dep_name) {
                Ok((out, _)) => {
                    send_msg(socket, &ServerMessage::CellOutput { output: out }).await?;
                }
                Err(e) => {
                    send_msg(
                        socket,
                        &ServerMessage::Error {
                            message: format!("cell `{dep_name}`: {e}"),
                        },
                    )
                    .await?;
                }
            }
        }
    }

    let dirty: Vec<String> = session
        .dirty
        .iter()
        .filter(|(_, d)| **d)
        .map(|(k, _)| k.clone())
        .collect();
    send_msg(socket, &ServerMessage::CellsDirty { cells: dirty }).await?;
    send_msg(socket, &full_state(session)).await?;
    Ok(())
}

async fn run_dirty_streaming(
    session: &mut Session,
    socket: &mut WebSocket,
) -> Result<(), anyhow::Error> {
    let order = session.graph.order.clone();
    for name in order {
        if !session.dirty.get(&name).copied().unwrap_or(false) {
            continue;
        }
        let cell = match session.notebook.cell(&name) {
            Some(c) => c.clone(),
            None => continue,
        };
        let ready = cell.params.is_empty()
            || cell.params.iter().all(|p| {
                session
                    .outputs
                    .get(&p.name)
                    .map(|o| o.success)
                    .unwrap_or(false)
            });
        if !ready {
            continue;
        }

        send_msg(socket, &ServerMessage::CellRunning { name: name.clone() }).await?;

        match session.run_cell_once(&name) {
            Ok((out, _)) => {
                send_msg(socket, &ServerMessage::CellOutput { output: out }).await?;
            }
            Err(e) => {
                send_msg(
                    socket,
                    &ServerMessage::Error {
                        message: format!("cell `{name}`: {e}"),
                    },
                )
                .await?;
            }
        }
    }

    let dirty: Vec<String> = session
        .dirty
        .iter()
        .filter(|(_, d)| **d)
        .map(|(k, _)| k.clone())
        .collect();
    send_msg(socket, &ServerMessage::CellsDirty { cells: dirty }).await?;
    send_msg(socket, &full_state(session)).await?;
    Ok(())
}

fn full_state(session: &Session) -> ServerMessage {
    ServerMessage::NotebookState {
        snapshot: session.snapshot(),
        notebook_source: session.notebook.source.clone(),
        cells_detail: session
            .notebook
            .cells
            .iter()
            .map(|c| protocol::CellDetail {
                name: c.name.clone(),
                source: strip_labrs_attrs(&c.source),
                docs: c.docs.clone(),
                return_type: c.return_type.clone(),
                params: c
                    .params
                    .iter()
                    .map(|p| protocol::ParamDetail {
                        name: p.name.clone(),
                        ty: p.ty.clone(),
                    })
                    .collect(),
            })
            .collect(),
        helpers_detail: session
            .notebook
            .helpers
            .iter()
            .map(|h| protocol::HelperDetail {
                name: h.name.clone(),
                source: strip_labrs_attrs(&h.source),
                docs: h.docs.clone(),
            })
            .collect(),
        markdown_detail: session
            .notebook
            .markdown
            .iter()
            .map(|m| protocol::MarkdownDetail {
                name: m.name.clone(),
                content: m.content.clone(),
                source: m.source.clone(),
            })
            .collect(),
        definitions_detail: session
            .notebook
            .definitions
            .iter()
            .map(|d| protocol::DefinitionDetail {
                name: d.name.clone(),
                kind: d.kind.clone(),
                source: d.source.clone(),
            })
            .collect(),
    }
}

async fn send_msg(socket: &mut WebSocket, msg: &ServerMessage) -> Result<(), axum::Error> {
    let text = serde_json::to_string(msg).unwrap();
    socket.send(Message::Text(text.into())).await
}
