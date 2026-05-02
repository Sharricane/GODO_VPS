# GODO_VPS

基于 Rust 的新加坡 VPS 代理管理工具，支持 VLESS+Reality（主）+ Hysteria2（备），全平台客户端。

> ⚠️ **本仓库为 PUBLIC。`.env` 文件已 gitignore，绝对不能提交。**

## 快速开始

```bash
# 1. 克隆仓库
git clone https://github.com/Sharricane/GODO_VPS.git
cd GODO_VPS

# 2. 创建本地配置文件
cp .env.example .env

# 3. 填入真实值（购买 VPS 后）
#    编辑 .env，填写 VPS_HOST、SSH 密钥路径、协议凭据等

# 4. 编译管理工具
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
cargo build --release

# 5. 一键部署 VPS
./target/release/godo-vps bootstrap --host YOUR_VPS_IP

# 6. 生成客户端配置
./target/release/godo-vps gen-client-config -o ~/clash-config.yaml
./target/release/godo-vps sub
```

## ⚠️ 安全警告

| 文件 | 状态 | 说明 |
|---|---|---|
| `.env` | **已 gitignore** | 含真实凭据，绝不提交 |
| `.env.example` | 可提交 | 仅占位符，无真实值 |
| `CLAUDE.md` | 可提交 | 项目上下文，无凭据 |

**如果不小心提交了 `.env`：立即轮换所有凭据（UUID、密码、SSH 密钥），然后从 git 历史中删除。**

## 项目结构

```
GODO_VPS/
├── .env.example          # 配置模板（可提交）
├── .env                  # 真实配置（已 gitignore，不可提交）
├── .gitignore            # 覆盖凭据、密钥、构建产物、OS 垃圾
├── CLAUDE.md             # Claude Code 项目上下文 + 配置清单
├── Cargo.toml            # Rust 项目配置
├── src/
│   ├── main.rs
│   ├── cli.rs            # 命令行定义
│   ├── config/
│   │   ├── keys.rs       # X25519 密钥生成（Reality）
│   │   ├── singbox.rs    # sing-box 服务端配置生成
│   │   ├── clash.rs      # Clash 客户端配置生成
│   │   └── sub.rs        # 订阅链接生成
│   ├── deploy/
│   │   ├── ssh.rs        # SSH 操作封装
│   │   └── bootstrap.rs  # VPS 全套安装编排
│   └── monitor/
│       ├── health.rs     # 节点健康检查
│       ├── notify.rs     # Telegram 告警
│       ├── daemon.rs     # 持续监控
│       └── status.rs     # 一次性状态检查
└── target/release/
    └── godo-vps          # 编译后的二进制（已 gitignore）
```

## 子命令

```bash
godo-vps bootstrap          # 全套 VPS 初始化（sing-box + BBR + 防火墙）
godo-vps gen-server-config  # 生成并上传 sing-box 服务端配置
godo-vps gen-client-config  # 生成 Clash 客户端 YAML
godo-vps sub                # 输出订阅链接（vless:// + hy2://）
godo-vps status             # 节点存活检查
godo-vps monitor            # 持续监控 + Telegram 告警
```

## 客户端

| 平台 | 推荐客户端 | 说明 |
|---|---|---|
| Windows / macOS | Clash Verge Rev | 导入 `gen-client-config` 生成的 YAML |
| iOS | Shadowrocket | 需非中区 Apple ID |
| Android | v2rayNG | 导入 `sub` 输出的订阅链接 |

## 技术栈

- **服务端**：sing-box（VLESS+Reality + Hysteria2）
- **规则集**：Loyalsoldier/clash-rules（每日更新，内地/香港自动分流）
- **管理工具**：Rust（稳定性优先，单二进制无依赖）
- **VPS**：GigsGigsCloud SimpleCloud K+ 新加坡
