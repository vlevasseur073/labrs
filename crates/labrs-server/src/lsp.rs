//! WebSocket proxy for rust-analyzer (Language Server Protocol).
//!
//! Browser Monaco clients speak JSON-RPC over `/lsp`; this module frames those
//! messages for rust-analyzer's stdio transport (`Content-Length` headers).

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// Ensure an isolated Cargo package under `.labrs/lsp/` for notebooks that are
/// not already their own crate root.
///
/// Critical: the package must declare its own `[workspace]` so Cargo does not
/// attach it to a parent workspace (e.g. the labrs repo itself). Without that,
/// `cargo metadata` fails and rust-analyzer cannot load std / deps.
pub fn ensure_lsp_scratch(notebook_path: &Path) -> PathBuf {
    let parent = notebook_path.parent().unwrap_or_else(|| Path::new("."));
    let lsp_dir = parent.join(".labrs").join("lsp");
    let _ = std::fs::create_dir_all(&lsp_dir);

    // Tiny path-dep crate named `labrs` so `use labrs::prelude::*` resolves.
    // (Cannot path-depend on the real workspace member: manifests use
    // `*.workspace = true` inheritance.)
    let stub_dir = lsp_dir.join("labrs_stub");
    let _ = std::fs::create_dir_all(stub_dir.join("src"));
    let _ = std::fs::write(
        stub_dir.join("Cargo.toml"),
        r#"[package]
name = "labrs"
version = "0.0.0"
edition = "2021"
publish = false
"#,
    );
    let _ = std::fs::write(
        stub_dir.join("src").join("lib.rs"),
        r#"//! Stub `labrs` for rust-analyzer in notebook scratch packages.
pub mod prelude {}

/// Passthrough attribute stubs (proc-macros are disabled in the LSP client).
pub use prelude::*;
"#,
    );

    let cargo = lsp_dir.join("Cargo.toml");
    let toml = r#"[package]
name = "labrs_lsp_scratch"
version = "0.0.0"
edition = "2021"
publish = false

# Isolate from any parent Cargo workspace (required for rust-analyzer).
[workspace]

[[bin]]
name = "labrs_lsp_scratch"
path = "notebook.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
labrs = { path = "labrs_stub" }
"#;
    let _ = std::fs::write(&cargo, toml);

    let nb = lsp_dir.join("notebook.rs");
    if !nb.is_file() {
        let _ = std::fs::write(&nb, "fn main() {}\n");
    }
    lsp_dir
}

/// Copy the live notebook source into the scratch `notebook.rs` so on-disk
/// content matches what the client opens via LSP (helps cargo/RA indexing).
pub fn sync_scratch_notebook(notebook_path: &Path, source: &str) {
    let (_, doc) = lsp_paths(notebook_path);
    // Only write when using the scratch document, not the real notebook path.
    let is_scratch = doc.file_name().and_then(|f| f.to_str()) == Some("notebook.rs")
        && doc
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
            == Some("lsp");
    if is_scratch {
        let _ = std::fs::write(&doc, source);
    }
}

/// Resolve workspace root and document URI path for the notebook.
pub fn lsp_paths(notebook_path: &Path) -> (PathBuf, PathBuf) {
    let notebook_path = notebook_path
        .canonicalize()
        .unwrap_or_else(|_| notebook_path.to_path_buf());
    let parent = notebook_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let sibling_cargo = parent.join("Cargo.toml");
    if sibling_cargo.is_file() {
        if let Ok(text) = std::fs::read_to_string(&sibling_cargo) {
            if text.contains("[package]") {
                let own_workspace = text.contains("[workspace]");
                let ancestor_ws = find_ancestor_workspace(&parent);
                // Use the notebook's own crate only when Cargo can load it:
                // either it declares [workspace], or it is not nested under another.
                if own_workspace || ancestor_ws.is_none() {
                    return (parent, notebook_path);
                }
            }
        }
    }

    let root = ensure_lsp_scratch(&notebook_path);
    let doc = root.join("notebook.rs");
    (root, doc)
}

/// Find a parent directory that defines a Cargo `[workspace]` (excluding `dir` itself).
fn find_ancestor_workspace(dir: &Path) -> Option<PathBuf> {
    for anc in dir.ancestors().skip(1) {
        let cargo = anc.join("Cargo.toml");
        if let Ok(text) = std::fs::read_to_string(&cargo) {
            if text.contains("[workspace]") {
                return Some(anc.to_path_buf());
            }
        }
        // Stop at filesystem root
        if anc.parent().is_none() {
            break;
        }
    }
    None
}

fn find_rust_analyzer() -> Option<PathBuf> {
    which::which("rust-analyzer").ok()
}

/// Handle one LSP WebSocket session: spawn rust-analyzer and bidirectional proxy.
pub async fn handle_lsp_websocket(socket: WebSocket, notebook_path: PathBuf) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let Some(ra_path) = find_rust_analyzer() else {
        tracing::error!("rust-analyzer not found on PATH");
        let error_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "window/showMessage",
            "params": {
                "type": 1,
                "message": "rust-analyzer not found on PATH. Install it (e.g. rustup component add rust-analyzer) for code intelligence."
            }
        });
        let _ = ws_sender
            .send(Message::Text(error_msg.to_string().into()))
            .await;
        return;
    };

    // Refresh scratch package + seed notebook.rs from the real file when possible.
    if let Ok(source) = std::fs::read_to_string(&notebook_path) {
        let _ = lsp_paths(&notebook_path); // ensure scratch exists / Cargo.toml refreshed
        sync_scratch_notebook(&notebook_path, &source);
    }

    let (workspace_root, doc_path) = lsp_paths(&notebook_path);
    tracing::info!(
        "LSP: rust-analyzer={} workspace={} doc={}",
        ra_path.display(),
        workspace_root.display(),
        doc_path.display()
    );

    // Verify cargo can see the package (log only).
    match std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(&workspace_root)
        .output()
    {
        Ok(out) if !out.status.success() => {
            let err = String::from_utf8_lossy(&out.stderr);
            tracing::warn!("LSP cargo metadata failed: {}", err.trim());
        }
        Err(e) => tracing::warn!("LSP cargo metadata error: {e}"),
        _ => {}
    }

    let mut child = match Command::new(&ra_path)
        .current_dir(&workspace_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("failed to start rust-analyzer: {e}");
            let error_msg = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "window/showMessage",
                "params": {
                    "type": 1,
                    "message": format!("Failed to start rust-analyzer: {e}")
                }
            });
            let _ = ws_sender
                .send(Message::Text(error_msg.to_string().into()))
                .await;
            return;
        }
    };

    let stdin = child.stdin.take().expect("rust-analyzer stdin");
    let stdout = child.stdout.take().expect("rust-analyzer stdout");
    let stderr = child.stderr.take().expect("rust-analyzer stderr");

    let stdin = Arc::new(Mutex::new(stdin));
    let stdin_w = stdin.clone();

    let ws_to_lsp = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let content = text.as_str();
                    let header = format!("Content-Length: {}\r\n\r\n", content.len());
                    let mut stdin = stdin_w.lock().await;
                    if stdin.write_all(header.as_bytes()).await.is_err() {
                        break;
                    }
                    if stdin.write_all(content.as_bytes()).await.is_err() {
                        break;
                    }
                    if stdin.flush().await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    let ws_sender = Arc::new(Mutex::new(ws_sender));
    let ws_sender_out = ws_sender.clone();

    let lsp_to_ws = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        loop {
            match read_lsp_message(&mut reader).await {
                Ok(Some(text)) => {
                    let mut sender = ws_sender_out.lock().await;
                    if sender.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }
    });

    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while reader.read_line(&mut line).await.is_ok() {
            if line.is_empty() {
                break;
            }
            tracing::debug!("rust-analyzer: {}", line.trim());
            line.clear();
        }
    });

    let _ = tokio::join!(ws_to_lsp, lsp_to_ws);
    stderr_task.abort();
    let _ = stderr_task.await;
    kill_child(&mut child).await;
    tracing::info!("LSP session ended");
}

async fn kill_child(child: &mut Child) {
    let _ = child.kill().await;
    let _ = child.wait().await;
}

/// Read one LSP message (headers + body) from rust-analyzer stdout.
async fn read_lsp_message<R: AsyncReadExt + Unpin>(
    reader: &mut BufReader<R>,
) -> std::io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                content_length = value.trim().parse().ok();
            }
        }
    }
    let Some(len) = content_length else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "LSP message missing Content-Length",
        ));
    };
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    String::from_utf8(buf)
        .map(Some)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
