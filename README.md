# GODO VPS

A Rust CLI for deploying and operating a personal multi-protocol proxy server on
a Singapore VPS, with diagnostics and a Cloudflare CDN front for mainland-China
clients that get DPI-throttled on plain Reality/Hysteria2.

Stack: sing-box (VLESS-Reality, Hysteria2, VMess-WS) - nginx reverse proxy -
Cloudflare CDN - Clash / Mihomo clients.

---

## Why CDN-VMess exists

The original deployment used VLESS-Reality on TCP/443 and Hysteria2 on UDP/8443.
Both worked from Hong Kong networks but were throttled to ~2000 ms latency from
mainland China Unicom / Mobile because the ISP DPI identifies the Reality TLS
fingerprint after enough traffic. The IP itself was not blocked - TCP/443
handshake succeeded from every CN probe location - but the inner TLS data flow
was selectively rate-limited.

To bypass this, traffic now goes through Cloudflare:

    CN client -> CF edge (TLS) -> CF origin pull (HTTP) -> nginx :80 /vmws ->
    sing-box vmess-in 127.0.0.1:10086 -> outbound direct -> target

From the ISP perspective the traffic looks like a normal HTTPS request to a
Cloudflare-hosted website, which is indistinguishable from any other CDN flow.
Mainland latency dropped from ~2000 ms to ~400 ms after the switch.

The legacy VLESS-Reality and Hysteria2 inbounds are retained as fallbacks for
clients on networks where the CDN edge is slower (HK, overseas).

---

## Quickstart

    git clone https://github.com/Sharricane/GODO_VPS.git && cd GODO_VPS
    cp .env.example .env        # fill in VPS host, SSH key, credentials
    cargo build --release
    ./target/release/godo-vps bootstrap     # provision a fresh VPS
    ./target/release/godo-vps sub           # print client subscription links

Never commit `.env` - it is gitignored and contains real credentials.

---

## Subscription URLs

Each device type pulls its own YAML; the server hosts them all from
`/var/www/sub/` via nginx :80.

| Device          | URL path                  | Variant            |
|-----------------|---------------------------|--------------------|
| iOS (ClashMi)   | `/ios-clash-mi.yaml`      | slim, fake-ip+sniffer, 3 rule-providers (50 MB cap) |
| Android (CMFA)  | `/android-clash-vmess.yaml` | full                                              |
| macOS (ClashX)  | `/mac-clashx-pro.yaml`    | full                                              |
| Windows (Verge) | `/windows-clash-vless.yaml` | full                                            |
| Generic         | `/<hash>.yaml`            | full                                              |

The iOS variant strips fields the iOS VPN extension rejects (`dns.listen`,
`external-controller`, `bind-address`, `allow-lan`) and trims the rule-providers
to three to stay under the 50 MB extension memory cap. It enables `fake-ip` with
a `sniffer` block so native iOS apps that use HTTP/3 (Claude, ChatGPT, browsers)
still get matched by domain rules instead of leaking past as raw UDP.

---

## Commands

| Command              | Description |
|----------------------|-------------|
| `bootstrap`          | Full VPS setup: sing-box, BBR, ufw, fail2ban, SSH hardening |
| `gen-server-config`  | Generate and upload sing-box server config |
| `gen-client-config`  | Generate Clash YAML; add `--ios` for the slim variant |
| `sub`                | Print subscription links (vless:// + hy2://) |
| `status`             | One-shot node health check |
| `monitor`            | Continuous health monitor with optional Telegram alerts and `--allow-restart` |
| `diag`               | Pull comprehensive read-only diagnostics: BBR, TCP retrans counters, UDP errors, scanner IPs, recent sing-box errors |

`diag` is the right tool when something is misbehaving. It does not touch the
running service and is what was used to discover the DPI throttling that
motivated the CDN-VMess work.

---

## Server-side topology

    Cloudflare zone xn--7xa.monster
      cdn.xn--7xa.monster  proxied (orange)  A -> VPS IP
      Configuration Rule: hostname=cdn.* -> SSL mode Flexible
        (CF terminates TLS; talks plain HTTP :80 to origin)

    VPS (Debian 12, GigsGigsCloud SimpleCloud K+ Singapore)
      nginx :80
        default_server  (sites-enabled/sub)
          /*.yaml         -> /var/www/sub/    (subscription delivery)
          /vmws           -> 127.0.0.1:10086  (vmess-in upstream)
          /               -> 404

      sing-box (NRestarts=0)
        :443 TCP   VLESS-Reality   (legacy direct path)
        :443 UDP   Hysteria2       (legacy direct path)
        :8443 UDP  Hysteria2       (legacy direct path)
        127.0.0.1:10086  vmess-in WS path /vmws  (CDN front pipeline)

      cron
        /etc/cron.d/godo-traffic     (every minute, passive byte sampler)
        /etc/cron.d/godo-bw-check    (every 30 min, vnstat quota alert)

Hysteria2 inbounds carry `up_mbps=200`, `down_mbps=30`, `ignore_client_bandwidth=false`
so BRUTAL congestion control actually engages. Without these the protocol
silently falls back to default ACK-pacing and gives up its main advantage on
lossy paths.

---

## Atomic config rotation

Server-side config changes go through:

    sudo /usr/local/bin/godo-singbox-rotate /path/to/new-config.json

The script validates the new config with `sing-box check`, backs the current
file up to `/etc/sing-box/backups/`, swaps atomically, restarts sing-box, and
auto-rolls-back if the 2-second post-restart probe (service active +
TCP/443 listener + UDP/8443 listener) fails. Use this rather than manual `cp`
plus `systemctl restart` so a bad config cannot take the node down.

---

## Clients

| Platform        | Recommended app                                                |
|-----------------|----------------------------------------------------------------|
| Windows         | Clash Verge Rev                                                |
| macOS           | ClashX Pro or Clash Verge Rev                                  |
| iOS             | ClashMi (use `/ios-clash-mi.yaml`, enable TUN/VPN in app settings) |
| Android         | ClashMetaForAndroid (enable TUN mode in app settings)          |

Group `Proxy` is a fallback group ordered `CDN-VMess -> SG-Hysteria2 ->
SG-Reality`. The client tries the CDN path first and only falls back to the
direct protocols if the CDN probe fails. On a mainland-China network you should
see CDN-VMess get selected.

---

## Configuration drift

The Cloudflare side is configured via API. Useful state:

| Resource                                | Identifier                              |
|-----------------------------------------|-----------------------------------------|
| Zone `xn--7xa.monster`                  | id `5f1c912d7bd68c037e419ebe9987e9c2`   |
| DNS A `cdn.xn--7xa.monster` (proxied)   | -> VPS IP                                |
| DNS A `origin.xn--7xa.monster` (DNS-only) | -> VPS IP, kept for future use         |
| Config Rule "godo-cdn-ssl-mode"         | host=cdn.* -> SSL Flexible              |

The CDN setup does not require Cloudflare Pro. SSL Flexible is scoped per-host
via Configuration Rule so the rest of the zone (apex GitHub Pages site, other
subdomains) keeps its existing SSL mode.

---

## Local memory and tooling

This repo carries a `CLAUDE.md` with project context for Claude Code sessions.
The investigation that produced the CDN-VMess pipeline, the iOS quirks, and
the BRUTAL hint requirement is documented there and in the local memory under
`~/.claude/projects/-home-miranda-project-GODO-VPS/memory/`.

---

## License

MIT
