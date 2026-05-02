# GODO VPS

A Rust CLI for deploying and managing a personal proxy server on a Singapore VPS.

**Stack:** sing-box · VLESS+Reality · Hysteria2 · Clash · Loyalsoldier rules

---

## Quickstart

```bash
git clone https://github.com/Sharricane/GODO_VPS.git && cd GODO_VPS
cp .env.example .env        # fill in VPS credentials
cargo build --release
./target/release/godo-vps bootstrap   # provision VPS end-to-end
./target/release/godo-vps sub         # get client subscription links
```

> **Never commit `.env`** — it is gitignored and contains real credentials.

## Commands

| Command | Description |
|---|---|
| `bootstrap` | Full VPS setup: sing-box, BBR, firewall, fail2ban |
| `gen-server-config` | Generate and upload sing-box server config |
| `gen-client-config` | Generate Clash YAML with CN/HK split routing |
| `sub` | Print subscription links (vless:// + hy2://) |
| `status` | One-shot node health check |
| `monitor` | Continuous monitoring with Telegram alerts |

## Clients

| Platform | App |
|---|---|
| Windows / macOS | [Clash Verge Rev](https://github.com/clash-verge-rev/clash-verge-rev) |
| iOS | Shadowrocket |
| Android | v2rayNG |

## License

MIT
