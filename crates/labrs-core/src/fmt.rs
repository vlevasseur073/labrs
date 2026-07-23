//! rustfmt integration for notebooks and cell snippets.

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Format an entire notebook file in place with rustfmt.
pub fn rustfmt_file(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let status = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(path)
        .status()
        .context("failed to run rustfmt (is it installed?)")?;
    if !status.success() {
        bail!("rustfmt failed on {}", path.display());
    }
    Ok(())
}

/// Format a Rust source snippet with rustfmt; returns formatted text.
pub fn rustfmt_source(source: &str) -> Result<String> {
    let mut child = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn rustfmt")?;

    {
        let mut stdin = child.stdin.take().context("failed to open rustfmt stdin")?;
        stdin.write_all(source.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("rustfmt failed: {err}");
    }
    Ok(String::from_utf8(output.stdout)?)
}

/// Format a cell function source. Falls back to original on rustfmt failure.
pub fn rustfmt_cell_source(source: &str) -> String {
    rustfmt_source(source).unwrap_or_else(|_| source.to_string())
}
