use crate::{cli::StatusArgs, monitor::health};
use anyhow::Result;
use colored::Colorize;

pub async fn run(args: StatusArgs) -> Result<()> {
    println!("Checking {} …", args.host.cyan());

    let h = health::check(
        &args.host, args.vless_port, args.hy2_port,
        args.ssh_port, &args.user, args.ssh_key.as_deref(),
    ).await;
    println!("{}", h.summary());

    if h.is_healthy() {
        println!("{}", "Node is healthy.".green().bold());
    } else {
        println!("{}", "Node has issues!".red().bold());
        std::process::exit(1);
    }

    Ok(())
}
