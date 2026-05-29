use crate::{cli::DiagArgs, deploy::ssh::Ssh};
use anyhow::Result;
use colored::Colorize;

pub async fn run(args: DiagArgs) -> Result<()> {
    println!("{}", format!("=== diag report for {} ===", args.host).cyan().bold());
    let ssh = Ssh::new(&args.host, args.ssh_port, &args.user, args.ssh_key.as_deref());

    print_section("sing-box service", &ssh,
        "systemctl show sing-box -p NRestarts,ActiveEnterTimestamp,ExecMainStartTimestamp,MainPID --no-pager").await;

    print_section("network stack",
        &ssh, &format!(
            "cat /proc/sys/net/ipv4/tcp_congestion_control; \
             cat /proc/sys/net/core/default_qdisc; \
             echo rmem_max=$(cat /proc/sys/net/core/rmem_max); \
             echo wmem_max=$(cat /proc/sys/net/core/wmem_max); \
             echo conntrack=$(cat /proc/sys/net/netfilter/nf_conntrack_count)/$(cat /proc/sys/net/netfilter/nf_conntrack_max)")).await;

    print_section("TCP retransmit / loss counters (cumulative since boot)", &ssh,
        "grep -A1 ^TcpExt: /proc/net/netstat | awk '\
            NR==1{for(i=1;i<=NF;i++) h[i]=$i} \
            NR==2{for(i=1;i<=NF;i++) if(h[i]~/Retrans|Loss|Drop|OFO|SynRetrans/) printf \"%-25s %s\\n\", h[i], $i}'").await;

    print_section("UDP error counters", &ssh,
        "grep -A1 ^Udp: /proc/net/snmp").await;

    print_section("active connections", &ssh,
        "echo \"tcp/443  established: $(ss -tn state established '( sport = :443 )' 2>/dev/null | tail -n+2 | wc -l)\"; \
         echo \"udp/8443 listener:    $(ss -uln '( sport = :8443 )' 2>/dev/null | tail -n+2 | wc -l)\"; \
         echo \"ss -s summary:\"; ss -s | head -10").await;

    let since = format!("{}h ago", args.since_hours);
    print_section(&format!("sing-box errors (last {since})"), &ssh,
        &format!("sudo -n journalctl -u sing-box --since '{since}' --no-pager 2>/dev/null | \
                  grep -i ERROR | wc -l | xargs -I{{}} echo 'total ERROR lines: {{}}'; \
                  sudo -n journalctl -u sing-box --since '{since}' --no-pager 2>/dev/null | \
                  grep -i ERROR | tail -15")).await;

    print_section("top scanner IPs (invalid REALITY handshakes — local 127.0.0.1 excluded)", &ssh,
        &format!("sudo -n journalctl -u sing-box --since '{since}' --no-pager 2>/dev/null | \
                  grep 'REALITY: processed invalid' | \
                  grep -oE 'from [0-9.]+' | awk '{{print $2}}' | grep -v '^127\\.' | \
                  sort | uniq -c | sort -rn | head -10")).await;

    println!("{}", "=== end diag ===".cyan().bold());
    Ok(())
}

async fn print_section(title: &str, ssh: &Ssh, cmd: &str) {
    println!("\n{}", format!("─── {title} ───").yellow());
    match ssh.exec(cmd).await {
        Ok(out) => {
            let out = out.trim_end();
            if out.is_empty() { println!("(no output)"); } else { println!("{out}"); }
        }
        Err(e) => println!("{} {e}", "ERR".red()),
    }
}
