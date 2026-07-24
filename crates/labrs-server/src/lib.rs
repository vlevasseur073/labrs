//! Web server for the labrs notebook UI.

mod lsp;
mod protocol;
mod static_files;

use crate::protocol::{ClientMessage, DirEntry, ServerMessage};
use anyhow::{bail, Context, Result};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use futures_util::StreamExt;
use labrs_core::fmt::rustfmt_cell_source;
use labrs_core::graph::transitive_dependents;
use labrs_core::{
    strip_labrs_attrs, with_labrs_attr, ActiveRun, AddKind, MoveDirection, Session,
};
use serde::Deserialize;
use std::fs;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    /// Filesystem root for browsing (usually process cwd).
    root: PathBuf,
    session: Arc<Mutex<Option<Session>>>,
    auto_react: Arc<Mutex<bool>>,
    /// Current browse directory relative to root (welcome mode).
    browse_cwd: Arc<Mutex<String>>,
    /// Shared kill handle for the in-flight cell cargo process.
    active_run: Arc<ActiveRun>,
}

/// Serve the notebook UI on the given port.
pub async fn serve(file: PathBuf, port: u16) -> Result<()> {
    serve_with_options(Some(file), port, true).await
}

/// Serve with optional notebook path and auto-reactivity default.
pub async fn serve_with_options(file: Option<PathBuf>, port: u16, auto_react: bool) -> Result<()> {
    let root = std::env::current_dir().context("current_dir")?;
    let active_run = ActiveRun::new();
    let session = match file {
        Some(path) => {
            let mut s = Session::open(&path)?;
            s.active_run = active_run.clone();
            s.set_auto_react(auto_react);
            tracing::info!("notebook: {}", path.display());
            Some(s)
        }
        None => {
            tracing::info!("no notebook — welcome / file browser");
            None
        }
    };

    let state = AppState {
        root: root.canonicalize().unwrap_or(root),
        session: Arc::new(Mutex::new(session)),
        auto_react: Arc::new(Mutex::new(auto_react)),
        browse_cwd: Arc::new(Mutex::new(String::new())),
        active_run,
    };

    let app = Router::new()
        .route("/", get(static_files::index))
        .route("/app.js", get(static_files::app_js))
        .route("/app.css", get(static_files::app_css))
        .route("/ws", get(ws_handler))
        .route("/lsp", get(lsp_handler))
        .route("/stop", post(stop_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("labrs UI: http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

#[derive(Debug, Deserialize)]
struct StopQuery {
    /// Optional cell name; if omitted, stops whatever is currently running.
    cell: Option<String>,
}

/// Kill the in-flight cell process. Used by the Stop button (HTTP so it works
/// even while the WebSocket handler is blocked in `cargo run`).
async fn stop_handler(
    State(state): State<AppState>,
    Query(q): Query<StopQuery>,
) -> impl IntoResponse {
    let current = state.active_run.current_cell();
    let should = match (&current, &q.cell) {
        (Some(running), Some(want)) => running == want,
        (Some(_), None) => true,
        (None, _) => false,
    };
    if should {
        // Prefer AppState handle (same Arc as the session) so Stop works while
        // the WebSocket thread is blocked inside cargo/wait.
        let stopped = state.active_run.cancel();
        Json(serde_json::json!({ "ok": true, "cell": stopped }))
    } else {
        // Still try a blanket cancel in case the cell name is stale in the UI.
        let stopped = state.active_run.cancel();
        Json(serde_json::json!({
            "ok": stopped.is_some(),
            "cell": stopped,
            "message": if stopped.is_some() { "stopped" } else { "no matching cell is running" }
        }))
    }
}

async fn lsp_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let path = {
            let guard = state.session.lock().await;
            guard.as_ref().map(|s| s.path.clone())
        };
        match path {
            Some(path) => lsp::handle_lsp_websocket(socket, path).await,
            None => {
                use futures_util::SinkExt;
                let (mut sender, _recv) = socket.split();
                let msg = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "window/showMessage",
                    "params": {
                        "type": 2,
                        "message": "Open a notebook to enable rust-analyzer."
                    }
                });
                let _ = sender.send(Message::Text(msg.to_string().into())).await;
            }
        }
    })
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    {
        if send_current_state(&state, &mut socket).await.is_err() {
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

async fn send_current_state(state: &AppState, socket: &mut WebSocket) -> Result<(), axum::Error> {
    let guard = state.session.lock().await;
    if let Some(session) = guard.as_ref() {
        send_msg(socket, &full_state(session)).await
    } else {
        drop(guard);
        let welcome = welcome_message(state).await;
        send_msg(socket, &welcome).await
    }
}

async fn welcome_message(state: &AppState) -> ServerMessage {
    let cwd = state.browse_cwd.lock().await.clone();
    let auto_react = *state.auto_react.lock().await;
    let entries = list_dir_entries(&state.root, &cwd).unwrap_or_default();
    ServerMessage::Welcome {
        root: state.root.display().to_string(),
        cwd,
        entries,
        auto_react,
    }
}

async fn handle_client_msg(
    state: &AppState,
    msg: ClientMessage,
    socket: &mut WebSocket,
) -> Result<(), anyhow::Error> {
    match msg {
        ClientMessage::GetState => {
            send_current_state(state, socket).await?;
        }
        ClientMessage::ListDir { path } => {
            let cwd = state.browse_cwd.lock().await.clone();
            let rel = path.unwrap_or(cwd);
            let entries = list_dir_entries(&state.root, &rel)?;
            *state.browse_cwd.lock().await = normalize_rel(&rel);
            send_msg(
                socket,
                &ServerMessage::DirListing {
                    path: normalize_rel(&rel),
                    entries,
                },
            )
            .await?;
        }
        ClientMessage::OpenNotebook { path } => {
            let abs = resolve_under_root(&state.root, &path)?;
            if !abs.is_file() {
                bail!("not a file: {}", abs.display());
            }
            if abs.extension().and_then(|e| e.to_str()) != Some("rs") {
                bail!("open a `.rs` notebook file");
            }
            state.active_run.cancel();
            state.active_run.finish();
            let mut session = Session::open(&abs)?;
            session.active_run = state.active_run.clone();
            session.set_auto_react(*state.auto_react.lock().await);
            *state.session.lock().await = Some(session);
            let guard = state.session.lock().await;
            send_msg(socket, &full_state(guard.as_ref().unwrap())).await?;
        }
        ClientMessage::CreateNotebook { name, dir } => {
            let cwd = state.browse_cwd.lock().await.clone();
            let dir_rel = dir.unwrap_or(cwd);
            let stem = name.trim().trim_end_matches(".rs");
            if stem.is_empty() || stem.contains('/') || stem.contains('\\') || stem.contains("..") {
                bail!("invalid notebook name");
            }
            let abs_dir = resolve_under_root(&state.root, &dir_rel)?;
            if !abs_dir.is_dir() {
                bail!("not a directory: {}", abs_dir.display());
            }
            let file = abs_dir.join(format!("{stem}.rs"));
            if file.exists() {
                bail!("{} already exists", file.display());
            }
            fs::write(&file, scaffold_notebook(stem))?;
            state.active_run.cancel();
            state.active_run.finish();
            let mut session = Session::open(&file)?;
            session.active_run = state.active_run.clone();
            session.set_auto_react(*state.auto_react.lock().await);
            *state.session.lock().await = Some(session);
            let guard = state.session.lock().await;
            send_msg(socket, &full_state(guard.as_ref().unwrap())).await?;
        }
        ClientMessage::CloseNotebook => {
            state.active_run.cancel();
            state.active_run.finish();
            *state.session.lock().await = None;
            let welcome = welcome_message(state).await;
            send_msg(socket, &welcome).await?;
        }
        ClientMessage::SetAuto { enabled } => {
            *state.auto_react.lock().await = enabled;
            let mut guard = state.session.lock().await;
            if let Some(session) = guard.as_mut() {
                session.set_auto_react(enabled);
                send_msg(socket, &full_state(session)).await?;
            } else {
                drop(guard);
                let welcome = welcome_message(state).await;
                send_msg(socket, &welcome).await?;
            }
        }
        other => {
            let mut guard = state.session.lock().await;
            let session = guard
                .as_mut()
                .context("no notebook open — open or create one first")?;
            handle_session_msg(session, other, socket).await?;
        }
    }
    Ok(())
}

async fn handle_session_msg(
    session: &mut Session,
    msg: ClientMessage,
    socket: &mut WebSocket,
) -> Result<(), anyhow::Error> {
    match msg {
        ClientMessage::EditCell { name, source } => {
            let formatted = session.edit_cell(&name, &source)?;
            send_msg(
                socket,
                &ServerMessage::CellFormatted {
                    name: name.clone(),
                    source: formatted,
                },
            )
            .await?;
            send_msg(socket, &full_state(session)).await?;
        }
        ClientMessage::EditHelper { name, source } => {
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
                run_dirty_streaming(session, socket).await?;
            } else {
                send_msg(socket, &full_state(session)).await?;
            }
        }
        ClientMessage::EditDefinition { name, source } => {
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
                run_dirty_streaming(session, socket).await?;
            } else {
                send_msg(socket, &full_state(session)).await?;
            }
        }
        ClientMessage::EditPreamble { source } => {
            let formatted = session.edit_preamble(&source)?;
            send_msg(
                socket,
                &ServerMessage::PreambleFormatted { source: formatted },
            )
            .await?;
            if session.auto_react {
                run_dirty_streaming(session, socket).await?;
            } else {
                send_msg(socket, &full_state(session)).await?;
            }
        }
        ClientMessage::EditMarkdown { name, content } => {
            session.edit_markdown(&name, &content)?;
            send_msg(socket, &full_state(session)).await?;
        }
        ClientMessage::AddItem {
            kind,
            after_kind,
            after_name,
        } => {
            let add = AddKind::parse(&kind)?;
            let after = match (after_kind.as_deref(), after_name.as_deref()) {
                (None, None) => None,
                (Some("__start__"), _) | (_, Some("__start__")) => {
                    Some((AddKind::Cell, "__start__".into()))
                }
                (Some(k), Some(n)) => Some((AddKind::parse(k)?, n.to_string())),
                _ => bail!("after_kind and after_name must both be set or both omitted"),
            };
            let _name = session.add_item(add, after)?;
            send_msg(socket, &full_state(session)).await?;
        }
        ClientMessage::ChangeKind { name, from, to } => {
            session.change_kind(&name, AddKind::parse(&from)?, AddKind::parse(&to)?)?;
            send_msg(socket, &full_state(session)).await?;
        }
        ClientMessage::DeleteItem { kind, name } => {
            session.delete_item(AddKind::parse(&kind)?, &name)?;
            send_msg(socket, &full_state(session)).await?;
        }
        ClientMessage::MoveItem {
            kind,
            name,
            direction,
        } => {
            let dir = match direction.as_str() {
                "up" => MoveDirection::Up,
                "down" => MoveDirection::Down,
                other => bail!("unknown direction `{other}`"),
            };
            session.move_item(AddKind::parse(&kind)?, &name, dir)?;
            send_msg(socket, &full_state(session)).await?;
        }
        ClientMessage::RunCell { name } => {
            if let Some(cell) = session.notebook.cell(&name).cloned() {
                let bare = strip_labrs_attrs(&cell.source);
                let for_disk = with_labrs_attr(&bare, "cell");
                let formatted = rustfmt_cell_source(&for_disk);
                if strip_labrs_attrs(&formatted) != bare {
                    session.edit_cell(&name, &bare)?;
                }
            }
            run_reactive_streaming(session, &name, socket).await?;
        }
        ClientMessage::StopCell { name } => {
            let current = session.active_run.current_cell();
            let matched = current.as_deref() == Some(name.as_str()) || current.is_some();
            if matched {
                let stopped = session.active_run.cancel().unwrap_or(name);
                send_msg(
                    socket,
                    &ServerMessage::CellStopped { name: stopped },
                )
                .await?;
            }
        }
        ClientMessage::RunAll => {
            let names: Vec<String> = session
                .notebook
                .cells
                .iter()
                .map(|c| c.name.clone())
                .collect();
            for name in names {
                session.dirty.insert(name, true);
            }
            run_dirty_streaming(session, socket).await?;
        }
        ClientMessage::ClearOutputs => {
            session.clear_outputs();
            send_msg(socket, &full_state(session)).await?;
        }
        ClientMessage::Reload => {
            session.reload()?;
            send_msg(socket, &full_state(session)).await?;
        }
        _ => bail!("unexpected message for open notebook"),
    }
    Ok(())
}

fn scaffold_notebook(stem: &str) -> String {
    format!(
        r##"//! # {stem}
//!
//! A labrs notebook. Cells are bindings; plain functions are helpers.

use labrs::prelude::*;

/// Helper: reusable logic (not a notebook binding)
fn double(val: u16) -> u16 {{
    2 * val
}}

#[labrs::markdown]
pub const intro: &str = r#"# Welcome to labrs

Cells are named bindings. Helpers are plain functions."#;

/// Input value
#[labrs::cell]
pub fn val() -> u16 {{
    4
}}

/// Report using the helper and the `val` cell
#[labrs::cell]
pub fn report(val: &u16) -> String {{
    let double_val = double(*val);
    let msg = format!("Double of {{val}} is {{double_val}}");
    println!("{{msg}}");
    msg
}}
"##
    )
}

fn normalize_rel(path: &str) -> String {
    let p = path.trim().trim_start_matches("./").trim_matches('/');
    if p.is_empty() || p == "." {
        String::new()
    } else {
        p.replace('\\', "/")
    }
}

fn resolve_under_root(root: &Path, rel: &str) -> Result<PathBuf> {
    let rel = normalize_rel(rel);
    let candidate = if rel.is_empty() {
        root.to_path_buf()
    } else {
        root.join(&rel)
    };
    let canon = candidate
        .canonicalize()
        .with_context(|| format!("path not found: {}", candidate.display()))?;
    if !canon.starts_with(root) {
        bail!("path escapes workspace root");
    }
    // Reject odd components in relative form
    for c in Path::new(&rel).components() {
        match c {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => bail!("invalid path component"),
        }
    }
    Ok(canon)
}

fn list_dir_entries(root: &Path, rel: &str) -> Result<Vec<DirEntry>> {
    let abs = resolve_under_root(root, rel)?;
    if !abs.is_dir() {
        bail!("not a directory");
    }
    let rel_norm = normalize_rel(rel);
    let mut entries = Vec::new();
    let mut read: Vec<_> = fs::read_dir(&abs)?.filter_map(|e| e.ok()).collect();
    read.sort_by_key(|e| {
        (
            !e.path().is_dir(),
            e.file_name().to_string_lossy().to_lowercase(),
        )
    });
    for e in read {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let is_dir = e.path().is_dir();
        let is_notebook = !is_dir && name.ends_with(".rs");
        if !is_dir && !is_notebook {
            continue;
        }
        let path = if rel_norm.is_empty() {
            name.clone()
        } else {
            format!("{rel_norm}/{name}")
        };
        entries.push(DirEntry {
            name,
            path,
            is_dir,
            is_notebook,
        });
    }
    Ok(entries)
}

async fn run_reactive_streaming(
    session: &mut Session,
    name: &str,
    socket: &mut WebSocket,
) -> Result<(), anyhow::Error> {
    session.active_run.clear_stop();
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

    if first.error.as_deref() == Some("cancelled") || session.active_run.should_stop() {
        send_msg(
            socket,
            &ServerMessage::CellStopped {
                name: name.to_string(),
            },
        )
        .await?;
        session.active_run.clear_stop();
        send_msg(socket, &full_state(session)).await?;
        return Ok(());
    }

    if session.auto_react && first.success && changed {
        let cascade = transitive_dependents(&session.graph, name);
        for dep_name in cascade {
            if session.active_run.should_stop() {
                break;
            }
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
                    let cancelled = out.error.as_deref() == Some("cancelled");
                    send_msg(socket, &ServerMessage::CellOutput { output: out }).await?;
                    if cancelled || session.active_run.should_stop() {
                        send_msg(
                            socket,
                            &ServerMessage::CellStopped {
                                name: dep_name.clone(),
                            },
                        )
                        .await?;
                        break;
                    }
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

    session.active_run.clear_stop();

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
    session.active_run.clear_stop();
    let order = session.graph.order.clone();
    for name in order {
        if session.active_run.should_stop() {
            break;
        }
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
                let cancelled = out.error.as_deref() == Some("cancelled");
                send_msg(socket, &ServerMessage::CellOutput { output: out }).await?;
                if cancelled || session.active_run.should_stop() {
                    send_msg(
                        socket,
                        &ServerMessage::CellStopped {
                            name: name.clone(),
                        },
                    )
                    .await?;
                    break;
                }
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

    session.active_run.clear_stop();

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
    let (lsp_root, lsp_document) = lsp::lsp_paths(&session.path);
    lsp::sync_scratch_notebook(&session.path, &session.notebook.source);
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
                span: c.span,
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
                span: h.span,
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
                span: m.span,
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
                span: d.span,
            })
            .collect(),
        lsp_root: lsp_root.display().to_string(),
        lsp_document: lsp_document.display().to_string(),
    }
}

async fn send_msg(socket: &mut WebSocket, msg: &ServerMessage) -> Result<(), axum::Error> {
    let text = serde_json::to_string(msg).unwrap();
    socket.send(Message::Text(text.into())).await
}
