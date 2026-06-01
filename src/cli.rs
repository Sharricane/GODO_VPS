use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "godo-vps",
    about = "SG VPS proxy manager — VLESS+Reality + Hysteria2 + multi-device Clash config",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Bootstrap a fresh VPS: install sing-box, tune BBR, harden SSH, deploy config
    Bootstrap(BootstrapArgs),
    /// Generate sing-box server config JSON and save to VPS
    GenServerConfig(ServerConfigArgs),
    /// Generate Clash client config with Loyalsoldier rules (CN+HK auto-split)
    GenClientConfig(ClientConfigArgs),
    /// Print subscription links for all client apps
    Sub(SubArgs),
    /// One-shot node health check
    Status(StatusArgs),
    /// Run continuous health monitor with Telegram alerts
    Monitor(MonitorArgs),
    /// Pull full read-only diagnostics: BBR, TCP retrans, UDP errors, scanner IPs, journal errors
    Diag(DiagArgs),
}

// ── diag ─────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct DiagArgs {
    #[arg(long, env = "VPS_HOST")]
    pub host: String,

    #[arg(long, env = "VPS_SSH_PORT", default_value = "22")]
    pub ssh_port: u16,

    #[arg(long, env = "VPS_USER", default_value = "root")]
    pub user: String,

    #[arg(long, env = "VPS_SSH_KEY")]
    pub ssh_key: Option<String>,

    #[arg(long, default_value = "1", help = "Look-back window in hours for journal errors")]
    pub since_hours: u32,
}

// ── bootstrap ───────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct BootstrapArgs {
    #[arg(long, env = "VPS_HOST", help = "VPS public IP")]
    pub host: String,

    #[arg(long, env = "VPS_SSH_PORT", default_value = "22")]
    pub ssh_port: u16,

    #[arg(long, env = "VPS_USER", default_value = "root")]
    pub user: String,

    #[arg(long, env = "VPS_SSH_KEY", help = "Path to SSH private key")]
    pub ssh_key: Option<String>,

    #[arg(long, env = "SINGBOX_VLESS_PORT", default_value = "443")]
    pub vless_port: u16,

    #[arg(long, env = "SINGBOX_HY2_PORT", default_value = "8443")]
    pub hy2_port: u16,

    #[arg(long, env = "REALITY_SNI", default_value = "www.cloudflare.com")]
    pub sni: String,

    #[arg(long, short, help = "Skip confirmation prompts")]
    pub yes: bool,

    #[arg(long, help = "New SSH port to set after hardening (leave blank to keep current)")]
    pub new_ssh_port: Option<u16>,
}

// ── gen-server-config ────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ServerConfigArgs {
    #[arg(long, env = "VPS_HOST")]
    pub host: String,

    #[arg(long, env = "VPS_SSH_PORT", default_value = "22")]
    pub ssh_port: u16,

    #[arg(long, env = "VPS_USER", default_value = "root")]
    pub user: String,

    #[arg(long, env = "VPS_SSH_KEY")]
    pub ssh_key: Option<String>,

    #[arg(long, env = "SINGBOX_VLESS_PORT", default_value = "443")]
    pub vless_port: u16,

    #[arg(long, env = "SINGBOX_HY2_PORT", default_value = "8443")]
    pub hy2_port: u16,

    #[arg(long, env = "REALITY_SNI", default_value = "www.cloudflare.com")]
    pub sni: String,

    #[arg(long, env = "VLESS_UUID", help = "Leave blank to auto-generate")]
    pub uuid: Option<String>,

    #[arg(long, env = "HY2_PASSWORD", help = "Leave blank to auto-generate")]
    pub hy2_password: Option<String>,

    #[arg(long, env = "REALITY_PRIVATE_KEY", help = "Leave blank to auto-generate")]
    pub reality_private_key: Option<String>,

    #[arg(long, env = "REALITY_PUBLIC_KEY")]
    pub reality_public_key: Option<String>,

    #[arg(long, env = "REALITY_SHORT_ID", help = "Leave blank to auto-generate")]
    pub reality_short_id: Option<String>,
}

// ── gen-client-config ────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ClientConfigArgs {
    #[arg(long, env = "VPS_HOST")]
    pub host: String,

    #[arg(long, env = "SINGBOX_VLESS_PORT", default_value = "443")]
    pub vless_port: u16,

    #[arg(long, env = "SINGBOX_HY2_PORT", default_value = "8443")]
    pub hy2_port: u16,

    #[arg(long, env = "REALITY_SNI", default_value = "www.cloudflare.com")]
    pub sni: String,

    #[arg(long, env = "VLESS_UUID")]
    pub uuid: String,

    #[arg(long, env = "HY2_PASSWORD")]
    pub hy2_password: String,

    #[arg(long, env = "REALITY_PUBLIC_KEY")]
    pub reality_public_key: String,

    #[arg(long, env = "REALITY_SHORT_ID")]
    pub reality_short_id: String,

    #[arg(long, env = "CF_SUBDOMAIN", help = "Cloudflare-fronted subdomain for VMess-WS (e.g. cdn.example.com); omit to skip CDN node")]
    pub cdn_host: Option<String>,

    #[arg(long, help = "Generate iOS-compatible config (strips dns.listen, external-controller, fake-ip; trims rule-providers for 50MB VPN extension limit)")]
    pub ios: bool,

    #[arg(long, help = "Generate ULTRA-minimal iOS config (no rule-providers, no GEOIP, IPv4-only) to dodge ClashMi issue #195 rules-mode bug")]
    pub ios_minimal: bool,

    #[arg(long, help = "Generate Clash for Windows (legacy Dreamacro core) variant — drops VLESS/Reality/Hysteria2/sniffer which CFW doesn't support, keeps only CDN-VMess")]
    pub cfw: bool,

    #[arg(long, short, help = "Output file path (default: stdout)")]
    pub output: Option<String>,
}

// ── sub ──────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct SubArgs {
    #[arg(long, env = "VPS_HOST")]
    pub host: String,

    #[arg(long, env = "SINGBOX_VLESS_PORT", default_value = "443")]
    pub vless_port: u16,

    #[arg(long, env = "SINGBOX_HY2_PORT", default_value = "8443")]
    pub hy2_port: u16,

    #[arg(long, env = "REALITY_SNI", default_value = "www.cloudflare.com")]
    pub sni: String,

    #[arg(long, env = "VLESS_UUID")]
    pub uuid: String,

    #[arg(long, env = "HY2_PASSWORD")]
    pub hy2_password: String,

    #[arg(long, env = "REALITY_PUBLIC_KEY")]
    pub reality_public_key: String,

    #[arg(long, env = "REALITY_SHORT_ID")]
    pub reality_short_id: String,
}

// ── status ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct StatusArgs {
    #[arg(long, env = "VPS_HOST")]
    pub host: String,

    #[arg(long, env = "SINGBOX_VLESS_PORT", default_value = "443")]
    pub vless_port: u16,

    #[arg(long, env = "SINGBOX_HY2_PORT", default_value = "8443")]
    pub hy2_port: u16,

    #[arg(long, env = "VPS_SSH_PORT", default_value = "22")]
    pub ssh_port: u16,

    #[arg(long, env = "VPS_USER", default_value = "root")]
    pub user: String,

    #[arg(long, env = "VPS_SSH_KEY")]
    pub ssh_key: Option<String>,
}

// ── monitor ──────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct MonitorArgs {
    #[arg(long, env = "VPS_HOST")]
    pub host: String,

    #[arg(long, env = "SINGBOX_VLESS_PORT", default_value = "443")]
    pub vless_port: u16,

    #[arg(long, env = "SINGBOX_HY2_PORT", default_value = "8443")]
    pub hy2_port: u16,

    #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
    pub tg_token: Option<String>,

    #[arg(long, env = "TELEGRAM_CHAT_ID")]
    pub tg_chat_id: Option<String>,

    #[arg(long, env = "HEALTH_CHECK_INTERVAL", default_value = "300",
          help = "Seconds between checks")]
    pub interval: u64,

    #[arg(long, env = "VPS_SSH_PORT", default_value = "22")]
    pub ssh_port: u16,

    #[arg(long, env = "VPS_USER", default_value = "root")]
    pub user: String,

    #[arg(long, env = "VPS_SSH_KEY")]
    pub ssh_key: Option<String>,

    #[arg(long, env = "ALLOW_AUTO_RESTART",
          help = "Auto-restart sing-box when unhealthy (off by default — would kick all clients)")]
    pub allow_restart: bool,
}
