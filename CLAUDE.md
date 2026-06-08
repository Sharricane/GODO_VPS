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
