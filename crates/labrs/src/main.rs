use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use labrs_core::fmt::rustfmt_file;
use labrs_core::graph::build_graph;
use labrs_core::parse::parse_notebook;
use labrs_core::{ExecuteOptions, Executor};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "labrs", version, about = "Reactive Rust notebooks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new notebook
    New {
        name: String,
        /// Add as a workspace member Cargo.toml entry
        #[arg(long)]
        workspace: bool,
    },
    /// Parse and print the dependency graph
    Graph { file: PathBuf },
    /// Format the notebook with rustfmt
    Fmt { file: PathBuf },
    /// Run all cells in topological order
    Run {
        file: PathBuf,
        /// Skip rustfmt before run
        #[arg(long)]
        no_fmt: bool,
    },
    /// Start the interactive web UI
    Edit {
        /// Notebook to open (optional — omit for welcome / file browser)
        file: Option<PathBuf>,
        #[arg(long, default_value = "8080")]
        port: u16,
        /// Disable automatic re-run of dependent cells (Pluto-style cascade is on by default)
        #[arg(long)]
        no_auto: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::New { name, workspace } => cmd_new(&name, workspace),
        Commands::Graph { file } => cmd_graph(&file),
        Commands::Fmt { file } => {
            rustfmt_file(&file)?;
            println!("Formatted {}", file.display());
            Ok(())
        }
        Commands::Run { file, no_fmt } => cmd_run(&file, !no_fmt),
        Commands::Edit {
            file,
            port,
            no_auto,
        } => {
            println!("{}", welcome(port, file.as_deref()));
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(labrs_server::serve_with_options(file, port, !no_auto))
        }
    }
}

fn welcome(port: u16, file: Option<&Path>) -> String {
    let url = format!("http://127.0.0.1:{port}");
    let tip = random_tip();
    let file_line = match file {
        Some(p) => format!("\n\t  {}", format!("notebook  {}", p.display()).dimmed()),
        None => format!("\n\t  {}", "welcome · browse or create a notebook".dimmed()),
    };
    format!(
        "\n\t{}\n\t ➜  {}:  {}{}\n\n\t{} {}\n",
        "labrs".green().bold(),
        "UI".green(),
        url.cyan().underline(),
        file_line,
        "💡 Tip:".yellow().bold(),
        tip.dimmed(),
    )
}

fn random_tip() -> &'static str {
    const TIPS: &[&str] = &[
        "labrs new hello          — scaffold a notebook (.rs + optional Cargo.toml)",
        "labrs edit               — open the UI without a file (file browser + New notebook)",
        "labrs edit notebook.rs   — jump straight into a notebook",
        "labrs edit --port 3000   — serve the UI on another port",
        "labrs edit --no-auto     — disable Auto-run cascade (manual dirty mode)",
        "labrs graph notebook.rs  — print the cell DAG, helpers, and diagnostics",
        "labrs run notebook.rs    — topo-run all cells from the CLI",
        "labrs run notebook.rs --no-fmt  — skip rustfmt before a CLI run",
        "labrs fmt notebook.rs    — format the notebook with rustfmt",
        "In the UI: Detach Shared to edit helpers/structures beside Notebook",
        "In Shared: uncheck Read-only to edit helpers, structures, and preamble",
        "Cell params are dependencies: fn report(val: &u16) depends on cell `val`",
        "Plain fns are helpers (not in the DAG); mark cells with #[labrs::cell]",
        "Click the labrs brand in the UI to return to the file browser",
        "Toggle Auto-run in the toolbar to cascade dependents when outputs change",
    ];
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0)
        % TIPS.len();
    TIPS[idx]
}

fn cmd_new(name: &str, workspace: bool) -> Result<()> {
    let stem = name.trim_end_matches(".rs");
    let file = PathBuf::from(format!("{stem}.rs"));
    if file.exists() {
        bail!("{} already exists", file.display());
    }

    let contents = format!(
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
    );
    fs::write(&file, contents)?;
    ensure_cargo_toml(stem, workspace)?;
    println!("Created {}", file.display());
    println!("  labrs run {}", file.display());
    println!("  labrs edit {}", file.display());
    Ok(())
}

fn ensure_cargo_toml(stem: &str, _workspace: bool) -> Result<()> {
    let cargo = Path::new("Cargo.toml");
    if cargo.exists() {
        let text = fs::read_to_string(cargo)?;
        if !text.contains("labrs") {
            println!(
                "Tip: add labrs to Cargo.toml for rust-analyzer:\n\n[dependencies]\nlabrs = \"0.1\"\n"
            );
        }
        return Ok(());
    }

    let toml = format!(
        r#"[package]
name = "{stem}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{stem}"
path = "{stem}.rs"

[dependencies]
labrs = "0.1"
"#
    );
    fs::write(cargo, toml)?;
    println!("Wrote Cargo.toml for rust-analyzer support");
    Ok(())
}

fn cmd_graph(file: &Path) -> Result<()> {
    let nb = parse_notebook(file)?;
    let g = build_graph(&nb);

    println!("Notebook: {}", file.display());
    println!("Cells: {}", nb.cells.len());
    println!("Helpers: {}", nb.helpers.len());
    println!("Definitions: {}", nb.definitions.len());
    println!("Markdown: {}", nb.markdown.len());
    println!();

    if !nb.helpers.is_empty() {
        println!("Helpers (not in DAG):");
        for h in &nb.helpers {
            println!(
                "  - {}{}",
                h.name,
                if h.explicit { " #[labrs::helper]" } else { "" }
            );
        }
        println!();
    }

    println!("Edges:");
    if g.edges.is_empty() {
        println!("  (none)");
    } else {
        for e in &g.edges {
            println!("  {} -> {} (param {})", e.from, e.to, e.param_name);
        }
    }
    println!();
    println!("Topological order: {}", g.order.join(" → "));
    println!();

    if g.diagnostics.is_empty() {
        println!("Diagnostics: ok");
    } else {
        println!("Diagnostics:");
        for d in &g.diagnostics {
            let loc = d.cell.as_deref().unwrap_or("-");
            println!("  [{:?}] ({loc}) {}", d.severity, d.message);
        }
    }
    Ok(())
}

fn cmd_run(file: &Path, rustfmt: bool) -> Result<()> {
    let opts = ExecuteOptions {
        rustfmt,
        cache_dir: Some(
            file.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(".labrs")
                .join("cache"),
        ),
    };
    let executor = Executor::new(&opts)?;
    let outputs = executor
        .run_notebook(file, &opts)
        .with_context(|| format!("failed to run {}", file.display()))?;

    for out in &outputs {
        println!("═══ cell `{}` ═══", out.cell);
        if out.success {
            println!("status: ok");
            println!(
                "return: {}",
                serde_json::to_string_pretty(&out.value).unwrap_or_default()
            );
        } else {
            println!("status: error");
            if let Some(err) = &out.error {
                println!("{err}");
            }
        }
        if !out.stdout.is_empty() {
            println!("── stdout ──\n{}", out.stdout);
        }
        if !out.stderr.is_empty() {
            println!("── stderr ──\n{}", out.stderr);
        }
        println!();
    }
    Ok(())
}
