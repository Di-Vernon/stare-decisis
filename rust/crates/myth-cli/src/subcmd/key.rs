//! `myth key set | show | clear` — Anthropic API credential management.
//!
//! Storage format: `~/.myth/credentials`, plain-text `KEY=VALUE` lines,
//! 0600 permissions. v0.3+ may migrate to OS keychain integration.
//!
//! Phase 5 Stage B.2 brought this stub to life so the post-experiment
//! key (rotated after revoking the prior one) has a single canonical
//! home instead of living in `~/myth/experiment/.../.env`.

use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};
use myth_common::credentials_path;

use crate::args::{KeyAction, KeyArgs};

const ENV_VAR: &str = "ANTHROPIC_API_KEY";
const FILE_PERMS: u32 = 0o600;

pub async fn run(args: KeyArgs) -> Result<ExitCode> {
    match args.action {
        KeyAction::Set { from_stdin } => set(from_stdin),
        KeyAction::Show => show(),
        KeyAction::Clear => clear(),
    }
}

fn set(from_stdin: bool) -> Result<ExitCode> {
    let key = if from_stdin {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("reading API key from stdin")?;
        buf.trim().to_string()
    } else {
        return Err(anyhow!(
            "interactive prompt not yet supported; pipe the key in: \
             `echo \"sk-ant-...\" | myth key set --from-stdin`"
        ));
    };

    if key.is_empty() {
        return Err(anyhow!("empty key rejected"));
    }
    if !key.starts_with("sk-ant-") {
        return Err(anyhow!(
            "key does not look like an Anthropic key (expected sk-ant- prefix)"
        ));
    }

    let path = credentials_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(&path, format!("{}={}\n", ENV_VAR, key))
        .with_context(|| format!("writing {}", path.display()))?;
    set_perms_0600(&path)?;
    println!("API key saved to {} (0600)", path.display());
    Ok(ExitCode::SUCCESS)
}

fn show() -> Result<ExitCode> {
    let path = credentials_path();
    if !path.exists() {
        eprintln!(
            "no credentials at {}; run `myth key set --from-stdin`",
            path.display()
        );
        return Ok(ExitCode::FAILURE);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim();
            let masked = if v.len() <= 12 {
                "*".repeat(v.len())
            } else {
                format!("{}...{}", &v[..7], &v[v.len() - 4..])
            };
            println!("{} = {}", k.trim(), masked);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn clear() -> Result<ExitCode> {
    let path = credentials_path();
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("removing {}", path.display()))?;
        println!("removed {}", path.display());
    } else {
        println!("no credentials at {}; nothing to clear", path.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn set_perms_0600(path: &Path) -> Result<()> {
    let mut perms = fs::metadata(path)
        .with_context(|| format!("stat {}", path.display()))?
        .permissions();
    perms.set_mode(FILE_PERMS);
    fs::set_permissions(path, perms)
        .with_context(|| format!("chmod {}", path.display()))?;
    Ok(())
}

// `myth key load` is intentionally NOT a subcommand. Library consumers
// (Rust + Python) read the file directly via `myth_common::credentials_path`
// and the format documented above.

pub fn read_credentials(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == ENV_VAR {
                return Ok(v.trim().to_string());
            }
        }
    }
    Err(anyhow!("{} not found in {}", ENV_VAR, path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_credentials_extracts_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("credentials");
        fs::write(&path, "ANTHROPIC_API_KEY=sk-ant-test-12345\n").unwrap();
        let key = read_credentials(&path).unwrap();
        assert_eq!(key, "sk-ant-test-12345");
    }

    #[test]
    fn read_credentials_skips_comments_and_blank() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("credentials");
        fs::write(
            &path,
            "# header\n\nANTHROPIC_API_KEY=sk-ant-real\nOTHER_VAR=ignored\n",
        )
        .unwrap();
        assert_eq!(read_credentials(&path).unwrap(), "sk-ant-real");
    }

    #[test]
    fn read_credentials_errors_on_missing_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("credentials");
        fs::write(&path, "OTHER=foo\n").unwrap();
        assert!(read_credentials(&path).is_err());
    }
}
