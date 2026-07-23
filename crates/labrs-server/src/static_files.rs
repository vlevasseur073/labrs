//! Embedded static assets for the labrs web UI.

use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};

pub async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn app_js() -> Response {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
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
    <div class="brand-wrap">
      <button id="btn-toggle-inspector" class="icon menu-btn" type="button" title="Toggle inspector" aria-label="Toggle inspector" aria-controls="inspector" aria-expanded="true">
        <span class="menu-icon" aria-hidden="true"></span>
      </button>
      <button type="button" class="brand brand-btn" id="btn-home" title="Home / files">labrs</button>
    </div>
    <div class="actions" id="notebook-actions">
      <label class="auto-toggle" title="Automatically re-run dependent cells when an upstream output changes">
        <input type="checkbox" id="chk-auto" checked />
        Auto-run
      </label>
      <button id="btn-run-all" class="primary">Run all</button>
      <button id="btn-reload">Reload</button>
      <span id="conn" class="conn">connecting…</span>
    </div>
  </header>
  <div id="welcome" class="welcome hidden">
    <aside class="file-browser" aria-label="Files">
      <div class="file-browser-head">
        <div class="file-browser-title">Files</div>
        <div id="file-cwd" class="file-cwd"></div>
      </div>
      <div id="file-list" class="file-list"></div>
    </aside>
    <main class="welcome-main">
      <div class="welcome-hero">
        <div class="welcome-brand">labrs</div>
        <h1 class="welcome-title">Open a notebook</h1>
        <p class="welcome-sub">Browse <code>.rs</code> files on the left, or create a new reactive notebook.</p>
        <div class="welcome-actions">
          <button type="button" class="primary" id="btn-new-notebook">New notebook</button>
        </div>
        <p class="welcome-hint">Root: <span id="welcome-root"></span></p>
      </div>
    </main>
  </div>
  <div class="workspace" id="workspace">
    <aside id="inspector" class="inspector" aria-label="Sidebar"></aside>
    <div class="workspace-main">
      <aside id="diag" class="diag hidden"></aside>
      <div class="main-chrome">
        <div class="main-tabs" role="tablist" aria-label="Main panes">
          <button type="button" class="main-tab active" id="tab-notebook" role="tab" aria-selected="true" data-main-tab="notebook">Notebook</button>
          <button type="button" class="main-tab" id="tab-shared" role="tab" aria-selected="false" data-main-tab="shared">Shared</button>
        </div>
        <button type="button" class="detach-btn" id="btn-detach-shared" title="Show Shared beside Notebook">Detach</button>
      </div>
      <div class="main-panes" id="main-panes">
        <main id="notebook" class="notebook pane" role="tabpanel" aria-labelledby="tab-notebook"></main>
        <div class="split-handle" id="split-handle" aria-hidden="true" title="Drag to resize"></div>
        <section id="shared" class="shared pane" role="tabpanel" aria-labelledby="tab-shared" hidden></section>
      </div>
    </div>
  </div>
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
  --accent-soft: #ccfbf1;
  --accent-2: #b45309;
  --border: #e7e5e4;
  --border-strong: #d6d3d1;
  --dirty: #ca8a04;
  --error: #b91c1c;
  --ok: #15803d;
  --helper: #1d4ed8;
  --md: #78716c;
  --mono: "IBM Plex Mono", "JetBrains Mono", ui-monospace, monospace;
  --sans: "IBM Plex Sans", "Source Sans 3", system-ui, sans-serif;
  --radius: 12px;
  --shadow: 0 1px 2px rgba(28,25,23,0.04), 0 8px 24px rgba(28,25,23,0.04);
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
/* Detached: app fills viewport; each main pane scrolls on its own */
body.split-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  max-height: 100vh;
  overflow: hidden;
}
body.split-layout .topbar { flex-shrink: 0; position: relative; }
body.split-layout .workspace {
  flex: 1 1 auto;
  min-height: 0;
  height: auto;
  max-height: none;
  overflow: hidden;
}
.topbar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.85rem 1.5rem; position: sticky; top: 0; z-index: 10;
  backdrop-filter: blur(12px); background: color-mix(in srgb, var(--bg) 78%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--border) 80%, transparent); gap: 1rem; flex-wrap: wrap;
}
.brand-wrap { display: flex; align-items: center; gap: 0.45rem; min-width: 0; }
.brand { font-weight: 700; letter-spacing: -0.04em; font-size: 1.3rem; color: var(--accent); }
.brand-btn {
  font: inherit; font-weight: 700; letter-spacing: -0.04em; font-size: 1.3rem; color: var(--accent);
  border: none; background: transparent; padding: 0; cursor: pointer;
}
.brand-btn:hover { filter: brightness(0.92); }
.welcome {
  display: grid;
  grid-template-columns: minmax(240px, 300px) minmax(0, 1fr);
  min-height: calc(100vh - 3.5rem);
  align-items: stretch;
}
.welcome.hidden { display: none; }
body.welcome-mode .workspace { display: none !important; }
body.welcome-mode .menu-btn { display: none; }
body.welcome-mode #notebook-actions .auto-toggle,
body.welcome-mode #btn-run-all,
body.welcome-mode #btn-reload { display: none; }
body:not(.welcome-mode) #welcome { display: none !important; }
.file-browser {
  border-right: 1px solid var(--border);
  background: color-mix(in srgb, var(--surface) 78%, transparent);
  backdrop-filter: blur(8px);
  padding: 1rem 0.85rem 2rem;
  overflow: auto;
}
.file-browser-head { margin-bottom: 0.75rem; padding: 0 0.25rem; }
.file-browser-title {
  font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.08em;
  color: var(--muted); font-weight: 700;
}
.file-cwd {
  margin-top: 0.25rem; font-family: var(--mono); font-size: 0.72rem; color: var(--muted);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.file-list { display: flex; flex-direction: column; gap: 0.2rem; }
.file-item {
  display: flex; align-items: center; gap: 0.45rem; width: 100%; text-align: left;
  border: 1px solid transparent; background: transparent; border-radius: 8px;
  padding: 0.45rem 0.55rem; cursor: pointer; color: inherit; font: inherit;
}
.file-item:hover { background: var(--surface); border-color: var(--border); }
.file-item .file-icon { color: var(--muted); font-size: 0.85rem; width: 1.1rem; text-align: center; }
.file-item.notebook .file-icon { color: var(--accent); }
.file-item .file-name {
  font-family: var(--mono); font-size: 0.82rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.welcome-main {
  display: flex; align-items: center; justify-content: center; padding: 2rem 1.5rem;
}
.welcome-hero { max-width: 28rem; text-align: left; }
.welcome-brand {
  font-weight: 700; letter-spacing: -0.04em; font-size: 2rem; color: var(--accent); margin-bottom: 0.75rem;
}
.welcome-title {
  margin: 0 0 0.55rem; font-size: 1.55rem; letter-spacing: -0.03em; font-weight: 650;
}
.welcome-sub { margin: 0 0 1.25rem; color: var(--muted); line-height: 1.5; }
.welcome-sub code { font-family: var(--mono); font-size: 0.9em; }
.welcome-actions { display: flex; gap: 0.55rem; flex-wrap: wrap; margin-bottom: 1.25rem; }
.welcome-hint { margin: 0; font-size: 0.78rem; color: var(--muted); font-family: var(--mono); word-break: break-all; }
.menu-btn {
  width: 2.15rem; height: 2.15rem; color: var(--accent);
  border: 1px solid transparent; background: transparent;
}
.menu-btn:hover { color: var(--accent); background: var(--accent-soft); border-color: color-mix(in srgb, var(--accent) 30%, var(--border)); }
.menu-btn[aria-expanded="false"] { color: var(--muted); }
.menu-icon {
  display: block; width: 1.05rem; height: 0.78rem; position: relative;
}
.menu-icon::before, .menu-icon::after {
  content: ""; position: absolute; left: 0; right: 0; height: 2px;
  background: currentColor; border-radius: 1px;
}
.menu-icon::before { top: 0; box-shadow: 0 0.28rem 0 currentColor; }
.menu-icon::after { bottom: 0; }
.actions { display: flex; gap: 0.45rem; align-items: center; flex-wrap: wrap; }
.auto-toggle {
  display: flex; align-items: center; gap: 0.4rem; font-size: 0.88rem; color: var(--muted);
  user-select: none; cursor: pointer; margin-right: 0.35rem;
  padding: 0.3rem 0.55rem; border-radius: 999px; border: 1px solid transparent;
}
.auto-toggle:hover { border-color: var(--border); background: var(--surface); }
.auto-toggle input { accent-color: var(--accent); }
button, select.kind-select {
  font: inherit; border: 1px solid var(--border-strong); background: var(--surface);
  padding: 0.4rem 0.75rem; border-radius: 8px; cursor: pointer; color: var(--ink);
  transition: border-color 0.12s, background 0.12s, transform 0.08s;
}
button:hover, select.kind-select:hover { border-color: var(--accent); }
button:active { transform: translateY(1px); }
button.primary { background: var(--accent); color: white; border-color: var(--accent); }
button.primary:hover { filter: brightness(1.06); }
button.icon {
  width: 2rem; height: 2rem; padding: 0; display: inline-flex; align-items: center; justify-content: center;
  border-radius: 8px; color: var(--muted); background: transparent; border-color: transparent;
  font-size: 0.95rem; line-height: 1;
}
button.icon:hover { color: var(--ink); background: #f5f5f4; border-color: var(--border); }
button.icon.danger:hover { color: var(--error); background: #fef2f2; border-color: #fecaca; }
button.icon:disabled { opacity: 0.35; pointer-events: none; }
button.icon.run-icon {
  color: white; background: var(--accent); border-color: var(--accent); width: 2.15rem; height: 2.15rem;
}
button.icon.run-icon:hover { filter: brightness(1.08); color: white; }
select.kind-select {
  font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.04em;
  padding: 0.25rem 0.45rem; border-radius: 6px; color: var(--muted); background: #fafaf9;
}
.conn { font-size: 0.8rem; color: var(--muted); margin-left: 0.35rem; }
.conn.ok { color: var(--ok); }
.conn.bad { color: var(--error); }
.diag {
  margin: 0.75rem 1.25rem 0; padding: 0.75rem 1rem; border-radius: 10px;
  background: #fef2f2; border: 1px solid #fecaca; color: var(--error); font-family: var(--mono); font-size: 0.85rem;
}
.diag.hidden { display: none; }
.workspace {
  --inspector-w: 280px;
  display: grid;
  grid-template-columns: var(--inspector-w) minmax(0, 1fr);
  align-items: stretch;
  gap: 0;
  width: 100%;
  margin: 0;
  min-height: calc(100vh - 3.5rem);
  transition: grid-template-columns 0.2s ease;
}
.workspace.inspector-collapsed {
  --inspector-w: 0px;
}
.workspace-main { min-width: 0; display: flex; flex-direction: column; min-height: calc(100vh - 3.5rem); }
.workspace-main.split-mode {
  height: 100%;
  max-height: 100%;
  min-height: 0;
  overflow: hidden;
}
.main-chrome {
  display: flex; align-items: center; justify-content: space-between; gap: 0.75rem;
  padding: 0.55rem 1.25rem 0.35rem; position: sticky; top: 3.55rem; z-index: 8;
  background: color-mix(in srgb, var(--bg) 82%, transparent); backdrop-filter: blur(10px);
  flex-shrink: 0;
}
.workspace-main.split-mode .main-chrome { position: static; }
.main-tabs {
  display: flex; gap: 0.15rem; padding: 0.18rem;
  border-radius: 10px; background: color-mix(in srgb, #f5f5f4 70%, transparent);
  border: 1px solid var(--border);
}
.main-tab {
  border: 1px solid transparent; background: transparent; border-radius: 8px;
  padding: 0.38rem 0.9rem; cursor: pointer; font-size: 0.82rem; font-weight: 650; color: var(--muted);
}
.main-tab:hover { color: var(--ink); background: color-mix(in srgb, var(--surface) 80%, transparent); }
.main-tab.active {
  color: var(--accent); background: var(--surface);
  border-color: color-mix(in srgb, var(--accent) 22%, var(--border));
  box-shadow: 0 1px 2px rgba(28,25,23,0.04);
}
.workspace-main.split-mode .main-tab { pointer-events: none; opacity: 0.55; }
.workspace-main.split-mode .main-tab.active { opacity: 1; }
.detach-btn {
  font-size: 0.78rem; font-weight: 600; color: var(--muted);
  border-radius: 999px; padding: 0.32rem 0.75rem;
}
.detach-btn:hover { color: var(--accent); }
.main-panes {
  flex: 1; min-height: 0; display: grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: start;
}
.workspace-main.split-mode .main-panes {
  grid-template-columns: minmax(0, var(--split-left, 1fr)) 6px minmax(0, var(--split-right, 1fr));
  align-items: stretch;
  overflow: hidden;
  height: auto;
}
.pane { min-width: 0; }
.workspace-main:not(.split-mode) .pane[hidden] { display: none !important; }
.workspace-main.split-mode .pane[hidden] { display: block !important; }
.split-handle {
  display: none; cursor: col-resize; align-self: stretch; min-height: 12rem;
  background: transparent; position: relative; z-index: 2;
}
.workspace-main.split-mode .split-handle { display: block; min-height: 0; }
.split-handle::before {
  content: ""; position: absolute; top: 1rem; bottom: 1rem; left: 2px; width: 2px;
  border-radius: 2px; background: var(--border-strong);
}
.split-handle:hover::before, .split-handle.dragging::before { background: var(--accent); }
.notebook, .shared {
  max-width: 960px; width: 100%; margin: 0 auto; padding: 0.75rem 1.5rem 4rem;
  display: flex; flex-direction: column; gap: 0;
}
.workspace-main.split-mode .notebook,
.workspace-main.split-mode .shared {
  display: block !important;
  max-width: none; margin: 0; padding: 0.75rem 1rem 2rem;
  min-width: 0; min-height: 0; height: 100%;
  overflow-x: hidden; overflow-y: scroll;
  overscroll-behavior: contain;
  -webkit-overflow-scrolling: touch;
}
body.split-layout .inspector {
  max-height: none;
  height: 100%;
}
.shared-toolbar {
  display: flex; align-items: center; justify-content: space-between; gap: 0.75rem;
  margin: 0 0 0.85rem; padding: 0.35rem 0.15rem 0.55rem;
  border-bottom: 1px solid var(--border); flex-wrap: wrap;
}
.shared-toolbar-title {
  font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.08em;
  color: var(--muted); font-weight: 700;
}
.shared-readonly-toggle {
  display: flex; align-items: center; gap: 0.4rem; font-size: 0.88rem; color: var(--muted);
  user-select: none; cursor: pointer;
  padding: 0.3rem 0.55rem; border-radius: 999px; border: 1px solid transparent;
}
.shared-readonly-toggle:hover { border-color: var(--border); background: var(--surface); }
.shared-readonly-toggle input { accent-color: var(--accent); }
.shared-section-label {
  font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.08em;
  color: var(--muted); font-weight: 700; margin: 0.85rem 0 0.35rem 0.15rem;
}
.shared-section-label:first-child { margin-top: 0.25rem; }
.card.definition { box-shadow: inset 3px 0 0 var(--accent-2), var(--shadow); }
.card.definition .badge.readonly {
  color: var(--accent-2); border-color: #fed7aa; background: #fff7ed;
}
.def-source {
  margin: 0; padding: 0.85rem 1rem; font-family: var(--mono); font-size: 0.8rem;
  white-space: pre-wrap; word-break: break-word; color: #44403c; background: #fafaf9;
  border-top: 1px solid var(--border); max-height: 22rem; overflow: auto;
}
.insp-kind.impl, .insp-kind.const, .insp-kind.static, .insp-kind.mod {
  color: var(--accent-2); border-color: #fed7aa; background: #fff7ed;
}
.insp-kind.use, .insp-kind.item { color: var(--muted); }
.inspector {
  position: sticky; top: 3.6rem; align-self: start;
  width: 100%; min-width: 0; max-height: calc(100vh - 3.6rem); overflow: auto;
  padding: 1rem 0.85rem 2rem 1rem;
  border-right: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
  background: color-mix(in srgb, var(--surface) 78%, transparent);
  backdrop-filter: blur(8px);
  transition: opacity 0.18s ease;
}
.workspace.inspector-collapsed .inspector {
  opacity: 0; pointer-events: none; overflow: hidden;
  padding: 0; border-right-width: 0; visibility: hidden;
}
.inspector-title {
  font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.08em;
  color: var(--muted); font-weight: 700; margin: 0 0 0.85rem 0.15rem;
}
.side-tabs {
  display: flex; gap: 0.15rem; margin: 0 0 0.95rem;
  padding: 0.2rem; border-radius: 10px; background: color-mix(in srgb, #f5f5f4 70%, transparent);
  border: 1px solid var(--border);
}
.side-tab {
  flex: 1; border: 1px solid transparent; background: transparent;
  border-radius: 8px; padding: 0.4rem 0.5rem; cursor: pointer;
  font-size: 0.78rem; font-weight: 650; color: var(--muted); letter-spacing: 0.01em;
}
.side-tab:hover { color: var(--ink); background: color-mix(in srgb, var(--surface) 80%, transparent); }
.side-tab.active {
  color: var(--accent); background: var(--surface);
  border-color: color-mix(in srgb, var(--accent) 22%, var(--border));
  box-shadow: 0 1px 2px rgba(28,25,23,0.04);
}
.side-panel { min-width: 0; }
.plan-list { display: flex; flex-direction: column; gap: 0.25rem; }
.plan-item {
  display: block; width: 100%; text-align: left;
  border: 1px solid transparent; background: transparent;
  border-radius: 9px; padding: 0.5rem 0.55rem; cursor: pointer; color: inherit;
  transition: background 0.12s, border-color 0.12s;
}
.plan-item:hover { background: var(--surface); border-color: var(--border); }
.plan-item.active { background: var(--accent-soft); border-color: color-mix(in srgb, var(--accent) 35%, var(--border)); }
.plan-item:focus-visible { outline: 2px solid color-mix(in srgb, var(--accent) 50%, transparent); outline-offset: 1px; }
.plan-title {
  font-weight: 650; font-size: 0.88rem; color: var(--ink); line-height: 1.3;
  overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;
}
.plan-title.h2 { font-size: 0.84rem; font-weight: 600; padding-left: 0.65rem; }
.plan-title.h3 { font-size: 0.8rem; font-weight: 550; padding-left: 1.15rem; color: #44403c; }
.plan-meta {
  margin-top: 0.2rem; font-size: 0.7rem; color: var(--muted); font-family: var(--mono);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.plan-preview {
  margin-top: 0.25rem; font-size: 0.74rem; color: var(--muted); line-height: 1.35;
  overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;
}
.insp-section { margin-bottom: 1.15rem; }
.insp-section + .insp-section { padding-top: 0.85rem; border-top: 1px solid var(--border); }
.insp-h {
  display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;
  width: 100%; margin: 0 0 0.45rem; padding: 0.28rem 0.35rem 0.28rem 0.2rem;
  font: inherit; font-size: 0.72rem; text-transform: uppercase;
  letter-spacing: 0.07em; color: var(--muted); font-weight: 650;
  background: transparent; border: 1px solid transparent; border-radius: 8px;
  cursor: pointer; text-align: left;
}
.insp-h:hover { background: color-mix(in srgb, var(--surface) 80%, transparent); border-color: var(--border); color: var(--ink); }
.insp-h:focus-visible { outline: 2px solid color-mix(in srgb, var(--accent) 50%, transparent); outline-offset: 1px; }
.insp-h-left { display: flex; align-items: center; gap: 0.35rem; min-width: 0; }
.insp-chevron {
  display: inline-block; width: 0.55rem; height: 0.55rem; flex-shrink: 0;
  border-right: 1.75px solid currentColor; border-bottom: 1.75px solid currentColor;
  transform: rotate(45deg); transition: transform 0.15s ease; margin: 0 0.1rem 0.12rem 0.05rem;
  opacity: 0.75;
}
.insp-section.folded .insp-chevron { transform: rotate(-45deg); margin-bottom: 0; margin-left: 0.12rem; }
.insp-section.folded .insp-h { margin-bottom: 0; }
.insp-section.folded .insp-list { display: none; }
.insp-count {
  font-variant-numeric: tabular-nums; font-size: 0.68rem; color: var(--muted);
  background: #f5f5f4; border: 1px solid var(--border); border-radius: 999px;
  padding: 0.05rem 0.45rem; font-weight: 600; letter-spacing: 0;
  text-transform: none;
}
.insp-empty { font-size: 0.8rem; color: var(--muted); padding: 0.35rem 0.45rem; font-style: italic; }
.insp-list { display: flex; flex-direction: column; gap: 0.3rem; }
.insp-item {
  display: block; width: 100%; text-align: left;
  border: 1px solid transparent; background: transparent;
  border-radius: 9px; padding: 0.45rem 0.55rem; cursor: pointer;
  color: inherit; transition: background 0.12s, border-color 0.12s;
}
.insp-item:hover { background: var(--surface); border-color: var(--border); }
.insp-item:focus-visible { outline: 2px solid color-mix(in srgb, var(--accent) 50%, transparent); outline-offset: 1px; }
.insp-item.active { background: var(--accent-soft); border-color: color-mix(in srgb, var(--accent) 35%, var(--border)); }
.insp-row { display: flex; align-items: baseline; justify-content: space-between; gap: 0.5rem; min-width: 0; }
.insp-name { font-family: var(--mono); font-size: 0.82rem; font-weight: 600; color: var(--ink); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.insp-kind {
  flex-shrink: 0; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.04em;
  color: var(--muted); border: 1px solid var(--border); border-radius: 999px;
  padding: 0.08rem 0.4rem; background: #fafaf9;
}
.insp-kind.struct, .insp-kind.enum, .insp-kind.type, .insp-kind.trait { color: var(--accent-2); border-color: #fed7aa; background: #fff7ed; }
.insp-kind.helper { color: var(--helper); border-color: #bfdbfe; background: #eff6ff; }
.insp-kind.cell { color: var(--accent); border-color: #99f6e4; background: var(--accent-soft); }
.insp-meta { margin-top: 0.15rem; font-size: 0.72rem; color: var(--muted); font-family: var(--mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.insp-value {
  margin-top: 0.28rem; font-family: var(--mono); font-size: 0.72rem; color: #44403c;
  background: #fafaf9; border: 1px solid var(--border); border-radius: 6px;
  padding: 0.28rem 0.4rem; max-height: 4.2rem; overflow: auto; white-space: pre-wrap; word-break: break-word;
}
.insp-value.empty { color: var(--muted); font-style: italic; background: transparent; border-style: dashed; }
.insp-value.error { color: var(--error); background: #fef2f2; border-color: #fecaca; }
.insp-sig { margin-top: 0.18rem; font-family: var(--mono); font-size: 0.7rem; color: var(--muted); line-height: 1.35; white-space: pre-wrap; word-break: break-word; }
.card.flash {
  outline: 2px solid color-mix(in srgb, var(--accent) 55%, transparent);
  outline-offset: 2px;
}
@media (max-width: 900px) {
  .workspace { grid-template-columns: 1fr; }
  .workspace.inspector-collapsed { grid-template-columns: 1fr; }
  .inspector {
    position: static; width: auto; max-height: none; border-right: none;
    border-bottom: 1px solid var(--border); padding: 0.85rem 1rem 1rem;
  }
  .workspace.inspector-collapsed .inspector { display: none; transform: none; }
  .insp-list { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 0.35rem; }
}
.card {
  background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius);
  overflow: hidden; box-shadow: var(--shadow); transition: border-color 0.15s, box-shadow 0.15s;
}
.card:hover { border-color: color-mix(in srgb, var(--accent) 25%, var(--border)); }
.card.dirty { box-shadow: inset 3px 0 0 var(--dirty), var(--shadow); }
.card.error { box-shadow: inset 3px 0 0 var(--error), var(--shadow); }
.card.success { box-shadow: inset 3px 0 0 var(--ok), var(--shadow); }
.card.helper { box-shadow: inset 3px 0 0 var(--helper), var(--shadow); }
.card.md { box-shadow: inset 3px 0 0 var(--md), var(--shadow); }
.card.running { box-shadow: inset 3px 0 0 var(--accent), var(--shadow); }
.card-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.5rem 0.75rem 0.5rem 0.85rem; border-bottom: 1px solid var(--border); gap: 0.75rem;
  background: linear-gradient(180deg, #fffcf7 0%, var(--surface) 100%);
}
.card-head-left, .card-head-right { display: flex; align-items: center; gap: 0.4rem; min-width: 0; }
.card-head-left { flex: 1; }
.card-title { font-weight: 600; font-family: var(--mono); font-size: 0.88rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.meta { color: var(--muted); font-size: 0.78rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.toolbar-sep { width: 1px; height: 1.25rem; background: var(--border); margin: 0 0.15rem; }
.badge {
  font-size: 0.68rem; text-transform: uppercase; letter-spacing: 0.05em;
  color: var(--muted); border: 1px solid var(--border); padding: 0.18rem 0.5rem; border-radius: 999px;
  white-space: nowrap; background: #fafaf9;
}
.badge.success { color: var(--ok); border-color: color-mix(in srgb, var(--ok) 35%, var(--border)); background: #f0fdf4; }
.badge.dirty { color: var(--dirty); border-color: color-mix(in srgb, var(--dirty) 40%, var(--border)); background: #fffbeb; }
.badge.error { color: var(--error); border-color: color-mix(in srgb, var(--error) 35%, var(--border)); background: #fef2f2; }
.badge.pristine { color: var(--muted); }
.badge.running { color: var(--accent); border-color: color-mix(in srgb, var(--accent) 40%, var(--border)); background: var(--accent-soft); }
@keyframes labrs-spin { to { transform: rotate(360deg); } }
.badge.running .spin {
  display: inline-block; animation: labrs-spin 0.75s linear infinite; margin-right: 0.25rem;
}
.docs { padding: 0.55rem 0.95rem; color: var(--muted); font-size: 0.88rem; white-space: pre-wrap; border-bottom: 1px solid var(--border); }
.editor { height: 150px; border-bottom: 1px solid var(--border); }
.editor.tall { height: 210px; }
.editor.md-edit { height: 140px; }
.panels { display: grid; grid-template-columns: 1fr 1fr; gap: 0; background: #fafaf9; }
@media (max-width: 720px) { .panels { grid-template-columns: 1fr; } }
.panel { padding: 0.7rem 0.9rem; min-height: 3.5rem; }
.panel + .panel { border-left: 1px solid var(--border); }
@media (max-width: 720px) { .panel + .panel { border-left: none; border-top: 1px solid var(--border); } }
.panel h4 { margin: 0 0 0.35rem; font-size: 0.68rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); font-weight: 600; }
.panel pre {
  margin: 0; font-family: var(--mono); font-size: 0.8rem; white-space: pre-wrap; word-break: break-word; color: #44403c;
}
.md-body { padding: 1.1rem 1.2rem; line-height: 1.6; }
.md-body h1, .md-body h2, .md-body h3 { margin-top: 0; letter-spacing: -0.02em; }
.md-body code { font-family: var(--mono); background: #f5f5f4; padding: 0.1em 0.35em; border-radius: 4px; font-size: 0.9em; }
.md-edit-wrap { display: none; }
.md-edit-wrap.open { display: block; }
.md-body.hidden { display: none; }
.insert-gap {
  position: relative; height: 1.15rem; margin: 0.2rem 0;
  display: flex; align-items: center; justify-content: center;
}
.insert-gap .plus {
  opacity: 0; width: 1.55rem; height: 1.55rem; border-radius: 999px;
  border: 1px solid var(--border-strong); background: var(--surface);
  color: var(--accent); font-size: 1.05rem; line-height: 1; padding: 0;
  display: flex; align-items: center; justify-content: center;
  transition: opacity 0.12s ease, transform 0.12s ease, box-shadow 0.12s;
  z-index: 2; box-shadow: var(--shadow);
}
.insert-gap:hover .plus, .insert-gap.open .plus { opacity: 1; transform: scale(1.05); }
.insert-gap::before {
  content: ""; position: absolute; left: 6%; right: 6%; height: 1px;
  background: transparent; transition: background 0.12s;
}
.insert-gap:hover::before, .insert-gap.open::before {
  background: linear-gradient(90deg, transparent, var(--accent), transparent);
  opacity: 0.45;
}
.insert-menu {
  display: none; position: absolute; top: 100%; margin-top: 0.3rem;
  background: var(--surface); border: 1px solid var(--border); border-radius: 10px;
  box-shadow: var(--shadow); padding: 0.3rem; z-index: 5; gap: 0.15rem;
}
.insert-gap.open .insert-menu { display: flex; }
.insert-menu button {
  border: none; background: transparent; padding: 0.4rem 0.8rem; border-radius: 7px; font-size: 0.85rem;
}
.insert-menu button:hover { background: var(--accent-soft); color: var(--accent); border-color: transparent; }
.footer-add {
  margin-top: 1.75rem; padding: 1.1rem 1.25rem;
  border: 1px dashed color-mix(in srgb, var(--accent) 35%, var(--border));
  border-radius: 14px; display: flex; flex-direction: column; align-items: center; gap: 0.75rem;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--accent-soft) 45%, transparent), transparent),
    color-mix(in srgb, var(--surface) 85%, transparent);
}
.footer-add .footer-label {
  font-size: 0.78rem; text-transform: uppercase; letter-spacing: 0.08em; color: var(--muted); font-weight: 600;
}
.footer-add .footer-btns { display: flex; gap: 0.55rem; flex-wrap: wrap; justify-content: center; }
.footer-add button {
  min-width: 8.5rem; border-radius: 999px; padding: 0.55rem 1rem;
  border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
  background: var(--surface); color: var(--accent); font-weight: 600;
}
.footer-add button:hover {
  background: var(--accent); color: white; border-color: var(--accent);
}
.footer-add button .plus-mark { margin-right: 0.35rem; font-weight: 700; }
"#;

const APP_JS: &str = r#"
(() => {
  const notebookEl = document.getElementById("notebook");
  const sharedEl = document.getElementById("shared");
  const mainPanesEl = document.getElementById("main-panes");
  const workspaceMainEl = document.querySelector(".workspace-main");
  const inspectorEl = document.getElementById("inspector");
  const diagEl = document.getElementById("diag");
  const connEl = document.getElementById("conn");
  const editors = new Map();
  let state = null;
  let welcomeState = null;
  let ws;
  const mdEditing = new Set();
  let openInsertKey = null;
  let pendingFocus = null; // { key, selection }
  const runningCells = new Set();
  let activeInspectKey = null;
  const DEF_PRIMARY = ["struct", "enum", "type", "trait", "const", "static", "mod", "impl"];
  const DEF_OTHER = ["use", "item"];
  let mainMode = (() => {
    try {
      const m = localStorage.getItem("labrs.mainMode");
      return m === "split" || m === "tabs" ? m : "tabs";
    } catch (_) { return "tabs"; }
  })();
  let mainTab = (() => {
    try {
      const t = localStorage.getItem("labrs.mainTab");
      return t === "shared" || t === "notebook" ? t : "notebook";
    } catch (_) { return "notebook"; }
  })();
  let splitLeftFrac = (() => {
    try {
      const v = parseFloat(localStorage.getItem("labrs.splitLeft"));
      return Number.isFinite(v) && v > 0.25 && v < 0.75 ? v : 0.55;
    } catch (_) { return 0.55; }
  })();
  let sideTab = (() => {
    try {
      const t = localStorage.getItem("labrs.sideTab");
      return t === "plan" || t === "inspector" ? t : "inspector";
    } catch (_) { return "inspector"; }
  })();
  let activePlanKey = null;
  let sharedReadOnly = (() => {
    try {
      const v = localStorage.getItem("labrs.sharedReadOnly");
      if (v === "0") return false;
      if (v === "1") return true;
    } catch (_) {}
    return true;
  })();
  function persistSharedReadOnly() {
    try { localStorage.setItem("labrs.sharedReadOnly", sharedReadOnly ? "1" : "0"); } catch (_) {}
  }
  const foldedSections = (() => {
    const defaults = { variables: false, structures: false, helpers: false, other_defs: true };
    try {
      const raw = localStorage.getItem("labrs.inspectorFolded");
      if (!raw) return defaults;
      return { ...defaults, ...JSON.parse(raw) };
    } catch (_) {
      return defaults;
    }
  })();
  function persistFoldedSections() {
    try { localStorage.setItem("labrs.inspectorFolded", JSON.stringify(foldedSections)); } catch (_) {}
  }
  function persistMainLayout() {
    try {
      localStorage.setItem("labrs.mainMode", mainMode);
      localStorage.setItem("labrs.mainTab", mainTab);
      localStorage.setItem("labrs.splitLeft", String(splitLeftFrac));
    } catch (_) {}
  }
  function partitionDefs(all) {
    const primary = [];
    const other = [];
    for (const d of all || []) {
      if (DEF_PRIMARY.includes(d.kind)) primary.push(d);
      else other.push(d);
    }
    return { primary, other };
  }
  function applyMainLayout() {
    if (!workspaceMainEl) return;
    const split = mainMode === "split";
    workspaceMainEl.classList.toggle("split-mode", split);
    document.body.classList.toggle("split-layout", split);
    if (mainPanesEl) {
      if (split) {
        mainPanesEl.style.setProperty("--split-left", splitLeftFrac + "fr");
        mainPanesEl.style.setProperty("--split-right", (1 - splitLeftFrac) + "fr");
      } else {
        mainPanesEl.style.removeProperty("--split-left");
        mainPanesEl.style.removeProperty("--split-right");
      }
    }
    if (notebookEl) notebookEl.hidden = !split && mainTab !== "notebook";
    if (sharedEl) sharedEl.hidden = !split && mainTab !== "shared";
    document.querySelectorAll(".main-tab").forEach(tab => {
      const id = tab.getAttribute("data-main-tab");
      tab.classList.toggle("active", split ? true : mainTab === id);
      if (!split) tab.setAttribute("aria-selected", mainTab === id ? "true" : "false");
      else tab.setAttribute("aria-selected", "true");
    });
    const detachBtn = document.getElementById("btn-detach-shared");
    if (detachBtn) {
      detachBtn.textContent = split ? "Reattach" : "Detach";
      detachBtn.title = split ? "Show Shared as a tab" : "Show Shared beside Notebook";
    }
    persistMainLayout();
  }
  function ensurePaneForKind(kind) {
    if (kind === "helper" || kind === "definition") {
      if (mainMode === "tabs" && mainTab !== "shared") {
        mainTab = "shared";
        applyMainLayout();
      }
    } else if (kind === "cell" || kind === "markdown") {
      if (mainMode === "tabs" && mainTab !== "notebook") {
        mainTab = "notebook";
        applyMainLayout();
      }
    }
  }

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
    const notebookScroll = notebookEl ? notebookEl.scrollTop : 0;
    const sharedScroll = sharedEl ? sharedEl.scrollTop : 0;
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
    return { scrollY, notebookScroll, sharedScroll, focusKey, selection, pending };
  }

  function restoreViewState(vs) {
    requestAnimationFrame(() => {
      if (mainMode === "split") {
        if (notebookEl) notebookEl.scrollTop = vs.notebookScroll || 0;
        if (sharedEl) sharedEl.scrollTop = vs.sharedScroll || 0;
        window.scrollTo(0, 0);
      } else {
        window.scrollTo(0, vs.scrollY || 0);
      }
      const key = vs.focusKey;
      if (key && editors.has(key)) {
        const ed = editors.get(key);
        ed.focus();
        const sel = (vs.pending.get(key) && vs.pending.get(key).selection) || vs.selection;
        if (sel) ed.setSelection(sel);
        // Monaco layout/focus can jump parent scroll; re-apply pane scroll after focus
        requestAnimationFrame(() => {
          if (mainMode === "split") {
            if (notebookEl) notebookEl.scrollTop = vs.notebookScroll || 0;
            if (sharedEl) sharedEl.scrollTop = vs.sharedScroll || 0;
          }
        });
      }
      pendingFocus = null;
    });
  }

  function handleServer(msg) {
    switch (msg.type) {
      case "welcome":
        welcomeState = msg;
        state = null;
        runningCells.clear();
        document.body.classList.remove("split-layout");
        setAppMode("welcome");
        renderWelcome();
        break;
      case "dir_listing":
        if (welcomeState) {
          welcomeState.cwd = msg.path;
          welcomeState.entries = msg.entries;
          renderWelcome();
        }
        break;
      case "notebook_state":
        state = msg;
        welcomeState = null;
        runningCells.clear();
        const autoEl = document.getElementById("chk-auto");
        if (autoEl && msg.snapshot && typeof msg.snapshot.auto_react === "boolean") {
          autoEl.checked = msg.snapshot.auto_react;
        }
        setAppMode("notebook");
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
      case "definition_formatted": {
        const ed = editors.get("def:" + msg.name);
        if (ed && ed.getValue() !== msg.source) {
          const sel = ed.getSelection();
          ed.setValue(msg.source);
          if (sel) ed.setSelection(sel);
        }
        break;
      }
      case "preamble_formatted": {
        const ed = editors.get("def:__preamble__");
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
    const inspVal = document.querySelector(`[data-insp-value="${CSS.escape(out.cell)}"]`);
    if (inspVal) {
      if (!out.success) {
        inspVal.className = "insp-value error";
        inspVal.textContent = out.error || "error";
      } else if (out.value === undefined || out.value === null) {
        inspVal.className = "insp-value empty";
        inspVal.textContent = "—";
      } else {
        inspVal.className = "insp-value";
        const text = typeof out.value === "string" ? JSON.stringify(out.value) : JSON.stringify(out.value, null, 2);
        inspVal.textContent = text;
      }
    }
  }

  function paramNames(params) {
    return (params || []).map(p => typeof p === "string" ? p : p.name);
  }

  function firstSigLine(source) {
    const line = String(source || "").split("\n").find(l => l.trim()) || "";
    return line.trim().replace(/\s*\{\s*$/, "");
  }

  function focusItem(kind, name) {
    ensurePaneForKind(kind);
    activeInspectKey = kind + ":" + name;
    if (inspectorEl) {
      inspectorEl.querySelectorAll(".insp-item.active").forEach(n => n.classList.remove("active"));
      const btn = inspectorEl.querySelector(`[data-insp-key="${CSS.escape(activeInspectKey)}"]`);
      if (btn) btn.classList.add("active");
    }
    requestAnimationFrame(() => {
      const card = document.querySelector(`[data-item="${CSS.escape(kind + ":" + name)}"]`);
      if (card) {
        card.scrollIntoView({ behavior: "smooth", block: "center" });
        card.classList.add("flash");
        setTimeout(() => card.classList.remove("flash"), 900);
      }
    });
  }

  function setAppMode(mode) {
    document.body.classList.toggle("welcome-mode", mode === "welcome");
    const welcomeEl = document.getElementById("welcome");
    if (welcomeEl) welcomeEl.classList.toggle("hidden", mode !== "welcome");
    if (mode === "welcome") {
      editors.forEach(ed => ed.dispose());
      editors.clear();
      if (notebookEl) notebookEl.innerHTML = "";
      if (sharedEl) sharedEl.innerHTML = "";
      if (inspectorEl) inspectorEl.innerHTML = "";
    }
  }

  function parentDir(cwd) {
    if (!cwd) return null;
    const parts = String(cwd).split("/").filter(Boolean);
    if (!parts.length) return null;
    parts.pop();
    return parts.join("/");
  }

  function renderWelcome() {
    if (!welcomeState) return;
    const rootEl = document.getElementById("welcome-root");
    const cwdEl = document.getElementById("file-cwd");
    const listEl = document.getElementById("file-list");
    if (rootEl) rootEl.textContent = welcomeState.root || "";
    if (cwdEl) cwdEl.textContent = welcomeState.cwd ? "./" + welcomeState.cwd : ".";
    if (!listEl) return;
    listEl.innerHTML = "";

    const parent = parentDir(welcomeState.cwd);
    if (welcomeState.cwd) {
      const up = document.createElement("button");
      up.type = "button";
      up.className = "file-item";
      up.innerHTML = '<span class="file-icon" aria-hidden="true">↑</span><span class="file-name">..</span>';
      up.onclick = () => send({ type: "list_dir", path: parent || "" });
      listEl.appendChild(up);
    }

    const entries = welcomeState.entries || [];
    if (!entries.length) {
      const empty = el("div", "insp-empty");
      empty.textContent = "No folders or .rs notebooks here";
      listEl.appendChild(empty);
    }
    for (const e of entries) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "file-item" + (e.is_notebook ? " notebook" : "");
      const icon = e.is_dir ? "▸" : "·";
      btn.innerHTML = `<span class="file-icon" aria-hidden="true">${icon}</span><span class="file-name"></span>`;
      btn.querySelector(".file-name").textContent = e.name;
      if (e.is_dir) {
        btn.onclick = () => send({ type: "list_dir", path: e.path });
      } else {
        btn.onclick = () => send({ type: "open_notebook", path: e.path });
      }
      listEl.appendChild(btn);
    }

    const autoEl = document.getElementById("chk-auto");
    if (autoEl && typeof welcomeState.auto_react === "boolean") {
      autoEl.checked = welcomeState.auto_react;
    }
  }

  function makeInspSection(id, label, count, buildList) {
    const sec = el("section", "insp-section" + (foldedSections[id] ? " folded" : ""));
    sec.setAttribute("data-insp-section", id);
    const head = document.createElement("button");
    head.type = "button";
    head.className = "insp-h";
    head.setAttribute("aria-expanded", foldedSections[id] ? "false" : "true");
    head.setAttribute("aria-controls", "insp-list-" + id);
    const left = el("span", "insp-h-left");
    const chev = el("span", "insp-chevron");
    chev.setAttribute("aria-hidden", "true");
    const title = document.createElement("span");
    title.textContent = label;
    left.appendChild(chev);
    left.appendChild(title);
    const countEl = el("span", "insp-count");
    countEl.textContent = String(count);
    head.appendChild(left);
    head.appendChild(countEl);
    head.onclick = () => {
      foldedSections[id] = !foldedSections[id];
      persistFoldedSections();
      sec.classList.toggle("folded", foldedSections[id]);
      head.setAttribute("aria-expanded", foldedSections[id] ? "false" : "true");
    };
    const list = el("div", "insp-list");
    list.id = "insp-list-" + id;
    buildList(list);
    sec.appendChild(head);
    sec.appendChild(list);
    return sec;
  }

  function mdOutlineTitle(content, fallback) {
    const lines = String(content || "").split("\n");
    for (const line of lines) {
      const m = /^(#{1,3})\s+(.+?)\s*$/.exec(line.trim());
      if (m) return { level: m[1].length, title: m[2].replace(/[`*_]/g, "").trim() };
    }
    const first = lines.map(l => l.trim()).find(Boolean);
    return { level: 1, title: first ? first.slice(0, 80) : (fallback || "Untitled") };
  }

  function mdHeadings(content) {
    const out = [];
    for (const line of String(content || "").split("\n")) {
      const m = /^(#{1,3})\s+(.+?)\s*$/.exec(line.trim());
      if (m) out.push({ level: m[1].length, title: m[2].replace(/[`*_]/g, "").trim() });
    }
    return out;
  }

  function mdPreview(content) {
    const lines = String(content || "").split("\n")
      .map(l => l.trim())
      .filter(l => l && !/^#{1,6}\s/.test(l));
    return lines[0] || "";
  }

  function orderedMarkdown() {
    const byMd = Object.fromEntries((state.markdown_detail || []).map(m => [m.name, m]));
    const snap = state.snapshot || {};
    const order = snap.order || [];
    const fromOrder = order
      .filter(e => e.kind === "markdown")
      .map(e => byMd[e.id])
      .filter(Boolean);
    if (fromOrder.length) return fromOrder;
    return state.markdown_detail || [];
  }

  function renderInspectorPanel(panel) {
    const snap = state.snapshot || {};
    const cellState = Object.fromEntries((snap.cells || []).map(c => [c.name, c]));
    const cells = state.cells_detail || [];
    const helpers = state.helpers_detail || [];
    const { primary: defs, other: otherDefs } = partitionDefs(state.definitions_detail);

    panel.appendChild(makeInspSection("variables", "Variables", cells.length, (varList) => {
      if (!cells.length) {
        const empty = el("div", "insp-empty");
        empty.textContent = "No cells yet";
        varList.appendChild(empty);
        return;
      }
      for (const c of cells) {
        const st = cellState[c.name];
        const out = st && st.output;
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "insp-item" + (activeInspectKey === "cell:" + c.name ? " active" : "");
        btn.setAttribute("data-insp-key", "cell:" + c.name);
        btn.onclick = () => focusItem("cell", c.name);
        const row = el("div", "insp-row");
        const name = el("span", "insp-name");
        name.textContent = c.name;
        const kind = el("span", "insp-kind cell");
        kind.textContent = "cell";
        row.appendChild(name);
        row.appendChild(kind);
        btn.appendChild(row);
        const meta = el("div", "insp-meta");
        meta.textContent = "→ " + (c.return_type || "?");
        btn.appendChild(meta);
        const val = el("div", "");
        val.setAttribute("data-insp-value", c.name);
        if (out && !out.success) {
          val.className = "insp-value error";
          val.textContent = out.error || "error";
        } else if (out && out.value !== undefined && out.value !== null) {
          val.className = "insp-value";
          val.textContent = typeof out.value === "string"
            ? JSON.stringify(out.value)
            : JSON.stringify(out.value, null, 2);
        } else {
          val.className = "insp-value empty";
          val.textContent = st && st.dirty ? "dirty" : "—";
        }
        btn.appendChild(val);
        varList.appendChild(btn);
      }
    }));

    panel.appendChild(makeInspSection("structures", "Structures", defs.length, (defList) => {
      if (!defs.length) {
        const empty = el("div", "insp-empty");
        empty.textContent = "No shared types";
        defList.appendChild(empty);
        return;
      }
      for (const d of defs) {
        const btn = document.createElement("button");
        btn.type = "button";
        const key = "definition:" + d.name;
        btn.className = "insp-item" + (activeInspectKey === key ? " active" : "");
        btn.setAttribute("data-insp-key", key);
        btn.onclick = () => focusItem("definition", d.name);
        const row = el("div", "insp-row");
        const name = el("span", "insp-name");
        name.textContent = d.name;
        const kind = el("span", "insp-kind " + d.kind);
        kind.textContent = d.kind;
        row.appendChild(name);
        row.appendChild(kind);
        btn.appendChild(row);
        const sig = el("div", "insp-sig");
        sig.textContent = firstSigLine(d.source) || d.kind;
        btn.appendChild(sig);
        defList.appendChild(btn);
      }
    }));

    if (otherDefs.length) {
      panel.appendChild(makeInspSection("other_defs", "Preamble", 1, (list) => {
        const btn = document.createElement("button");
        btn.type = "button";
        const key = "definition:__preamble__";
        btn.className = "insp-item" + (activeInspectKey === key ? " active" : "");
        btn.setAttribute("data-insp-key", key);
        btn.onclick = () => focusItem("definition", "__preamble__");
        const row = el("div", "insp-row");
        const name = el("span", "insp-name");
        name.textContent = "preamble";
        const kind = el("span", "insp-kind use");
        kind.textContent = "use";
        row.appendChild(name);
        row.appendChild(kind);
        btn.appendChild(row);
        const sig = el("div", "insp-sig");
        const nUse = otherDefs.filter(d => d.kind === "use").length;
        sig.textContent = nUse
          ? nUse + " import" + (nUse === 1 ? "" : "s")
          : otherDefs.length + " item" + (otherDefs.length === 1 ? "" : "s");
        btn.appendChild(sig);
        list.appendChild(btn);
      }));
    }

    panel.appendChild(makeInspSection("helpers", "Helpers", helpers.length, (helpList) => {
      if (!helpers.length) {
        const empty = el("div", "insp-empty");
        empty.textContent = "No helpers";
        helpList.appendChild(empty);
        return;
      }
      for (const h of helpers) {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "insp-item" + (activeInspectKey === "helper:" + h.name ? " active" : "");
        btn.setAttribute("data-insp-key", "helper:" + h.name);
        btn.onclick = () => focusItem("helper", h.name);
        const row = el("div", "insp-row");
        const name = el("span", "insp-name");
        name.textContent = h.name;
        const kind = el("span", "insp-kind helper");
        kind.textContent = "fn";
        row.appendChild(name);
        row.appendChild(kind);
        btn.appendChild(row);
        const sig = el("div", "insp-sig");
        sig.textContent = firstSigLine(h.source);
        btn.appendChild(sig);
        helpList.appendChild(btn);
      }
    }));
  }

  function renderPlanPanel(panel) {
    const sections = orderedMarkdown();
    const list = el("div", "plan-list");
    if (!sections.length) {
      const empty = el("div", "insp-empty");
      empty.textContent = "No markdown sections";
      list.appendChild(empty);
      panel.appendChild(list);
      return;
    }
    for (const md of sections) {
      const info = mdOutlineTitle(md.content, md.name);
      const headings = mdHeadings(md.content);
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "plan-item" + (activePlanKey === md.name ? " active" : "");
      btn.setAttribute("data-plan-key", md.name);
      btn.onclick = () => {
        activePlanKey = md.name;
        panel.querySelectorAll(".plan-item.active").forEach(n => n.classList.remove("active"));
        btn.classList.add("active");
        focusItem("markdown", md.name);
      };
      const title = el("div", "plan-title" + (info.level > 1 ? " h" + info.level : ""));
      title.textContent = info.title;
      btn.appendChild(title);
      const meta = el("div", "plan-meta");
      meta.textContent = md.name;
      btn.appendChild(meta);
      const preview = mdPreview(md.content);
      if (preview) {
        const prev = el("div", "plan-preview");
        prev.textContent = preview;
        btn.appendChild(prev);
      }
      list.appendChild(btn);

      // Nested headings beyond the first (section title already shown)
      if (headings.length > 1) {
        for (let i = 1; i < headings.length; i++) {
          const h = headings[i];
          const sub = document.createElement("button");
          sub.type = "button";
          sub.className = "plan-item" + (activePlanKey === md.name + '#' + i ? " active" : "");
          sub.onclick = () => {
            activePlanKey = md.name + '#' + i;
            panel.querySelectorAll(".plan-item.active").forEach(n => n.classList.remove("active"));
            sub.classList.add("active");
            focusItem("markdown", md.name);
          };
          const st = el("div", "plan-title h" + Math.min(h.level, 3));
          st.textContent = h.title;
          sub.appendChild(st);
          list.appendChild(sub);
        }
      }
    }
    panel.appendChild(list);
  }

  function renderSidebar() {
    if (!inspectorEl || !state) return;
    inspectorEl.innerHTML = "";

    const tabs = el("div", "side-tabs");
    tabs.setAttribute("role", "tablist");
    for (const [id, label] of [["inspector", "Inspector"], ["plan", "Plan"]]) {
      const tab = document.createElement("button");
      tab.type = "button";
      tab.className = "side-tab" + (sideTab === id ? " active" : "");
      tab.setAttribute("role", "tab");
      tab.setAttribute("aria-selected", sideTab === id ? "true" : "false");
      tab.textContent = label;
      tab.onclick = () => {
        if (sideTab === id) return;
        sideTab = id;
        try { localStorage.setItem("labrs.sideTab", sideTab); } catch (_) {}
        renderSidebar();
      };
      tabs.appendChild(tab);
    }
    inspectorEl.appendChild(tabs);

    const panel = el("div", "side-panel");
    panel.setAttribute("role", "tabpanel");
    if (sideTab === "plan") renderPlanPanel(panel);
    else renderInspectorPanel(panel);
    inspectorEl.appendChild(panel);
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

  function makeMonaco(host, key, value, language, onSaveCmd, pendingMap, readOnly) {
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
      readOnly: !!readOnly,
      domReadOnly: !!readOnly,
    });
    if (sel) editor.setSelection(sel);
    if (onSaveCmd && !readOnly) {
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

  function iconBtn(symbol, title, onClick, extraClass) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "icon" + (extraClass ? " " + extraClass : "");
    b.title = title;
    b.setAttribute("aria-label", title);
    b.textContent = symbol;
    b.onclick = (e) => { e.stopPropagation(); onClick(); };
    return b;
  }

  function itemChrome(kind, name, index, total, extraRightBtns) {
    const right = el("div", "card-head-right");
    const up = iconBtn("↑", "Move up", () => send({ type: "move_item", kind, name, direction: "up" }));
    const down = iconBtn("↓", "Move down", () => send({ type: "move_item", kind, name, direction: "down" }));
    if (index <= 0) up.disabled = true;
    if (index >= total - 1) down.disabled = true;
    right.appendChild(up);
    right.appendChild(down);
    right.appendChild(el("div", "toolbar-sep"));
    if (extraRightBtns) extraRightBtns.forEach(b => right.appendChild(b));
    const del = iconBtn("✕", "Delete", () => {
      if (confirm(`Delete ${kind} “${name}”?`)) {
        send({ type: "delete_item", kind, name });
      }
    }, "danger");
    right.appendChild(del);
    return right;
  }

  function insertGap(host, afterKind, afterName, key, kinds) {
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
    for (const [label, kind] of kinds) {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.onclick = (e) => {
        e.stopPropagation();
        openInsertKey = null;
        const msg = { type: "add_item", kind };
        if (afterKind && afterName && afterKind !== "__start__") {
          msg.after_kind = afterKind;
          msg.after_name = afterName;
        }
        send(msg);
      };
      menu.appendChild(b);
    }
    gap.appendChild(plus);
    gap.appendChild(menu);
    host.appendChild(gap);
  }

  function footerAdd(host, labelText, kinds) {
    const foot = el("div", "footer-add");
    const label = el("div", "footer-label");
    label.textContent = labelText;
    const btns = el("div", "footer-btns");
    for (const [text, kind] of kinds) {
      const b = document.createElement("button");
      b.type = "button";
      b.innerHTML = `<span class="plus-mark">+</span>${text}`;
      b.onclick = () => send({ type: "add_item", kind });
      btns.appendChild(b);
    }
    foot.appendChild(label);
    foot.appendChild(btns);
    host.appendChild(foot);
  }

  function helperChrome(h, idx, total, vs) {
    const ro = sharedReadOnly;
    const card = el("div", "card helper");
    card.setAttribute("data-item", "helper:" + h.name);
    const head = el("div", "card-head");
    const left = el("div", "card-head-left");
    if (!ro) left.appendChild(kindSelect("helper", h.name, "helper"));
    else {
      const kind = el("span", "insp-kind helper");
      kind.textContent = "fn";
      left.appendChild(kind);
    }
    const title = el("span", "card-title");
    title.textContent = h.name;
    left.appendChild(title);
    const extras = [];
    let saveBtn = null;
    if (!ro) {
      saveBtn = document.createElement("button");
      saveBtn.className = "primary";
      saveBtn.textContent = "Save";
      saveBtn.onclick = () => {
        const ed = editors.get("helper:" + h.name);
        pendingFocus = { key: "helper:" + h.name, selection: ed && ed.getSelection() };
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
      extras.push(saveBtn);
    } else {
      const badge = el("span", "badge readonly");
      badge.textContent = "read-only";
      extras.push(badge);
    }
    const right = ro
      ? (() => { const r = el("div", "card-head-right"); extras.forEach(b => r.appendChild(b)); return r; })()
      : itemChrome("helper", h.name, idx, total, extras);
    head.appendChild(left);
    head.appendChild(right);
    card.appendChild(head);
    if (h.docs) {
      const docs = el("div", "docs");
      docs.textContent = h.docs;
      card.appendChild(docs);
    }
    const editorHost = el("div", "editor");
    card.appendChild(editorHost);
    sharedEl.appendChild(card);
    makeMonaco(
      editorHost,
      "helper:" + h.name,
      h.source,
      "rust",
      saveBtn ? () => saveBtn.click() : null,
      vs.pending,
      ro
    );
  }

  function joinPreamble(defs) {
    return (defs || []).map(d => d.source).filter(Boolean).join("\n");
  }

  function preambleCard(otherDefs, vs) {
    const ro = sharedReadOnly;
    const source = joinPreamble(otherDefs);
    const card = el("div", "card definition");
    card.setAttribute("data-item", "definition:__preamble__");
    const head = el("div", "card-head");
    const left = el("div", "card-head-left");
    const kind = el("span", "insp-kind use");
    kind.textContent = "preamble";
    left.appendChild(kind);
    const title = el("span", "card-title");
    title.textContent = "imports & prelude";
    left.appendChild(title);
    const right = el("div", "card-head-right");
    let saveBtn = null;
    if (ro) {
      const badge = el("span", "badge readonly");
      badge.textContent = "read-only";
      right.appendChild(badge);
    } else {
      saveBtn = document.createElement("button");
      saveBtn.className = "primary";
      saveBtn.textContent = "Save";
      saveBtn.onclick = () => {
        const ed = editors.get("def:__preamble__");
        pendingFocus = { key: "def:__preamble__", selection: ed && ed.getSelection() };
        if (document.getElementById("chk-auto").checked && state && state.cells_detail) {
          for (const c of state.cells_detail) {
            runningCells.add(c.name);
            patchCellRunning(c.name, true);
          }
        }
        send({ type: "edit_preamble", source: ed ? ed.getValue() : source });
      };
      right.appendChild(saveBtn);
    }
    head.appendChild(left);
    head.appendChild(right);
    card.appendChild(head);
    const editorHost = el("div", "editor");
    card.appendChild(editorHost);
    sharedEl.appendChild(card);
    makeMonaco(
      editorHost,
      "def:__preamble__",
      source,
      "rust",
      saveBtn ? () => saveBtn.click() : null,
      vs.pending,
      ro
    );
  }

  function definitionCard(d, vs) {
    const ro = sharedReadOnly;
    const card = el("div", "card definition");
    card.setAttribute("data-item", "definition:" + d.name);
    const head = el("div", "card-head");
    const left = el("div", "card-head-left");
    const kind = el("span", "insp-kind " + d.kind);
    kind.textContent = d.kind;
    left.appendChild(kind);
    const title = el("span", "card-title");
    title.textContent = d.name;
    left.appendChild(title);
    const right = el("div", "card-head-right");
    let saveBtn = null;
    if (ro) {
      const badge = el("span", "badge readonly");
      badge.textContent = "read-only";
      right.appendChild(badge);
    } else {
      saveBtn = document.createElement("button");
      saveBtn.className = "primary";
      saveBtn.textContent = "Save";
      saveBtn.onclick = () => {
        const ed = editors.get("def:" + d.name);
        pendingFocus = { key: "def:" + d.name, selection: ed && ed.getSelection() };
        if (document.getElementById("chk-auto").checked && state && state.cells_detail) {
          for (const c of state.cells_detail) {
            runningCells.add(c.name);
            patchCellRunning(c.name, true);
          }
        }
        send({ type: "edit_definition", name: d.name, source: ed ? ed.getValue() : d.source });
      };
      right.appendChild(saveBtn);
    }
    head.appendChild(left);
    head.appendChild(right);
    card.appendChild(head);
    const editorHost = el("div", "editor tall");
    card.appendChild(editorHost);
    sharedEl.appendChild(card);
    makeMonaco(
      editorHost,
      "def:" + d.name,
      d.source,
      "rust",
      saveBtn ? () => saveBtn.click() : null,
      vs.pending,
      ro
    );
  }

  function sharedToolbar() {
    const bar = el("div", "shared-toolbar");
    const title = el("div", "shared-toolbar-title");
    title.textContent = "Shared code";
    const label = document.createElement("label");
    label.className = "shared-readonly-toggle";
    label.title = "When on, helpers, structures, and preamble are read-only";
    const chk = document.createElement("input");
    chk.type = "checkbox";
    chk.id = "chk-shared-ro";
    chk.checked = sharedReadOnly;
    chk.onchange = () => {
      sharedReadOnly = !!chk.checked;
      persistSharedReadOnly();
      render();
    };
    label.appendChild(chk);
    label.appendChild(document.createTextNode("Read-only"));
    bar.appendChild(title);
    bar.appendChild(label);
    return bar;
  }

  function render() {
    if (!state) return;
    applyMainLayout();
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
    if (sharedEl) sharedEl.innerHTML = "";
    renderSidebar();

    const byCell = Object.fromEntries((state.cells_detail || []).map(c => [c.name, c]));
    const byHelper = Object.fromEntries((state.helpers_detail || []).map(h => [h.name, h]));
    const byMd = Object.fromEntries((state.markdown_detail || []).map(m => [m.name, m]));
    const byDef = Object.fromEntries((state.definitions_detail || []).map(d => [d.name, d]));
    const cellState = Object.fromEntries((snap.cells || []).map(c => [c.name, c]));
    const { primary: primaryDefs, other: otherDefs } = partitionDefs(state.definitions_detail);

    const notebookOrder = [];
    const helperOrder = [];
    if (snap.order && snap.order.length) {
      for (const e of snap.order) {
        if (e.kind === "cell") notebookOrder.push({ kind: "cell", id: e.id });
        else if (e.kind === "markdown") notebookOrder.push({ kind: "markdown", id: e.id });
        else if (e.kind === "helper") helperOrder.push(e.name);
      }
    } else {
      for (const m of state.markdown_detail || []) notebookOrder.push({ kind: "markdown", id: m.name });
      for (const c of state.cells_detail || []) notebookOrder.push({ kind: "cell", id: c.name });
      for (const h of state.helpers_detail || []) helperOrder.push(h.name);
    }
    // Preserve file order for primary defs via snapshot.order when available
    const defOrderNames = [];
    if (snap.order && snap.order.length) {
      for (const e of snap.order) {
        if (e.kind === "definition" && byDef[e.name] && DEF_PRIMARY.includes(byDef[e.name].kind)) {
          defOrderNames.push(e.name);
        }
      }
    }
    const primaryOrdered = defOrderNames.length
      ? defOrderNames.map(n => byDef[n]).filter(Boolean)
      : primaryDefs;

    const nbKinds = [["Cell", "cell"], ["Markdown", "markdown"]];
    insertGap(notebookEl, "__start__", "__start__", "nb-top", nbKinds);

    notebookOrder.forEach((entry, idx) => {
      if (entry.kind === "markdown") {
        const md = byMd[entry.id];
        if (!md) return;
        const card = el("div", "card md");
        card.setAttribute("data-item", "markdown:" + md.name);
        const head = el("div", "card-head");
        const left = el("div", "card-head-left");
        left.appendChild(kindSelect("markdown", md.name, "markdown"));
        const title = el("span", "card-title");
        title.textContent = md.name;
        left.appendChild(title);
        const editBtn = document.createElement("button");
        const editing = mdEditing.has(md.name);
        editBtn.textContent = editing ? "Save" : "Edit";
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
        const body = el("div", editing ? "md-body hidden" : "md-body");
        body.innerHTML = `<p>${simpleMarkdown(md.content)}</p>`;
        const editWrap = el("div", editing ? "md-edit-wrap open" : "md-edit-wrap");
        const editorHost = el("div", "editor md-edit");
        editWrap.appendChild(editorHost);
        const right = itemChrome("markdown", md.name, idx, notebookOrder.length, [editBtn]);
        head.appendChild(left);
        head.appendChild(right);
        card.appendChild(head);
        card.appendChild(body);
        card.appendChild(editWrap);
        notebookEl.appendChild(card);
        if (editing) {
          makeMonaco(editorHost, "md:" + md.name, md.content, "markdown", () => editBtn.click(), vs.pending);
        }
        insertGap(notebookEl, "markdown", md.name, "after-md-" + md.name, nbKinds);
      } else {
        const c = byCell[entry.id];
        if (!c) return;
        const st = cellState[c.name];
        const card = el("div", `card ${statusClass(st, c.name)}`);
        card.setAttribute("data-cell-card", c.name);
        card.setAttribute("data-item", "cell:" + c.name);
        const head = el("div", "card-head");
        const left = el("div", "card-head-left");
        left.appendChild(kindSelect("cell", c.name, "cell"));
        const deps = paramNames(c.params);
        const titleWrap = el("div");
        titleWrap.innerHTML = `<div class="card-title">${escapeHtml(c.name)}</div>
          <div class="meta">→ ${escapeHtml(c.return_type)}${deps.length ? " · deps: " + deps.map(escapeHtml).join(", ") : ""}</div>`;
        left.appendChild(titleWrap);

        const runBtn = iconBtn("▶", "Run cell", () => {
          const ed = editors.get(c.name);
          const source = ed ? ed.getValue() : c.source;
          pendingFocus = { key: c.name, selection: ed && ed.getSelection() };
          runningCells.add(c.name);
          patchCellRunning(c.name, true);
          send({ type: "edit_cell", name: c.name, source });
          setTimeout(() => send({ type: "run_cell", name: c.name }), 50);
        }, "run-icon");
        const right = itemChrome("cell", c.name, idx, notebookOrder.length, [runBtn, statusBadge(st, c.name)]);
        head.appendChild(left);
        head.appendChild(right);
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
        insertGap(notebookEl, "cell", c.name, "after-cell-" + c.name, nbKinds);
      }
    });

    footerAdd(notebookEl, "Add to notebook", nbKinds);

    // Shared pane
    if (sharedEl) {
      sharedEl.appendChild(sharedToolbar());
      const shKinds = [["Helper", "helper"]];
      const helpersLabel = el("div", "shared-section-label");
      helpersLabel.textContent = "Helpers";
      sharedEl.appendChild(helpersLabel);
      if (!sharedReadOnly) {
        insertGap(sharedEl, "__start__", "__start__", "sh-top", shKinds);
      }

      helperOrder.forEach((name, idx) => {
        const h = byHelper[name];
        if (!h) return;
        helperChrome(h, idx, helperOrder.length, vs);
        if (!sharedReadOnly) {
          insertGap(sharedEl, "helper", h.name, "after-helper-" + h.name, shKinds);
        }
      });
      if (!helperOrder.length) {
        const empty = el("div", "insp-empty");
        empty.textContent = "No helpers yet";
        sharedEl.appendChild(empty);
      }
      if (!sharedReadOnly) {
        footerAdd(sharedEl, "Add to shared", shKinds);
      }

      const structsLabel = el("div", "shared-section-label");
      structsLabel.textContent = "Structures";
      sharedEl.appendChild(structsLabel);
      if (!primaryOrdered.length) {
        const empty = el("div", "insp-empty");
        empty.textContent = "No shared structures";
        sharedEl.appendChild(empty);
      } else {
        for (const d of primaryOrdered) definitionCard(d, vs);
      }

      if (otherDefs.length) {
        const otherLabel = el("div", "shared-section-label");
        otherLabel.textContent = "Preamble";
        sharedEl.appendChild(otherLabel);
        preambleCard(otherDefs, vs);
      }
    }

    restoreViewState(vs);

    document.onclick = () => {
      if (openInsertKey) {
        openInsertKey = null;
        document.querySelectorAll(".insert-gap.open").forEach(n => n.classList.remove("open"));
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
  const homeBtn = document.getElementById("btn-home");
  if (homeBtn) {
    homeBtn.onclick = () => {
      if (document.body.classList.contains("welcome-mode")) return;
      send({ type: "close_notebook" });
    };
  }
  const newNbBtn = document.getElementById("btn-new-notebook");
  if (newNbBtn) {
    newNbBtn.onclick = () => {
      const name = prompt("Notebook name (without .rs)", "notebook");
      if (!name) return;
      const dir = (welcomeState && welcomeState.cwd) || "";
      send({ type: "create_notebook", name, dir });
    };
  }

  document.querySelectorAll(".main-tab").forEach(tab => {
    tab.onclick = () => {
      if (mainMode === "split") {
        mainMode = "tabs";
      }
      mainTab = tab.getAttribute("data-main-tab") || "notebook";
      applyMainLayout();
      if (state) render();
    };
  });
  const detachBtn = document.getElementById("btn-detach-shared");
  if (detachBtn) {
    detachBtn.onclick = () => {
      mainMode = mainMode === "split" ? "tabs" : "split";
      if (mainMode === "tabs" && mainTab !== "notebook" && mainTab !== "shared") mainTab = "notebook";
      applyMainLayout();
      if (state) render();
    };
  }
  const splitHandle = document.getElementById("split-handle");
  if (splitHandle && mainPanesEl) {
    let dragging = false;
    splitHandle.addEventListener("mousedown", (e) => {
      if (mainMode !== "split") return;
      dragging = true;
      splitHandle.classList.add("dragging");
      e.preventDefault();
    });
    window.addEventListener("mousemove", (e) => {
      if (!dragging) return;
      const rect = mainPanesEl.getBoundingClientRect();
      const x = e.clientX - rect.left;
      splitLeftFrac = Math.min(0.72, Math.max(0.28, x / rect.width));
      mainPanesEl.style.setProperty("--split-left", splitLeftFrac + "fr");
      mainPanesEl.style.setProperty("--split-right", (1 - splitLeftFrac) + "fr");
    });
    window.addEventListener("mouseup", () => {
      if (!dragging) return;
      dragging = false;
      splitHandle.classList.remove("dragging");
      persistMainLayout();
    });
  }
  applyMainLayout();

  const workspaceEl = document.querySelector(".workspace");
  const toggleBtn = document.getElementById("btn-toggle-inspector");
  function setInspectorOpen(open) {
    if (!workspaceEl || !toggleBtn) return;
    workspaceEl.classList.toggle("inspector-collapsed", !open);
    toggleBtn.setAttribute("aria-expanded", open ? "true" : "false");
    toggleBtn.title = open ? "Hide inspector" : "Show inspector";
    try { localStorage.setItem("labrs.inspectorOpen", open ? "1" : "0"); } catch (_) {}
  }
  (function initInspectorToggle() {
    let open = true;
    try {
      const stored = localStorage.getItem("labrs.inspectorOpen");
      if (stored === "0") open = false;
      if (stored === "1") open = true;
    } catch (_) {}
    setInspectorOpen(open);
    if (toggleBtn) {
      toggleBtn.onclick = () => {
        const next = toggleBtn.getAttribute("aria-expanded") !== "true";
        setInspectorOpen(next);
      };
    }
  })();

  require(["vs/editor/editor.main"], () => connect());
})();
"#;
