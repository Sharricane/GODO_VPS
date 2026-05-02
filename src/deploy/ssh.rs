use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::process::Stdio;
use tokio::process::Command;

/// Thin async wrapper around system `ssh` / `scp`.
/// Uses StrictHostKeyChecking=accept-new so first connect works without manual confirmation.
pub struct Ssh {
    host: String,
    port: u16,
    user: String,
    key:  Option<String>,
}

impl Ssh {
    pub fn new(host: &str, port: u16, user: &str, key: Option<&str>) -> Self {
        Self {
            host: host.to_string(),
            port,
            user: user.to_string(),
            key: key.map(|s| s.to_string()),
        }
    }

    fn base_args(&self) -> Vec<String> {
        let mut args = vec![
            "-o".into(), "StrictHostKeyChecking=accept-new".into(),
            "-o".into(), "ConnectTimeout=30".into(),
            "-o".into(), "BatchMode=yes".into(),
            "-p".into(), self.port.to_string(),
        ];
        if let Some(ref k) = self.key {
            args.push("-i".into());
            args.push(k.clone());
        }
        args.push(format!("{}@{}", self.user, self.host));
        args
    }

    /// Run a command on the remote host; return stdout as String.
    pub async fn exec(&self, cmd: &str) -> Result<String> {
        let mut args = self.base_args();
        args.push(cmd.to_string());

        let out = Command::new("ssh")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("ssh binary not found")?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            anyhow::bail!("remote command failed ({}):\n{stderr}", out.status);
        }

        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Write a string as a file on the remote host (via base64 pipe to avoid quoting hell).
    pub async fn write_file(&self, content: &str, remote_path: &str) -> Result<()> {
        let encoded = STANDARD.encode(content.as_bytes());
        // Split into lines to avoid ARG_MAX issues for large configs
        let cmd = format!(
            "mkdir -p $(dirname {remote_path}) && \
             printf '%s' '{encoded}' | base64 -d > {remote_path}"
        );
        self.exec(&cmd).await?;
        Ok(())
    }

    /// Check TCP connectivity to the host:port without SSH.
    pub async fn tcp_open(host: &str, port: u16, timeout_secs: u64) -> bool {
        let addr = format!("{host}:{port}");
        tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        .map(|r| r.is_ok())
        .unwrap_or(false)
    }
}
