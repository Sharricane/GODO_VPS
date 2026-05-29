use crate::{
    cli::MonitorArgs,
    monitor::{health, notify::Notifier},
};
use anyhow::Result;
use colored::Colorize;

pub async fn run(args: MonitorArgs) -> Result<()> {
    let notifier = Notifier::new(args.tg_token.as_deref(), args.tg_chat_id.as_deref());
    let interval = std::time::Duration::from_secs(args.interval);
    let mut was_down = false;

    println!(
        "{} monitoring {} every {}s — Ctrl+C to stop",
        "●".green(),
        args.host.cyan(),
        args.interval,
    );

    loop {
        let h = health::check(
            &args.host, args.vless_port, args.hy2_port,
            args.ssh_port, &args.user, args.ssh_key.as_deref(),
        ).await;

        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{ts}] {}", h.summary());

        if !h.is_healthy() {
            if !was_down {
                was_down = true;
                notifier.alert_down(&args.host, &h.summary()).await;

                if args.allow_restart {
                    tracing::warn!("auto-restarting sing-box on {} (kicks all clients)", args.host);
                    let _ = health::restart_singbox(
                        &args.host, args.ssh_port, &args.user, args.ssh_key.as_deref(),
                    ).await;
                } else {
                    tracing::info!("auto-restart disabled (pass --allow-restart to enable)");
                }
            }
        } else if was_down {
            was_down = false;
            notifier.alert_recovered(&args.host).await;
        }

        tokio::time::sleep(interval).await;
    }
}
