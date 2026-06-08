# GODO VPS

A Rust CLI for deploying and operating a personal multi-protocol proxy server
on a Singapore VPS, with diagnostics and a Cloudflare CDN front for
mainland-China clients that get DPI-throttled on plain Reality / Hysteria2.

Stack: sing-box (VLESS-Reality, Hysteria2, VMess-WS) - nginx reverse proxy -
Cloudflare CDN - Clash / Mihomo clients (Windows / macOS / Android / iOS).

---

## Why CDN-VMess exists

The original deployment used VLESS-Reality on TCP/443 and Hysteria2 on
UDP/443+8443. Both worked from Hong Kong networks but were throttled to
~2000 ms latency from mainland-China Unicom / Mobile because the ISP DPI
identifies the Reality TLS fingerprint after enough traffic flows through it.
The IP itself was never blocked - TCP/443 handshake succeeded from every CN
probe location on `itdog.cn` - but the inner TLS data stream was selectively
rate-limited at the carrier level.

To bypass this, traffic now goes through Cloudflare:

    CN client --(TLS, client -> CF edge)--> CF
       |
       +--(HTTP, CF -> origin :80)--> nginx
                                        |
                                        +-- /vmws --> sing-box vmess-in
                                                         127.0.0.1:10086
                                                            |
                                                            +-- outbound direct --> target

From the ISP perspective the traffic looks like a normal HTTPS request to a
Cloudflare-hosted website - indistinguishable from any other CDN flow.
Mainland-CN latency on the proxy chain dropped from ~2000 ms to ~400 ms after
the switch.

The legacy VLESS-Reality and Hysteria2 inbounds are kept as fallbacks for
clients on networks where the CDN edge is slower (Hong Kong, overseas).

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
`/var/www/sub/` via nginx on port 80.

| Device          | URL path                       | Variant                                                  |
|-----------------|--------------------------------|----------------------------------------------------------|
| iOS (ClashMi)   | `/ios-clash-mi.yaml`           | minimal, default-Proxy with three CN apps DIRECT         |
| Android (CMFA)  | `/android-clash-vmess.yaml`    | full                                                     |
| macOS (ClashX)  | `/mac-clashx-pro.yaml`         | full                                                     |
| Windows (Verge) | `/windows-clash-vless.yaml`    | full (mihomo)                                            |
| Windows (CFW)   | `--cfw` output                 | CFW-legacy: Direct-VMess + CDN-VMess (no VLESS/Hy2)      |
| Generic         | `/<hash>.yaml`                 | full                                                     |

### The full variant

Used by Windows / macOS / Android. Contains everything: three proxy nodes
(SG-Reality, SG-Hysteria2, CDN-VMess), five proxy-groups (`Proxy`, `Select`,
`OpenAI`, `Streaming`, `Final`), nine rule-providers from Loyalsoldier, and a
sniffer block that recovers TLS-SNI / QUIC ClientHello / HTTP Host so HTTP/3
apps still match domain rules under fake-ip.

Group `Proxy`, `OpenAI`, and `Streaming` are all `type: fallback`. The main
variant orders them `CDN-VMess -> SG-Hysteria2 -> SG-Reality` (CDN first
because the typical client is on a CN ISP being DPI-throttled). The `--ios`
variant flips that ordering to `SG-Reality -> SG-Hysteria2 -> CDN-VMess`
because iOS users are usually on HK/overseas networks where the direct
Reality path is fastest and the CDN URL-test probe occasionally times out.

Every variant now pins Google domains (`google.com`, `googleapis.com`,
`gstatic.com`, `youtube.com`, `googleadservices.com`, etc.) to `Proxy`
**before** any RULE-SET match. Without this, Loyalsoldier's `direct.txt`
catches `clientservices.googleapis.com`, `adservice.google.com`,
`dl.google.com` and routes them DIRECT - the search page still loads but
SafeBrowsing / autocomplete / Ads / Maps APIs silently fail in HK and CN.

### The iOS variant (`--ios`)

iOS VPN extensions have hard sandbox constraints that desktop clients don't:

- `dns.listen`, `external-controller`, `bind-address`, `allow-lan`
  are silently rejected (the extension cannot bind sockets).
- The extension has a 50 MB RAM cap; loading the full Loyalsoldier rule
  set blows past it.
- ClashMi on certain iPhone models (issue
  [KaringX/clashmi#195](https://github.com/KaringX/clashmi/issues/195))
  has a rules-mode bug where rules-engine routing silently fails even though
  proxies and rule-providers load fine.

The iOS variant accommodates all of these:

- All fields that the extension rejects are stripped.
- `enhanced-mode: fake-ip` with a `sniffer` block (`parse-pure-ip`,
  `override-destination`, `force-dns-mapping` all enabled) so HTTP/3 apps
  (Claude, ChatGPT, Atlas browser) still hit the right rule.
- All rule-providers and GEOIP / RULE-SET rules removed - every routing
  decision is an inline DOMAIN-SUFFIX rule, which is the lightest match
  type for the buggy rules engine to handle.
- IPv4-only.
- Default-Proxy routing: everything goes through the proxy by default,
  with explicit DIRECT exemptions for three CN apps that the user needs
  to keep on CN routing (Bilibili, WeChat / Tencent IM, Xiaohongshu).
  Other CN domains (Taobao, Alipay, Sina, etc.) go through CDN-VMess.

If even the iOS variant doesn't work on a particularly affected iPhone
model, `--ios-minimal` exists for an even more aggressive strip-down
(single proxy node only, single proxy-group, ~85 inline rules covering
the major CN domains and major foreign services).

---

## Commands

| Command              | Description                                                                                                |
|----------------------|------------------------------------------------------------------------------------------------------------|
| `bootstrap`          | Full VPS setup: sing-box, BBR, ufw, fail2ban, SSH hardening                                                |
| `gen-server-config`  | Generate and upload sing-box server config                                                                 |
| `gen-client-config`  | Generate Clash YAML; add `--ios` for the iOS variant, `--ios-minimal` for the bug-workaround variant       |
| `sub`                | Print subscription links (vless:// + hy2://)                                                               |
| `status`             | One-shot node health check                                                                                 |
| `monitor`            | Continuous health monitor with optional Telegram alerts and `--allow-restart`                              |
| `diag`               | Pull comprehensive read-only diagnostics: BBR, TCP retrans counters, UDP errors, scanner IPs, recent errors |

`diag` is the right tool when something is misbehaving. It does not touch
the running service and is what was used to discover the DPI throttling that
motivated the CDN-VMess work.

---

## Server-side topology

    Cloudflare zone xn--7xa.monster
      cdn.xn--7xa.monster  proxied (orange)  A -> VPS IP
      Configuration Rule: hostname=cdn.* -> SSL mode Flexible
        (CF terminates TLS; talks plain HTTP :80 to origin)

    VPS (Debian, GigsGigsCloud SimpleCloud K+ Singapore)
      nginx :80
        default_server  (sites-enabled/sub)
          /*.yaml         -> /var/www/sub/    (subscription delivery)
          /vmws           -> 127.0.0.1:10086  (vmess-in upstream)
          /               -> 404

      sing-box (NRestarts = 0 since deploy)
        :443 TCP   VLESS-Reality        (legacy direct path)
        :443 UDP   Hysteria2            (legacy direct path)
        :8443 UDP  Hysteria2            (legacy direct path)
        127.0.0.1:10086  vmess-in WS path /vmws   (CDN front pipeline)

      cron
        /etc/cron.d/godo-traffic     (every minute, passive byte sampler)
        /etc/cron.d/godo-bw-check    (every 30 min, vnstat quota alert)

Hysteria2 inbounds carry `up_mbps=200`, `down_mbps=30`,
`ignore_client_bandwidth=false` so BRUTAL congestion control actually
engages. Without these the protocol silently falls back to default
ACK-pacing and gives up its main advantage on lossy paths.

---

## Atomic config rotation

Server-side config changes go through:

    sudo /usr/local/bin/godo-singbox-rotate /path/to/new-config.json

The script validates the new config with `sing-box check`, backs the current
file up to `/etc/sing-box/backups/`, swaps atomically, restarts sing-box,
and auto-rolls-back if the 2-second post-restart probe (service active +
TCP/443 listener + UDP/8443 listener) fails. Use this rather than manual
`cp` plus `systemctl restart` so a bad config cannot take the node down.

---

## Client setup

| Platform | App                              | Notes                                                                  |
|----------|----------------------------------|------------------------------------------------------------------------|
| Windows  | Clash Verge Rev                  | TUN mode in app settings                                               |
| macOS    | ClashX Pro / Clash Verge Rev     | TUN mode in app settings                                               |
| Android  | ClashMetaForAndroid              | TUN mode in app settings (required for full UDP/QUIC capture)          |
| iOS      | ClashMi                          | VPN profile must be approved; uses the `--ios` slim variant            |

When updating subscriptions, **delete the entire subscription entry and
re-add the URL** rather than clicking refresh. Clash clients sometimes
re-use the cached parsed config and a plain refresh does not pick up
structural changes (new proxy-groups, new rule-providers, sniffer block,
etc.).

After deleting and re-adding the subscription, kill the app process
completely (force-quit) before reopening - especially on iOS, where the
VPN extension caches the previous configuration in memory.

---

## Cloudflare configuration

The Cloudflare side is configured via API. Useful state:

| Resource                                  | Identifier                              |
|-------------------------------------------|-----------------------------------------|
| Zone `xn--7xa.monster`                    | id `5f1c912d7bd68c037e419ebe9987e9c2`   |
| DNS A `cdn.xn--7xa.monster` (proxied)     | -> VPS IP                                |
| DNS A `origin.xn--7xa.monster` (DNS-only) | -> VPS IP, kept for future use          |
| Config Rule `godo-cdn-ssl-mode`           | `host eq "cdn.xn--7xa.monster"` -> SSL Flexible |

Zone settings enabled (free plan, no Pro / Argo required):

- HTTP/3 on
- 0-RTT on
- TLS 1.3 on, `min_tls_version` 1.2
- WebSockets on
- Brotli on
- IPv6 on
- Always use HTTPS on
- Early hints on
- Automatic HTTPS rewrites on

The CDN setup does not require Cloudflare Pro. SSL Flexible is scoped
per-hostname via Configuration Rule so the rest of the zone (apex GitHub
Pages site, other subdomains) keeps its existing SSL mode.

---

## What can go wrong, and how to tell

| Symptom                                                            | Most likely cause                                                                                |
|--------------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| Latency stable at ~2000 ms from CN, drops when on HK Wi-Fi          | ISP DPI is throttling Reality / Hysteria2 - route through CDN-VMess                              |
| `claude.ai` returns 403 on raw curl from VPS                        | False alarm - curl lacks browser fingerprint; site does bot detection, not IP blocking          |
| `cdn.xn--7xa.monster` returns `ERR_CONNECTION_CLOSED`               | Wrong CF SSL mode - the per-host Config Rule must be set to `Flexible`, not zone-wide            |
| `cdn.xn--7xa.monster` times out from the VPS itself                 | CF anti-loop kicked in (origin IP == source IP) - test from a different vantage point            |
| iOS subscription imports, connection green, but apps fail           | ClashMi rules-mode bug - use the `--ios-minimal` variant or switch to global mode                |
| ChatGPT works but Claude doesn't on iOS                             | iOS Claude app uses HTTP/3; ensure `sniffer` block is loaded (check for unquoted port ranges)    |
| Atlas browser search fails                                          | Atlas uses `chatgpt.com` + `bing.com` as backends; both must have explicit DOMAIN-SUFFIX rules   |
| Mac / Windows DIRECT slow                                           | Stale subscription with old rule-providers - delete subscription entry, re-add URL, kill client  |
| Mac (ClashX Pro) or Android (legacy Clash) reports `unsupport type vless` | Client uses legacy Clash core (no VLESS / Hysteria2). Switch them to the `--cfw` (legacy) variant - VMess-only with Direct-VMess + CDN-VMess |
| Google search loads but autocomplete / images / Ads / Maps silently fail | Loyalsoldier `direct.txt` catches `clientservices.googleapis.com`, `adservice.google.com`, `dl.google.com`. The fix (explicit Google DOMAIN-SUFFIX rules at the top) ships in every variant; pull a fresh subscription if you see it |

---

## Local memory and tooling

This repo carries a `CLAUDE.md` with project context for Claude Code
sessions. The investigation that produced the CDN-VMess pipeline, the iOS
quirks, the BRUTAL hint requirement, and the per-platform variant strategy
is documented there and in the local memory under
`~/.claude/projects/-home-miranda-project-GODO-VPS/memory/`.

---

## License

MIT
