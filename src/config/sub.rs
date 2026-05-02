use crate::cli::SubArgs;
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use colored::Colorize;

pub async fn run(args: SubArgs) -> Result<()> {
    let vless_link = vless_reality_link(
        &args.host, args.vless_port, &args.uuid,
        &args.sni, &args.reality_public_key, &args.reality_short_id,
    );

    let hy2_link = hysteria2_link(&args.host, args.hy2_port, &args.hy2_password);

    // Base64-encoded multi-link subscription (v2ray/Shadowrocket format)
    let multi = format!("{vless_link}\n{hy2_link}");
    let sub_b64 = STANDARD.encode(multi.as_bytes());

    println!("{}", "─── Individual links ───".cyan().bold());
    println!("{}", "VLESS+Reality:".yellow());
    println!("{vless_link}\n");
    println!("{}", "Hysteria2:".yellow());
    println!("{hy2_link}\n");

    println!("{}", "─── Subscription (base64, paste into Shadowrocket / v2rayN / NekoBox) ───".cyan().bold());
    println!("{sub_b64}");

    println!("\n{}", "─── Client import tips ───".cyan().bold());
    println!("• Clash Verge Rev  → Profiles → New → Paste YAML from gen-client-config");
    println!("• Shadowrocket     → Add → Type=Subscribe → paste the base64 line above");
    println!("• v2rayN           → Server → Import from clipboard (paste vless:// or hy2://)");
    println!("• FlClash/NekoBox  → Profile → New → paste base64 subscription");
    println!("• Karing (iOS)     → Add → Remote → paste base64 subscription");

    Ok(())
}

fn vless_reality_link(
    host: &str, port: u16, uuid: &str,
    sni: &str, pub_key: &str, short_id: &str,
) -> String {
    format!(
        "vless://{uuid}@{host}:{port}?\
         encryption=none\
         &flow=xtls-rprx-vision\
         &security=reality\
         &sni={sni}\
         &fp=chrome\
         &pbk={pub_key}\
         &sid={short_id}\
         &type=tcp\
         &headerType=none\
         #{host}-Reality"
    )
}

fn hysteria2_link(host: &str, port: u16, password: &str) -> String {
    format!(
        "hysteria2://{password}@{host}:{port}?\
         insecure=1\
         &sni={host}\
         #{host}-Hy2"
    )
}
