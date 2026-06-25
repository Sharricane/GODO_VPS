# GODO_VPS — Claude Code 项目上下文

## 项目目标

在 **GigsGigsCloud SimpleCloud K+ 新加坡** VPS 上搭建个人代理服务器，满足：
- 内地、香港、海外三地均可顺畅使用
- 支持回国路由（海外→国内内容）
- iOS / Android / Windows / macOS 全平台客户端
- 同时支持至少 4 台电脑 + 5 台手机

## 技术栈

| 层 | 选型 | 说明 |
|---|---|---|
| 服务端内核 | sing-box | 支持 Reality + Hysteria2 |
| 部署方式 | fscarmen/sing-box 一键脚本 + 本项目 godo-vps CLI | |
| 主协议 | VLESS + Reality + Vision | 抗封锁，443 端口 |
| 备用协议 | Hysteria2 | UDP，深圳高峰期更快 |
| 桌面客户端 | Clash Verge Rev | Win/Mac，mihomo 内核 |
| iOS 客户端 | Shadowrocket | 需非中区 Apple ID |
| Android 客户端 | v2rayNG | |
| 规则集 | Loyalsoldier/clash-rules | 每日更新，内地/香港自动分流 |
| 管理工具 | godo-vps（本项目 Rust CLI） | 编译：cargo build --release |

## VPS 信息（购买后填写）

```
提供商：GigsGigsCloud SimpleCloud K+
地区：新加坡
IP：（待填写）
SSH 端口：（硬化后更新）
非 root 用户名：（Phase 3 创建后填写）
购买日期：
续费日期：
控制面板：
```

## 安全规范（不可绕过）

1. **SSH 仅允许密钥认证**，禁止密码登录
2. **非 root 用户**运行日常操作（Phase 3 创建 sudo 用户后切换）
3. **ufw 防火墙**仅开放 SSH、VLESS、Hysteria2 三个端口
4. **fail2ban** 防暴力破解
5. **BBR** 拥塞控制必须开启
6. **所有凭据**存放在 `.env`（已 gitignore），绝不提交到 GitHub

## 操作规范

- **执行破坏性操作前必须确认**（删文件、重置配置、reboot 等）
- **修改服务端配置前先备份**：`cp /etc/sing-box/config.json /etc/sing-box/config.json.bak`
- **与用户沟通用中文**
- **提交前运行** `git status` 确认 `.env` 不在暂存区
- **5+ 设备在用，绝不重启 sing-box / nginx**（细节见 memory/feedback_no_prod_disruption.md）
- **改 YAML 前先 grep 真实代理组名**（曾因 `🚀 Proxy` 误填导致整套配置炸；组名以文件里 `proxy-groups` 段为准）
- **改 YAML 后必须用 python3 -c 'yaml.safe_load' 验证**，再检查 rules[*] 引用的 group 是否存在
- **需要换协议栈但又不能重启 sing-box 时**：起独立进程在另一个端口（如 xray :10087），nginx upstream `sed` 改端口 + `nginx -s reload`。nginx 老 worker 自然 drain 旧连接，新 worker 接管新连接，**用户零感知**（已用于修 multiplex 的 broken pipe 问题）

## 客户端核心分类（决定能用什么 YAML）

| 客户端 | 核心 | 支持协议 | YAML 必须避开 |
|---|---|---|---|
| ClashX Pro (Mac) | legacy Clash | VMess / SS / Trojan | VLESS, Hysteria2, RULE-SET 用部分版本不识别 |
| Clash for Windows | legacy Clash | 同上 | 同上 |
| Clash for Android (原版) | legacy Clash | 同上 | 同上 |
| Clash Verge Rev | mihomo | VLESS, Reality, Hysteria2, ws-opts, sniffer | — |
| FlClash / ClashMetaForAndroid | mihomo | 同上 | — |
| ClashMi (iOS) | mihomo | 同上，但 iOS extension 沙箱限制 | `dns.listen`, `external-controller`, `bind-address`, `allow-lan` 会被静默拒绝 |

**legacy 客户端导入 VLESS proxy 会报 `unsupport type vless` 并整套配置加载失败。** 给 legacy 客户端的 YAML 只能有 VMess 节点（CFW 变种就是为此而生）。

## 已知坑

- **Loyalsoldier `direct.txt` 包含 Google 子域名**（`clientservices.googleapis.com`, `adservice.google.com`, `dl.google.com` 等）。任何用 `RULE-SET,direct,DIRECT` 的配置必须在它之前放显式 `DOMAIN-SUFFIX,google.com,Proxy` 等规则，否则 Google 搜索在 HK / CN 看起来加载了但实际部分服务挂掉（无补全、无图、无搜索结果）。已在 4 个 builder 全部修复。
- **fake-ip 模式下 `GEOIP,CN,DIRECT,no-resolve` 失效**——fake IP 不在 CN 段。要用 `GEOSITE,CN,DIRECT` 或去掉 `no-resolve`。
- **iOS Claude / ChatGPT app 用 HTTP/3 (QUIC over UDP 443)**，TUN 模式必须开 `sniffer.sniff.QUIC` 否则流量绕过代理，或加 `AND,((NETWORK,UDP),(DST-PORT,443)),REJECT` 强制 TCP 回退。
- **Reality / Hysteria2 在中国移动/联通 DPI 下被限速到 ~2000ms**——CDN-VMess 通道为此而生（经 Cloudflare 伪装成正常 HTTPS 网站访问）。
- **sing-box VMess inbound 的 `multiplex.enabled: true` 会让 Claude SSE 流式响应间歇 broken pipe**（legacy Clash 客户端 + mihomo 都中招，3-5% 概率）。当前修法：**xray 作为 parallel VMess inbound 跑在 127.0.0.1:10087，nginx upstream 切过去**。sing-box 的 :10086 留着不动当冷备，等下次自然重启再清理 multiplex 行。
- **xray + sing-box 共存**：不是替换，是平行。VLESS+Reality（:443 TCP）和 Hysteria2（:443 / :8443 UDP）继续在 sing-box；VMess 入站换 xray。两个进程互不干扰，systemd 各自管理。
- **iOS 必须用 redir-host DNS 模式，不能用 fake-ip**（mihomo / Clash Mi）。fake-ip 必须配 sniffer.parse-pure-ip + override-destination + force-dns-mapping 才能恢复目标域名，但**这三个选项会在 TLS 握手中途改写 destination，Anthropic 移动端 API 严格 mTLS 校验**直接拒，iOS Claude app 卡 splash 报 "something went wrong"。redir-host 模式 DNS 返回真实 IP，mihomo 从 DNS 缓存反查域名做路由，TLS 字节流原封不动透传 → Anthropic mTLS 不会出问题。**rules 的 `GEOIP,CN,DIRECT,no-resolve` 失效坑只存在于 fake-ip 模式，redir-host 没有这个问题。** 5月30号 commit e9d5f0b 用的就是 redir-host，能用；5月31号 commit 5f71f1c 改成 fake-ip + 激进 sniffer 后炸了 iOS Claude app，几周后才查到。**iOS yaml 永远 redir-host。**
- **Anthropic 移动端 API 对 datacenter IP 软拒**：a-api.anthropic.com 的 TCP / TLS 握手对 GigsGigsCloud Singapore IP 全部成功，但应用层第一个请求 Anthropic 返回 app 无法接受的响应（疑似设备 attestation 校验失败 / WAF challenge），iOS / Android Claude app 都卡 splash。修法：**装 xray-warp.service 作为 xray-with-WireGuard-WARP-outbound，nginx upstream 切到它，所有 anthropic / claude 域名 routing 规则指向 warp 出站**。出口 IP 变成 Cloudflare 内部 IP，Anthropic 看到 CF 流量后不再 soft-reject。Mac Claude Code 不受影响（同一条链路，WARP 也接受 Mac 的 API key 流量）。
- **xray 不支持 SIGHUP 配置热重载**——和 sing-box 一样。要改 xray 的 outbound / routing，**起新的 xray-warp.service 跑在另一个端口，nginx upstream 切上游 + `nginx -s reload` 优雅切换**，老 xray 维持现有连接到自然 close，零客户端断流。这套模式（"平行新服务 + nginx 上游切换"）现在 VPS 上已用了两次：sing-box VMess :10086 → xray :10087（删 multiplex），xray :10087 → xray-warp :10088（加 WARP 出站）。

## 服务清单（VPS 当前实际运行）

| 服务 | 端口 / 路径 | 用途 |
|---|---|---|
| sing-box.service | `:443 TCP` | VLESS+Reality（iOS / mihomo 客户端直连快速通道） |
| sing-box.service | `:443 UDP`, `:8443 UDP` | Hysteria2 |
| sing-box.service | `127.0.0.1:10086` | VMess（**冷备**，nginx 已不再路由到此） |
| xray.service | `127.0.0.1:10087` | VMess WS `/vmws`（**冷备**，nginx 已不再路由到此，被 xray-warp 取代） |
| xray-warp.service | `127.0.0.1:10088` | VMess WS `/vmws`（**主用**），anthropic 域名走 WireGuard 到 Cloudflare WARP 出站；其他域名走 freedom direct 出站 |
| nginx.service | `:80` | 静态订阅 yaml + `/vmws` 反代到 `127.0.0.1:10088` |

## 本地工具

```bash
# 编译
source ~/.cargo/env && cargo build --release

# 生成 Reality 密钥对
./target/release/godo-vps gen-server-config --host $VPS_HOST

# 生成 Clash 客户端配置
./target/release/godo-vps gen-client-config -o ~/clash-config.yaml

# 健康检查
./target/release/godo-vps status --host $VPS_HOST

# 持续监控（含 Telegram 告警）
./target/release/godo-vps monitor --host $VPS_HOST
```

## 七阶段配置清单

```
[ ] Phase 1: 初次登录与基础配置
      - 以 root SSH 登入
      - 修改 root 密码
      - 确认系统版本（目标：Debian 12）
      - 设置时区（Asia/Singapore）
      - apt update && apt upgrade

[ ] Phase 2: 服务器质量测试
      - 测试到中国大陆延迟（ping）
      - 测试到香港延迟
      - 线路质量评估（itdog.cn / ping.chinaz.com）
      - 检查 IP 是否被墙（使用 ping0.cc）

[ ] Phase 3: 安全加固
      - 创建非 root sudo 用户
      - 配置 SSH 密钥认证，禁用密码登录
      - 修改 SSH 端口（更新 .env 中的 VPS_SSH_PORT）
      - 安装配置 fail2ban
      - 配置 ufw 防火墙

[ ] Phase 4: BBR 启用
      - 启用 BBR 拥塞控制
      - 调整 sysctl 网络参数（TCP + UDP 缓冲区）
      - 验证 BBR 生效

[ ] Phase 5: sing-box 安装（Reality + Hysteria2）
      - 运行 fscarmen/sing-box 一键脚本
      - 选择 VLESS+Reality+Vision + Hysteria2 组合
      - 生成并保存密钥到 .env
      - 验证服务启动正常
      - 配置健康检查 cron

[ ] Phase 6: 客户端配置
      - 运行 godo-vps gen-client-config 生成 Clash 配置
      - 运行 godo-vps sub 获取订阅链接
      - Clash Verge Rev 导入配置（Windows/Mac）
      - Shadowrocket 导入订阅（iOS）
      - v2rayNG 导入订阅（Android）
      - 测试内地/香港/海外访问

[ ] Phase 7: 验证与监控
      - 运行 godo-vps status 确认节点健康
      - 配置 Telegram Bot 告警
      - 启动 godo-vps monitor 守护进程
      - 测试自动重启功能
      - 记录最终配置到本文件 VPS 信息区
```

## 依赖项目（复用，不重写）

- `fscarmen/sing-box` — 服务端一键部署脚本
- `Loyalsoldier/clash-rules` — 分流规则集
- `clash-verge-rev/clash-verge-rev` — 桌面客户端
- `MetaCubeX/mihomo` — Clash.Meta 内核
