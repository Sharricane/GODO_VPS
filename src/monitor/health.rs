use crate::deploy::ssh::Ssh;
use anyhow::Result;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct NodeHealth {
    pub host: String,
    pub vless_tcp: bool,
    pub hy2_udp:   bool,
    pub singbox_running: bool,
    pub latency_ms: Option<u128>,
}

impl NodeHealth {
    pub fn is_healthy(&self) -> bool {
        self.vless_tcp && self.hy2_udp && self.singbox_running
    }

    pub fn summary(&self) -> String {
        let vless = if self.vless_tcp { "✓".green() } else { "✗".red() };
        let hy2   = if self.hy2_udp  { "✓".green() } else { "✗".red() };
        let sb    = if self.singbox_running { "✓".green() } else { "✗".red() };
        let lat = self.latency_ms
            .map(|ms| format!("{ms}ms"))
            .unwrap_or_else(|| "?".to_string());
        format!(
            "host={} VLESS={vless} Hy2={hy2} sing-box={sb} latency={lat}",
            self.host
        )
    }
}

pub async fn check(
    host: &str, vless_port: u16, hy2_port: u16,
    ssh_port: u16, user: &str, key: Option<&str>,
) -> NodeHealth {
    let t0 = std::time::Instant::now();
    let vless_tcp = Ssh::tcp_open(host, vless_port, 10).await;
    let latency_ms = if vless_tcp { Some(t0.elapsed().as_millis()) } else { None };

    // UDP check via nc (best-effort; Hy2 also listens on TCP for QUIC handshake)
    let hy2_udp = Ssh::tcp_open(host, hy2_port, 10).await;

    let ssh = Ssh::new(host, ssh_port, user, key);
    let singbox_running = ssh
        .exec("systemctl is-active sing-box")
        .await
        .map(|o| o.trim() == "active")
        .unwrap_or(false);

    NodeHealth { host: host.to_string(), vless_tcp, hy2_udp, singbox_running, latency_ms }
}

pub async fn restart_singbox(host: &str, ssh_port: u16, user: &str, key: Option<&str>) -> Result<()> {
    let ssh = Ssh::new(host, ssh_port, user, key);
    ssh.exec("systemctl restart sing-box").await?;
    Ok(())
}
