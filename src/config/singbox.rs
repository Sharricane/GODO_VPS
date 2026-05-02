use crate::{
    cli::ServerConfigArgs,
    config::keys::{
        generate_reality_keypair, generate_short_id, generate_uuid,
        generate_hy2_password, or_generate, public_from_private,
    },
    deploy::ssh::Ssh,
};
use anyhow::Result;
use colored::Colorize;
use serde_json::{json, Value};

pub async fn run(args: ServerConfigArgs) -> Result<()> {
    let (uuid, uuid_new) = or_generate(args.uuid, generate_uuid);
    let (hy2_pass, hy2_new) = or_generate(args.hy2_password, generate_hy2_password);

    let (priv_key, pub_key, short_id, keys_new) = match args.reality_private_key {
        Some(ref p) if !p.is_empty() => {
            let pub_k = match args.reality_public_key {
                Some(ref k) if !k.is_empty() => k.clone(),
                _ => public_from_private(p)?,
            };
            let sid = args.reality_short_id
                .filter(|s| !s.is_empty())
                .unwrap_or_else(generate_short_id);
            (p.clone(), pub_k, sid, false)
        }
        _ => {
            let kp = generate_reality_keypair();
            let sid = kp.short_id.clone();
            (kp.private_key, kp.public_key, sid, true)
        }
    };

    let config = build_config(
        &uuid, &hy2_pass, &priv_key, &short_id,
        &args.sni, args.vless_port, args.hy2_port,
    );

    let json_str = serde_json::to_string_pretty(&config)?;

    println!("{}", "─── sing-box server config ───".cyan().bold());
    println!("{json_str}");

    if uuid_new || hy2_new || keys_new {
        println!("\n{}", "─── save these to your .env ───".yellow().bold());
        if uuid_new     { println!("VLESS_UUID={uuid}"); }
        if hy2_new      { println!("HY2_PASSWORD={hy2_pass}"); }
        if keys_new {
            println!("REALITY_PRIVATE_KEY={priv_key}");
            println!("REALITY_PUBLIC_KEY={pub_key}");
            println!("REALITY_SHORT_ID={short_id}");
        }
    }

    // Upload to VPS
    let ssh = Ssh::new(&args.host, args.ssh_port, &args.user, args.ssh_key.as_deref());
    ssh.write_file(&json_str, "/etc/sing-box/config.json").await?;
    ssh.exec("systemctl restart sing-box").await?;
    println!("{}", "\n✓ config uploaded and sing-box restarted".green());

    Ok(())
}

pub fn build_server_config(
    uuid: &str, hy2_pass: &str, reality_priv: &str, short_id: &str,
    sni: &str, vless_port: u16, hy2_port: u16,
) -> serde_json::Value {
    build_config(uuid, hy2_pass, reality_priv, short_id, sni, vless_port, hy2_port)
}

fn build_config(
    uuid: &str, hy2_pass: &str, reality_priv: &str, short_id: &str,
    sni: &str, vless_port: u16, hy2_port: u16,
) -> Value {
    json!({
        "log": { "level": "info", "timestamp": true },

        "inbounds": [
            {
                "type": "vless",
                "tag": "vless-in",
                "listen": "::",
                "listen_port": vless_port,
                "users": [{ "uuid": uuid, "flow": "xtls-rprx-vision" }],
                "tls": {
                    "enabled": true,
                    "server_name": sni,
                    "reality": {
                        "enabled": true,
                        "handshake": { "server": sni, "server_port": 443 },
                        "private_key": reality_priv,
                        "short_id": [short_id]
                    }
                }
            },
            {
                "type": "hysteria2",
                "tag": "hy2-in",
                "listen": "::",
                "listen_port": hy2_port,
                "users": [{ "password": hy2_pass }],
                "tls": {
                    "enabled": true,
                    "certificate_path": "/etc/sing-box/hy2.crt",
                    "key_path": "/etc/sing-box/hy2.key"
                }
            }
        ],

        "outbounds": [
            { "type": "direct", "tag": "direct" },
            { "type": "block",  "tag": "block" }
        ],

        "route": {
            "rules": [
                { "inbound": ["vless-in", "hy2-in"], "outbound": "direct" }
            ]
        }
    })
}
