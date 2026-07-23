//! Embedded static assets for the labrs web UI.

use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};

pub async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn app_js() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        APP_JS,
    )
        .into_response()
}

pub async fn app_css() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        APP_CSS,
    )
        .into_response()
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>labrs</title>
  <link rel="stylesheet" href="/app.css" />
  <link rel="stylesheet" data-name="vs/editor/editor.main"
    href="https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs/editor/editor.main.css" />
</head>
<body>
  <header class="topbar">
    <div class="brand">labrs</div>
    <div class="actions">
      <label class="auto-toggle" title="Automatically re-run dependent cells when an upstream output changes">
        <input type="checkbox" id="chk-auto" checked />
        Auto-run
      </label>
      <button id="btn-run-all" class="primary">Run all</button>
      <button id="btn-reload">Reload</button>
      <span id="conn" class="conn">connecting…</span>
    </div>
  </header>
  <aside id="diag" class="diag hidden"></aside>
  <main id="notebook" class="notebook"></main>
  <script>var require = { paths: { vs: "https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs" } };</script>
  <script src="https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs/loader.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs/editor/editor.main.js"></script>
  <script src="/app.js"></script>
</body>
</html>
"#;

const APP_CSS: &str = r#"
:root {
  --bg: #f4f0e8;
  --surface: #fffdf8;
  --ink: #1c1917;
  --muted: #78716c;
  --accent: #0f766e;
  --accent-2: #b45309;
  --border: #d6d3d1;
  --dirty: #ca8a04;
  --error: #b91c1c;
  --ok: #15803d;
  --helper: #1d4ed8;
  --mono: "IBM Plex Mono", "JetBrains Mono", ui-monospace, monospace;
  --sans: "IBM Plex Sans", "Source Sans 3", system-ui, sans-serif;
}
* { box-sizing: border-box; }
html, body { margin: 0; padding: 0; background: var(--bg); color: var(--ink); font-family: var(--sans); }
body {
  min-height: 100vh;
  background:
    radial-gradient(1200px 600px at 10% -10%, #ccfbf1 0%, transparent 55%),
    radial-gradient(900px 500px at 100% 0%, #fed7aa 0%, transparent 50%),
    var(--bg);
}
.topbar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.75rem 1.25rem; position: sticky; top: 0; z-index: 10;
  backdrop-filter: blur(8px); background: color-mix(in srgb, var(--bg) 80%, transparent);
  border-bottom: 1px solid var(--border); gap: 1rem; flex-wrap: wrap;
}
.brand { font-weight: 700; letter-spacing: -0.03em; font-size: 1.25rem; color: var(--accent); }
.actions { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
.auto-toggle {
  display: flex; align-items: center; gap: 0.35rem; font-size: 0.9rem; color: var(--muted);
  user-select: none; cursor: pointer; margin-right: 0.25rem;
}
.auto-toggle input { accent-color: var(--accent); }
button, select.kind-select {
  font: inherit; border: 1px solid var(--border); background: var(--surface);
  padding: 0.35rem 0.7rem; border-radius: 6px; cursor: pointer; color: var(--ink);
}
button:hover, select.kind-select:hover { border-color: var(--accent); }
button.primary { background: var(--accent); color: white; border-color: var(--accent); }
button.primary:hover { filter: brightness(1.05); }
select.kind-select { font-size: 0.8rem; text-transform: capitalize; }
.conn { font-size: 0.8rem; color: var(--muted); margin-left: 0.5rem; }
.conn.ok { color: var(--ok); }
.conn.bad { color: var(--error); }
.diag {
  margin: 0.75rem 1.25rem; padding: 0.75rem 1rem; border-radius: 8px;
  background: #fef2f2; border: 1px solid #fecaca; color: var(--error); font-family: var(--mono); font-size: 0.85rem;
}
.diag.hidden { display: none; }
.notebook { max-width: 920px; margin: 0 auto; padding: 1rem 1.25rem 3rem; display: flex; flex-direction: column; gap: 0; }
.card {
  background: var(--surface); border: 1px solid var(--border); border-radius: 10px;
  overflow: hidden; box-shadow: 0 1px 0 rgba(28,25,23,0.04);
}
.card.dirty { border-left: 4px solid var(--dirty); }
.card.error { border-left: 4px solid var(--error); }
.card.success { border-left: 4px solid var(--ok); }
.card.helper { border-left: 4px solid var(--helper); }
.card.md { border-left: 4px solid #a8a29e; }
.card-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.55rem 0.85rem; border-bottom: 1px solid var(--border); gap: 0.75rem;
}
.card-title { font-weight: 600; font-family: var(--mono); font-size: 0.9rem; }
.meta { color: var(--muted); font-size: 0.8rem; }
.badge {
  font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.04em;
  color: var(--muted); border: 1px solid var(--border); padding: 0.15rem 0.45rem; border-radius: 999px;
  white-space: nowrap;
}
.badge.success { color: var(--ok); border-color: color-mix(in srgb, var(--ok) 40%, var(--border)); }
.badge.dirty { color: var(--dirty); border-color: color-mix(in srgb, var(--dirty) 45%, var(--border)); }
.badge.error { color: var(--error); border-color: color-mix(in srgb, var(--error) 40%, var(--border)); }
.badge.pristine { color: var(--muted); }
.badge.running { color: var(--accent); border-color: color-mix(in srgb, var(--accent) 45%, var(--border)); }
@keyframes labrs-spin { to { transform: rotate(360deg); } }
.badge.running .spin {
  display: inline-block;
  animation: labrs-spin 0.75s linear infinite;
  margin-right: 0.25rem;
}
.card.running { border-left: 4px solid var(--accent); }
.docs { padding: 0.5rem 0.85rem; color: var(--muted); font-size: 0.9rem; white-space: pre-wrap; }
.editor { height: 160px; border-bottom: 1px solid var(--border); }
.editor.tall { height: 220px; }
.editor.md-edit { height: 140px; }
.panels { display: grid; grid-template-columns: 1fr 1fr; gap: 0; }
@media (max-width: 720px) { .panels { grid-template-columns: 1fr; } }
.panel { padding: 0.65rem 0.85rem; min-height: 4rem; }
.panel + .panel { border-left: 1px solid var(--border); }
@media (max-width: 720px) { .panel + .panel { border-left: none; border-top: 1px solid var(--border); } }
.panel h4 { margin: 0 0 0.35rem; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); }
.panel pre {
  margin: 0; font-family: var(--mono); font-size: 0.82rem; white-space: pre-wrap; word-break: break-word;
}
.md-body { padding: 1rem 1.1rem; line-height: 1.55; }
.md-body h1, .md-body h2, .md-body h3 { margin-top: 0; }
.md-body code { font-family: var(--mono); background: #f5f5f4; padding: 0.1em 0.35em; border-radius: 4px; }
.md-edit-wrap { display: none; }
.md-edit-wrap.open { display: block; }
.md-body.hidden { display: none; }
.insert-gap {
  position: relative; height: 1.1rem; margin: 0.15rem 0;
  display: flex; align-items: center; justify-content: center;
}
.insert-gap .plus {
  opacity: 0; width: 1.5rem; height: 1.5rem; border-radius: 999px;
  border: 1px solid var(--border); background: var(--surface);
  color: var(--accent); font-size: 1rem; line-height: 1; padding: 0;
  display: flex; align-items: center; justify-content: center;
  transition: opacity 0.12s ease;
  z-index: 2;
}
.insert-gap:hover .plus, .insert-gap.open .plus { opacity: 1; }
.insert-gap::before {
  content: ""; position: absolute; left: 8%; right: 8%; height: 1px;
  background: transparent; transition: background 0.12s;
}
.insert-gap:hover::before, .insert-gap.open::before { background: var(--border); }
.insert-menu {
  display: none; position: absolute; top: 100%; margin-top: 0.25rem;
  background: var(--surface); border: 1px solid var(--border); border-radius: 8px;
  box-shadow: 0 6px 20px rgba(28,25,23,0.08); padding: 0.25rem; z-index: 5;
  gap: 0.2rem;
}
.insert-gap.open .insert-menu { display: flex; }
.insert-menu button { border: none; background: transparent; padding: 0.35rem 0.75rem; border-radius: 5px; }
.insert-menu button:hover { background: #f5f5f4; }
.footer-add {
  margin-top: 1.25rem; padding: 0.85rem; border: 1px dashed var(--border);
  border-radius: 10px; display: flex; gap: 0.5rem; flex-wrap: wrap; justify-content: center;
  background: color-mix(in srgb, var(--surface) 70%, transparent);
}
.footer-add button { min-width: 7rem; }
"#;

const APP_JS: &str = r#"
(() => {
  const notebookEl = document.getElementById("notebook");
  const diagEl = document.getElementById("diag");
  const connEl = document.getElementById("conn");
  const editors = new Map();
  let state = null;
  let ws;
  const mdEditing = new Set();
  let openInsertKey = null;
  let pendingFocus = null; // { key, selection }
  const runningCells = new Set();

  function connect() {
    const proto = location.protocol === "https:" ? "wss" : "ws";
    ws = new WebSocket(`${proto}://${location.host}/ws`);
    ws.onopen = () => {
      connEl.textContent = "connected";
      connEl.className = "conn ok";
      send({ type: "get_state" });
    };
    ws.onclose = () => {
      connEl.textContent = "disconnected — retrying…";
      connEl.className = "conn bad";
      setTimeout(connect, 1500);
    };
    ws.onerror = () => { connEl.className = "conn bad"; };
    ws.onmessage = (ev) => {
      const msg = JSON.parse(ev.data);
      handleServer(msg);
    };
  }

  function send(obj) {
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify(obj));
  }

  function captureViewState() {
    const scrollY = window.scrollY;
    let focusKey = pendingFocus && pendingFocus.key;
    let selection = pendingFocus && pendingFocus.selection;
    if (!focusKey) {
      editors.forEach((ed, key) => {
        if (ed.hasTextFocus && ed.hasTextFocus()) {
          focusKey = key;
          selection = ed.getSelection();
        }
      });
    }
    const pending = new Map();
    editors.forEach((ed, name) => pending.set(name, {
      value: ed.getValue(),
      selection: ed.getSelection(),
    }));
    return { scrollY, focusKey, selection, pending };
  }

  function restoreViewState(vs) {
    requestAnimationFrame(() => {
      window.scrollTo(0, vs.scrollY);
      const key = vs.focusKey;
      if (key && editors.has(key)) {
        const ed = editors.get(key);
        ed.focus();
        const sel = (vs.pending.get(key) && vs.pending.get(key).selection) || vs.selection;
        if (sel) ed.setSelection(sel);
      }
      pendingFocus = null;
    });
  }

  function handleServer(msg) {
    switch (msg.type) {
      case "notebook_state":
        state = msg;
        runningCells.clear();
        const autoEl = document.getElementById("chk-auto");
        if (autoEl && msg.snapshot && typeof msg.snapshot.auto_react === "boolean") {
          autoEl.checked = msg.snapshot.auto_react;
        }
        render();
        break;
      case "cell_formatted":
      case "helper_formatted": {
        const ed = editors.get(msg.name) || editors.get("helper:" + msg.name);
        if (ed && ed.getValue() !== msg.source) {
          const sel = ed.getSelection();
          ed.setValue(msg.source);
          if (sel) ed.setSelection(sel);
        }
        break;
      }
      case "cell_running":
        runningCells.add(msg.name);
        patchCellRunning(msg.name, true);
        break;
      case "cell_output":
        runningCells.delete(msg.output.cell);
        patchCellRunning(msg.output.cell, false);
        if (state && state.snapshot && state.snapshot.cells) {
          const st = state.snapshot.cells.find(c => c.name === msg.output.cell);
          if (st) {
            st.output = msg.output;
            st.status = msg.output.success ? "success" : "error";
            st.dirty = false;
          }
          patchCellPanels(msg.output);
        }
        break;
      case "cells_dirty":
        break;
      case "error":
        diagEl.textContent = msg.message;
        diagEl.classList.remove("hidden");
        break;
    }
  }

  function patchCellRunning(name, isRunning) {
    const badge = document.querySelector(`[data-status-for="${CSS.escape(name)}"]`);
    const card = document.querySelector(`[data-cell-card="${CSS.escape(name)}"]`);
    if (badge) {
      if (isRunning) {
        badge.className = "badge running";
        badge.innerHTML = '<span class="spin" aria-hidden="true">⟳</span>running';
      } else if (state && state.snapshot) {
        const st = (state.snapshot.cells || []).find(c => c.name === name);
        const label = statusLabel(st);
        badge.className = `badge ${label}`;
        badge.textContent = label;
      }
    }
    if (card) {
      if (isRunning) card.classList.add("running");
      else card.classList.remove("running");
    }
  }

  function patchCellPanels(out) {
    const ret = document.querySelector(`[data-return-for="${CSS.escape(out.cell)}"]`);
    const logs = document.querySelector(`[data-logs-for="${CSS.escape(out.cell)}"]`);
    if (ret) ret.textContent = JSON.stringify(out.value, null, 2);
    if (logs) logs.textContent = formatLogs(out);
    const card = document.querySelector(`[data-cell-card="${CSS.escape(out.cell)}"]`);
    if (card) {
      card.classList.remove("dirty", "error", "success", "running");
      if (out.success) card.classList.add("success");
      else card.classList.add("error");
    }
  }

  function statusClass(cellState, name) {
    if (name && runningCells.has(name)) return "running";
    if (!cellState) return "";
    if (cellState.dirty) return "dirty";
    if (cellState.status === "error") return "error";
    if (cellState.status === "success") return "success";
    return "";
  }

  function statusLabel(cellState, name) {
    if (name && runningCells.has(name)) return "running";
    if (!cellState) return "pristine";
    if (cellState.dirty) return "dirty";
    return cellState.status || "pristine";
  }

  function statusBadge(cellState, name) {
    const label = statusLabel(cellState, name);
    const badge = el("span", `badge ${label}`);
    badge.setAttribute("data-status-for", name || "");
    if (label === "running") {
      badge.innerHTML = '<span class="spin" aria-hidden="true">⟳</span>running';
    } else {
      badge.textContent = label;
    }
    badge.title = "Execution status";
    return badge;
  }

  function simpleMarkdown(text) {
    return text
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/^### (.*)$/gm, "<h3>$1</h3>")
      .replace(/^## (.*)$/gm, "<h2>$1</h2>")
      .replace(/^# (.*)$/gm, "<h1>$1</h1>")
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/\*(.+?)\*/g, "<em>$1</em>")
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\n\n/g, "</p><p>")
      .replace(/\n/g, "<br/>");
  }

  function makeMonaco(host, key, value, language, onSaveCmd, pendingMap) {
    let initial = value;
    let sel = null;
    if (pendingMap && pendingMap.has(key)) {
      initial = pendingMap.get(key).value;
      sel = pendingMap.get(key).selection;
    }
    const editor = monaco.editor.create(host, {
      value: initial,
      language: language || "rust",
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 13,
      fontFamily: "IBM Plex Mono, JetBrains Mono, ui-monospace, monospace",
      tabSize: 4,
      insertSpaces: true,
      autoIndent: "full",
      scrollBeyondLastLine: false,
      wordWrap: "on",
      multiCursorModifier: "alt",
      renderLineHighlight: "line",
      padding: { top: 8, bottom: 8 },
    });
    if (sel) editor.setSelection(sel);
    if (onSaveCmd) {
      editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, onSaveCmd);
    }
    editors.set(key, editor);
    return editor;
  }

  function kindSelect(current, name, kind) {
    const sel = document.createElement("select");
    sel.className = "kind-select";
    sel.title = "Change cell type";
    for (const k of ["cell", "markdown", "helper"]) {
      const opt = document.createElement("option");
      opt.value = k;
      opt.textContent = k;
      if (k === current) opt.selected = true;
      sel.appendChild(opt);
    }
    sel.onchange = () => {
      const to = sel.value;
      if (to === current) return;
      pendingFocus = null;
      send({ type: "change_kind", name, from: current, to });
    };
    return sel;
  }

  function insertGap(afterKind, afterName, key) {
    const gap = el("div", "insert-gap" + (openInsertKey === key ? " open" : ""));
    const plus = document.createElement("button");
    plus.type = "button";
    plus.className = "plus";
    plus.textContent = "+";
    plus.title = "Insert below";
    plus.onclick = (e) => {
      e.stopPropagation();
      openInsertKey = openInsertKey === key ? null : key;
      render();
    };
    const menu = el("div", "insert-menu");
    for (const [label, kind] of [["Cell", "cell"], ["Markdown", "markdown"], ["Helper", "helper"]]) {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.onclick = (e) => {
        e.stopPropagation();
        openInsertKey = null;
        const msg = { type: "add_item", kind };
        if (afterKind && afterName) {
          msg.after_kind = afterKind;
          msg.after_name = afterName;
        }
        send(msg);
      };
      menu.appendChild(b);
    }
    gap.appendChild(plus);
    gap.appendChild(menu);
    return gap;
  }

  function footerAdd() {
    const foot = el("div", "footer-add");
    for (const [label, kind] of [["+ Cell", "cell"], ["+ Markdown", "markdown"], ["+ Helper", "helper"]]) {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.onclick = () => send({ type: "add_item", kind });
      foot.appendChild(b);
    }
    return foot;
  }

  function render() {
    if (!state) return;
    const vs = captureViewState();
    const snap = state.snapshot;
    const errors = (snap.diagnostics || []).filter(d => d.severity === "error");
    if (errors.length) {
      diagEl.innerHTML = errors.map(d => `${d.cell || ""}: ${escapeHtml(d.message)}`).join("<br/>");
      diagEl.classList.remove("hidden");
    } else {
      diagEl.classList.add("hidden");
    }

    editors.forEach(ed => ed.dispose());
    editors.clear();
    notebookEl.innerHTML = "";

    const byCell = Object.fromEntries((state.cells_detail || []).map(c => [c.name, c]));
    const byHelper = Object.fromEntries((state.helpers_detail || []).map(h => [h.name, h]));
    const byMd = Object.fromEntries((state.markdown_detail || []).map(m => [m.name, m]));
    const cellState = Object.fromEntries((snap.cells || []).map(c => [c.name, c]));

    const order = (snap.order && snap.order.length)
      ? snap.order.map(e => {
          if (e.kind === "cell") return { kind: "cell", id: e.id };
          if (e.kind === "helper") return { kind: "helper", id: e.name };
          if (e.kind === "markdown") return { kind: "markdown", id: e.id };
          return null;
        }).filter(Boolean)
      : [
          ...(state.markdown_detail || []).map(m => ({ kind: "markdown", id: m.name })),
          ...(state.helpers_detail || []).map(h => ({ kind: "helper", id: h.name })),
          ...(state.cells_detail || []).map(c => ({ kind: "cell", id: c.name })),
        ];

    // Leading insert (before first item)
    notebookEl.appendChild(insertGap("__start__", "__start__", "top"));

    order.forEach((entry, idx) => {
      if (entry.kind === "markdown") {
        const md = byMd[entry.id];
        if (!md) return;
        const card = el("div", "card md");
        const head = el("div", "card-head");
        const left = el("div", "actions");
        left.appendChild(kindSelect("markdown", md.name, "markdown"));
        const title = el("span", "card-title");
        title.textContent = " " + md.name;
        left.appendChild(title);
        const actions = el("div", "actions");
        const editBtn = document.createElement("button");
        const editing = mdEditing.has(md.name);
        editBtn.textContent = editing ? "Save" : "Edit";
        const body = el("div", editing ? "md-body hidden" : "md-body");
        body.innerHTML = `<p>${simpleMarkdown(md.content)}</p>`;
        const editWrap = el("div", editing ? "md-edit-wrap open" : "md-edit-wrap");
        const editorHost = el("div", "editor md-edit");
        editWrap.appendChild(editorHost);
        editBtn.onclick = () => {
          if (mdEditing.has(md.name)) {
            const ed = editors.get("md:" + md.name);
            const content = ed ? ed.getValue() : md.content;
            pendingFocus = { key: "md:" + md.name, selection: ed && ed.getSelection() };
            mdEditing.delete(md.name);
            send({ type: "edit_markdown", name: md.name, content });
          } else {
            mdEditing.add(md.name);
            render();
          }
        };
        actions.appendChild(editBtn);
        head.appendChild(left);
        head.appendChild(actions);
        card.appendChild(head);
        card.appendChild(body);
        card.appendChild(editWrap);
        notebookEl.appendChild(card);
        if (editing) {
          makeMonaco(editorHost, "md:" + md.name, md.content, "markdown", () => editBtn.click(), vs.pending);
        }
        notebookEl.appendChild(insertGap("markdown", md.name, "after-md-" + md.name));
      } else if (entry.kind === "helper") {
        const h = byHelper[entry.id];
        if (!h) return;
        const card = el("div", "card helper");
        const head = el("div", "card-head");
        const left = el("div", "actions");
        left.appendChild(kindSelect("helper", h.name, "helper"));
        const title = el("span", "card-title");
        title.textContent = " " + h.name;
        left.appendChild(title);
        const actions = el("div", "actions");
        const saveBtn = document.createElement("button");
        saveBtn.textContent = "Save";
        saveBtn.onclick = () => {
          const ed = editors.get("helper:" + h.name);
          pendingFocus = { key: "helper:" + h.name, selection: ed && ed.getSelection() };
          // Optimistic: mark likely dependents as running when auto is on
          if (document.getElementById("chk-auto").checked && state && state.cells_detail) {
            for (const c of state.cells_detail) {
              if ((c.source || "").includes(h.name)) {
                runningCells.add(c.name);
                patchCellRunning(c.name, true);
              }
            }
          }
          send({ type: "edit_helper", name: h.name, source: ed ? ed.getValue() : h.source });
        };
        actions.appendChild(saveBtn);
        head.appendChild(left);
        head.appendChild(actions);
        card.appendChild(head);
        if (h.docs) {
          const docs = el("div", "docs");
          docs.textContent = h.docs;
          card.appendChild(docs);
        }
        const editorHost = el("div", "editor");
        card.appendChild(editorHost);
        notebookEl.appendChild(card);
        makeMonaco(editorHost, "helper:" + h.name, h.source, "rust", () => saveBtn.click(), vs.pending);
        notebookEl.appendChild(insertGap("helper", h.name, "after-helper-" + h.name));
      } else {
        const c = byCell[entry.id];
        if (!c) return;
        const st = cellState[c.name];
        const card = el("div", `card ${statusClass(st, c.name)}`);
        card.setAttribute("data-cell-card", c.name);
        const head = el("div", "card-head");
        const left = el("div", "actions");
        left.appendChild(kindSelect("cell", c.name, "cell"));
        const title = el("div");
        title.innerHTML = `<span class="card-title"> ${escapeHtml(c.name)}</span>
          <span class="meta"> → ${escapeHtml(c.return_type)}${c.params.length ? " · deps: " + c.params.map(escapeHtml).join(", ") : ""}</span>`;
        left.appendChild(title);
        const actions = el("div", "actions");
        const runBtn = document.createElement("button");
        runBtn.className = "primary";
        runBtn.textContent = "Run";
        runBtn.onclick = () => {
          const ed = editors.get(c.name);
          const source = ed ? ed.getValue() : c.source;
          pendingFocus = { key: c.name, selection: ed && ed.getSelection() };
          runningCells.add(c.name);
          patchCellRunning(c.name, true);
          send({ type: "edit_cell", name: c.name, source });
          setTimeout(() => send({ type: "run_cell", name: c.name }), 50);
        };
        actions.appendChild(runBtn);
        actions.appendChild(statusBadge(st, c.name));
        head.appendChild(left);
        head.appendChild(actions);
        card.appendChild(head);
        if (c.docs) {
          const docs = el("div", "docs");
          docs.textContent = c.docs;
          card.appendChild(docs);
        }
        const editorHost = el("div", "editor tall");
        card.appendChild(editorHost);
        const panels = el("div", "panels");
        const out = st && st.output;
        panels.innerHTML = `
          <div class="panel"><h4>Return</h4><pre data-return-for="${escapeHtml(c.name)}">${escapeHtml(out ? JSON.stringify(out.value, null, 2) : "—")}</pre></div>
          <div class="panel"><h4>Logs</h4><pre data-logs-for="${escapeHtml(c.name)}">${escapeHtml(formatLogs(out))}</pre></div>`;
        card.appendChild(panels);
        notebookEl.appendChild(card);
        makeMonaco(editorHost, c.name, c.source, "rust", () => runBtn.click(), vs.pending);
        notebookEl.appendChild(insertGap("cell", c.name, "after-cell-" + c.name));
      }
    });

    notebookEl.appendChild(footerAdd());
    restoreViewState(vs);

    // close insert menus on outside click
    document.onclick = () => {
      if (openInsertKey) {
        openInsertKey = null;
        // don't full render — just close menus
        notebookEl.querySelectorAll(".insert-gap.open").forEach(n => n.classList.remove("open"));
      }
    };
  }

  function formatLogs(out) {
    if (!out) return "—";
    const parts = [];
    if (out.stdout) parts.push("[stdout]\n" + out.stdout);
    if (out.stderr) parts.push("[stderr]\n" + out.stderr);
    if (out.error) parts.push("[error]\n" + out.error);
    return parts.join("\n") || "(no output)";
  }

  function el(tag, cls) {
    const n = document.createElement(tag);
    if (cls) n.className = cls;
    return n;
  }
  function escapeHtml(s) {
    return String(s ?? "").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");
  }

  document.getElementById("btn-run-all").onclick = () => send({ type: "run_all" });
  document.getElementById("btn-reload").onclick = () => send({ type: "reload" });
  document.getElementById("chk-auto").onchange = (e) => {
    send({ type: "set_auto", enabled: !!e.target.checked });
  };

  require(["vs/editor/editor.main"], () => connect());
})();
"#;
