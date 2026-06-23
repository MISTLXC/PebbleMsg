# PebbleMsg

**基于邮件的桌面消息弹窗提醒工具。** 基于 Pebble 重构，使用 Rust + React 构建。接入任意邮箱，新邮件到达时桌面弹窗实时提醒，关键消息绝不会错过。

<p align="center">
  <img src="site/screenshots/main.png" alt="PebbleMsg 截图" width="800">
</p>

## 与 Pebble 的区别

PebbleMsg 在 Pebble 0.0.8 基础上做了以下增强：

- **消息弹窗** — 新邮件到达时桌面弹出通知窗口，可自定义窗口尺寸、显示内容和持续时间，不遗漏任何重要消息。
- **同步可靠性优化** — 重写了邮件同步策略，离线操作自动排队，恢复连接后重试，确保远端写入最终一致。
- 其余功能与 Pebble 保持一致。

## 功能特性

- **消息弹窗提醒** — 新邮件到达桌面弹窗，可自定义窗口大小、显示字段（发件人/主题/摘要/时间）、持续时长和屏幕位置，最多同时展示 5 个弹窗。
- **多邮箱接入** — 支持 Gmail、Outlook 及任意 IMAP 服务器，集中管理所有账户。
- **看板视图** — 拖拽邮件到待办、等待中、已完成，收件箱即任务板。
- **全文搜索** — 基于 Tantivy，按内容、发件人或日期快速定位。
- **延后提醒** — 延后邮件稍后处理，到时间重新提醒。
- **规则引擎** — 自定义条件自动标记、移动或归档邮件。
- **双语翻译** — 双语对照视图，支持 DeepL 或 LLM。
- **WebDAV 设置备份** — 规则、看板数据等备份到自建 WebDAV 服务器。

## 隐私保护

- 所有数据本地存储 — 邮件、搜索索引、附件均留存在你的设备。
- 无遥测，无第三方数据采集。
- 外发流量仅在你主动使用对应功能时发生：邮件同步、翻译、WebDAV 备份、外部图片加载或 GitHub 手动检查更新。

## 技术栈

| 层面 | 技术 |
|-------|------|
| 桌面框架 | [Tauri 2](https://tauri.app) |
| 后端 | Rust |
| 前端 | React 19 + TypeScript |
| 样式 | Tailwind CSS 4 |
| 搜索引擎 | Tantivy |
| 数据库 | SQLite |

## 开发指南

### 环境要求

- [Rust](https://rustup.rs) 最新稳定版
- [Node.js](https://nodejs.org) 18+
- [pnpm](https://pnpm.io) 9+
- [Tauri 系统依赖](https://tauri.app/start/prerequisites/)

### 快速开始

```bash
git clone https://github.com/QingJ01/Pebble.git
cd Pebble

cp .env.example .env
pnpm install
pnpm dev
```

### 构建

```bash
pnpm build          # 当前平台
pnpm build:windows  # Windows
pnpm build:macos    # macOS
pnpm build:linux    # Linux
```

## 项目结构

```
├── crates/            # Rust workspace
│   ├── pebble-core/   # 共享类型与接口
│   ├── pebble-crypto/ # AES-256 加密与密钥管理
│   ├── pebble-mail/   # IMAP/SMTP/Gmail/Outlook 邮件协议
│   ├── pebble-oauth/  # OAuth 2.0 + PKCE 认证
│   ├── pebble-privacy/# HTML 净化与追踪器拦截
│   ├── pebble-rules/  # 规则引擎
│   ├── pebble-search/ # 全文搜索
│   ├── pebble-store/  # 数据持久层
│   └── pebble-translate/ # 翻译
├── src/               # React 前端
├── src-tauri/         # Tauri 后端 & IPC
├── site/              # 官网
└── scripts/           # 构建脚本
```

## 许可证

[AGPL-3.0](LICENSE) — lvxiaochao @ Fiberhome
