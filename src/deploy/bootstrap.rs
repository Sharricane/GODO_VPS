use crate::{
    cli::BootstrapArgs,
    config::{
        keys::{generate_hy2_password, generate_reality_keypair, generate_uuid, or_generate},
        singbox,
    },
    deploy::ssh::Ssh,
};
use anyhow::Result;
use colored::Colorize;

pub async fn run(args: BootstrapArgs) -> Result<()> {
    let ssh = Ssh::new(&args.host, args.ssh_port, &args.user, args.ssh_key.as_deref());

    println!("{}", "═══ GODO-VPS Bootstrap ═══".cyan().bold());
    println!("  Host : {}:{}", args.host, args.ssh_port);
    println!("  User : {}", args.user);
    println!("  Protocols : VLESS+Reality (:{}) + Hysteria2 (:{})", args.vless_port, args.hy2_port);
    println!("  SNI : {}", args.sni);
    println!();

    if !args.yes {
        print!("Proceed? [y/N] ");
        use std::io::{BufRead, Write};
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::BufReader::new(std::io::stdin()).read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // ── 1. Generate credentials ──────────────────────────────────────────────
    step("Generating credentials");
    let (uuid, _) = or_generate(None, generate_uuid);
    let (hy2_pass, _) = or_generate(None, generate_hy2_password);
    let kp = generate_reality_keypair();
    ok();

    // ── 2. System update ─────────────────────────────────────────────────────
    step("Updating system packages");
    ssh.exec("DEBIAN_FRONTEND=noninteractive apt-get update -qq && \
              apt-get upgrade -y -qq").await?;
    ok();

    // ── 3. Install sing-box ──────────────────────────────────────────────────
    step("Installing sing-box");
    ssh.exec(
        "curl -fsSL https://sing-box.app/deb-install.sh | bash || \
         (apt-get install -y sing-box 2>/dev/null)"
    ).await?;
    ok();

    // ── 4. Self-signed cert for Hysteria2 ────────────────────────────────────
    step("Generating self-signed TLS cert for Hysteria2");
    ssh.exec(
        "openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:P-256 \
         -keyout /etc/sing-box/hy2.key -out /etc/sing-box/hy2.crt \
         -days 3650 -nodes -subj '/CN=bing.com' 2>/dev/null && \
         chmod 600 /etc/sing-box/hy2.key"
    ).await?;
    ok();

    // ── 5. Upload sing-box config ─────────────────────────────────────────────
    step("Uploading sing-box config");
    let config = singbox::build_server_config(
        &uuid, &hy2_pass, &kp.private_key, &kp.short_id,
        &args.sni, args.vless_port, args.hy2_port,
    );
    let json_str = serde_json::to_string_pretty(&config)?;
    ssh.write_file(&json_str, "/etc/sing-box/config.json").await?;
    ok();

    // ── 6. Enable + start sing-box ───────────────────────────────────────────
    step("Enabling and starting sing-box");
    ssh.exec("systemctl enable sing-box && systemctl restart sing-box").await?;
    ok();

    // ── 7. BBR + sysctl tuning ───────────────────────────────────────────────
    step("Enabling BBR + network tuning");
    ssh.exec(SYSCTL_SCRIPT).await?;
    ok();

    // ── 8. Firewall ──────────────────────────────────────────────────────────
    step("Configuring firewall");
    let fw = firewall_cmds(args.ssh_port, args.vless_port, args.hy2_port, args.new_ssh_port);
    ssh.exec(&fw).await?;
    ok();

    // ── 9. fail2ban ──────────────────────────────────────────────────────────
    step("Installing fail2ban");
    ssh.exec(
        "apt-get install -y fail2ban -qq && systemctl enable fail2ban && \
         systemctl start fail2ban"
    ).await?;
    ok();

    // ── 10. Health-check cron ────────────────────────────────────────────────
    step("Installing health-check cron");
    let cron_script = health_cron_script(args.vless_port, args.hy2_port);
    ssh.write_file(&cron_script, "/usr/local/bin/godo-healthcheck.sh").await?;
    ssh.exec(
        "chmod +x /usr/local/bin/godo-healthcheck.sh && \
         (crontab -l 2>/dev/null; \
          echo '*/5 * * * * /usr/local/bin/godo-healthcheck.sh >> /var/log/godo-health.log 2>&1') \
         | sort -u | crontab -"
    ).await?;
    ok();

    // ── 11. SSH hardening ────────────────────────────────────────────────────
    if let Some(new_port) = args.new_ssh_port {
        step(&format!("Hardening SSH → changing port to {new_port}"));
        ssh.exec(&format!(
            "sed -i 's/^#\\?Port .*/Port {new_port}/' /etc/ssh/sshd_config && \
             sed -i 's/^#\\?PermitRootLogin .*/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config && \
             sed -i 's/^#\\?PasswordAuthentication .*/PasswordAuthentication no/' /etc/ssh/sshd_config && \
             systemctl restart sshd"
        )).await?;
        ok();
        println!("  {} SSH is now on port {new_port}", "⚠".yellow());
    }

    // ── Done ─────────────────────────────────────────────────────────────────
    println!("\n{}", "═══ Bootstrap complete ═══".green().bold());
    println!("\n{}", "Save these to your .env:".yellow().bold());
    println!("VPS_HOST={}", args.host);
    println!("VPS_SSH_PORT={}", args.new_ssh_port.unwrap_or(args.ssh_port));
    println!("SINGBOX_VLESS_PORT={}", args.vless_port);
    println!("SINGBOX_HY2_PORT={}", args.hy2_port);
    println!("REALITY_SNI={}", args.sni);
    println!("VLESS_UUID={uuid}");
    println!("HY2_PASSWORD={hy2_pass}");
    println!("REALITY_PRIVATE_KEY={}", kp.private_key);
    println!("REALITY_PUBLIC_KEY={}", kp.public_key);
    println!("REALITY_SHORT_ID={}", kp.short_id);

    println!(
        "\n{}\n  godo-vps sub --host {} --uuid {} --hy2-password {} \\\n    \
         --reality-public-key {} --reality-short-id {}",
        "Get subscription links:".cyan(),
        args.host, uuid, hy2_pass, kp.public_key, kp.short_id
    );

    Ok(())
}

fn step(msg: &str) {
    print!("  {msg} … ");
    use std::io::Write;
    std::io::stdout().flush().ok();
}

fn ok() {
    println!("{}", "✓".green());
}

fn firewall_cmds(ssh_port: u16, vless_port: u16, hy2_port: u16, new_ssh: Option<u16>) -> String {
    let mut cmds = vec![
        "apt-get install -y ufw -qq".to_string(),
        "ufw --force reset".to_string(),
        "ufw default deny incoming".to_string(),
        "ufw default allow outgoing".to_string(),
        format!("ufw allow {ssh_port}/tcp comment 'SSH'"),
        format!("ufw allow {vless_port}/tcp comment 'VLESS-Reality'"),
        format!("ufw allow {hy2_port}/udp comment 'Hysteria2'"),
        format!("ufw allow {hy2_port}/tcp comment 'Hysteria2-tcp'"),
    ];
    if let Some(p) = new_ssh {
        cmds.push(format!("ufw allow {p}/tcp comment 'SSH-new'"));
    }
    cmds.push("ufw --force enable".to_string());
    cmds.join(" && ")
}

fn health_cron_script(vless_port: u16, hy2_port: u16) -> String {
    format!(
        r#"#!/bin/bash
set -euo pipefail

VLESS_PORT={vless_port}
HY2_PORT={hy2_port}
LOG_TAG="godo-health"

check_port() {{
    nc -z -w5 127.0.0.1 "$1" 2>/dev/null
}}

if ! check_port "$VLESS_PORT"; then
    logger -t "$LOG_TAG" "VLESS port $VLESS_PORT unreachable — restarting sing-box"
    systemctl restart sing-box
    sleep 5
fi

if ! check_port "$HY2_PORT"; then
    logger -t "$LOG_TAG" "Hysteria2 port $HY2_PORT unreachable — restarting sing-box"
    systemctl restart sing-box
fi

logger -t "$LOG_TAG" "health OK — vless:{vless_port} hy2:{hy2_port}"
"#
    )
}

const SYSCTL_SCRIPT: &str = r#"
# BBR
modprobe tcp_bbr 2>/dev/null || true
echo tcp_bbr > /proc/sys/net/ipv4/tcp_congestion_control || true

cat >> /etc/sysctl.conf << 'SYSCTL'
net.core.default_qdisc = fq
net.ipv4.tcp_congestion_control = bbr
net.core.rmem_max = 67108864
net.core.wmem_max = 67108864
net.ipv4.tcp_rmem = 4096 87380 67108864
net.ipv4.tcp_wmem = 4096 65536 67108864
net.core.netdev_max_backlog = 30000
net.ipv4.tcp_fastopen = 3
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 8192
net.ipv4.ip_local_port_range = 10000 65000
# UDP buffers for Hysteria2
net.core.rmem_default = 26214400
net.core.wmem_default = 26214400
SYSCTL

sysctl -p 2>/dev/null || true
"#;
