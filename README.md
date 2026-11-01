<div align="center">

<img src="./src-tauri/icons/icon.png" alt="smtc2web" width="112" height="112">

# smtc2web

**把系统正在播放的媒体，实时投到你的 OBS 直播间。**

Windows 读取 SMTC，Linux 读取 MPRIS2，再通过本机 HTTP 服务交给 OBS 浏览器源渲染。
主题可换、可热切换，支持 11 种界面语言。

[![Release](https://img.shields.io/github/v/release/smtc2web/smtc2web?color=blue&label=release)](https://github.com/smtc2web/smtc2web/releases/latest)
[![CI](https://github.com/smtc2web/smtc2web/actions/workflows/check.yml/badge.svg)](https://github.com/smtc2web/smtc2web/actions/workflows/check.yml)
[![Downloads](https://img.shields.io/github/downloads/smtc2web/smtc2web/total?label=downloads)](https://github.com/smtc2web/smtc2web/releases)
[![License](https://img.shields.io/badge/License-MIT-green)](./LICENSE)
[![Website](https://img.shields.io/badge/Web-smtc2web.org-4B32C3)](https://smtc2web.org)

[![Stars](https://img.shields.io/github/stars/smtc2web/smtc2web?style=social)](https://github.com/smtc2web/smtc2web/stargazers)
[![Forks](https://img.shields.io/github/forks/smtc2web/smtc2web?style=social)](https://github.com/smtc2web/smtc2web/forks)
[![Contributors](https://img.shields.io/github/contributors/smtc2web/smtc2web?label=contributors)](https://github.com/smtc2web/smtc2web/graphs/contributors)
[![Last commit](https://img.shields.io/github/last-commit/smtc2web/smtc2web?label=last%20commit)](https://github.com/smtc2web/smtc2web/commits/master)
[![Repo size](https://img.shields.io/github/repo-size/smtc2web/smtc2web?label=repo%20size)](https://github.com/smtc2web/smtc2web)

[![Rust](https://img.shields.io/badge/Rust-2024--edition-E5732E?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42B883?logo=vuedotjs&logoColor=white)](https://vuejs.org/)
[![Vite](https://img.shields.io/badge/Vite-8-646CFF?logo=vite&logoColor=white)](https://vite.dev/)
[![Languages](https://img.shields.io/badge/i18n-11%20languages-2B7AB9)](./src/i18n/locales)
[![Windows](https://img.shields.io/badge/Windows-%2311d0f5?logo=windows&logoColor=white)](#平台支持)
[![Linux](https://img.shields.io/badge/Linux-%23ccc?logo=linux&logoColor=black)](#平台支持)

**[中文]** · **[English](./README.en.md)**

</div>

---

![OBS 接入 smtc2web 效果](./screenshot.png)

> 新图标把 Bootstrap 的耳机图形和 Rust 的 Ferris 形象组合成"戴耳机听音乐的蟹宝"。

## 目录

- [功能特性](#功能特性)
- [快速开始](#快速开始)
- [主题](#主题)
- [配置](#配置)
- [开发与构建](#开发与构建)
- [Linux MPRIS2 支持](#linux-mpris2-支持)
- [平台支持](#平台支持)
- [安全说明](#安全说明)
- [仓库数据](#仓库数据)
- [许可与致谢](#许可与致谢)

## 功能特性

| 方向 | 能力 |
| --- | --- |
| 媒体来源 | Windows：SMTC（GlobalSystemMediaTransportControls，事件驱动）；Linux：MPRIS2（D-Bus，轮询发现播放器） |
| 直播接入 | 本机 HTTP 服务默认监听 `127.0.0.1:3030`，OBS 浏览器源直接打开即可，无需插件 |
| 主题 | 内置 Spotify 风格默认主题；支持 ZIP 上传安装；支持从 Git 仓库安装并检查更新 |
| 热切换 | 切换主题不重启服务，通过 SSE 通知所有已连接的浏览器源即时刷新 |
| 界面 | 系统托盘菜单、关闭窗口即销毁 WebView 的轻量模式、启动即隐藏到托盘 |
| 更新 | Tauri updater 签名校验自动更新，可关闭自动检查 |
| 定制 | 进程过滤器（按 app id / 名称挑选音源）、系统字体选择、11 种界面语言 |
| 其他 | 单实例保护、专辑封面 LRU 缓存、配置文件热重载、按天归档日志 |

## 快速开始

1. 从 [Releases](https://github.com/smtc2web/smtc2web/releases/latest) 下载安装包。

   | 平台 | 可用格式 |
   | --- | --- |
   | Windows | `smtc2web_<版本>_x64-setup.exe`、`smtc2web_<版本>_arm64-setup.exe` |
   | Linux (Debian/Ubuntu) | `smtc2web_<版本>_amd64.deb`、`smtc2web_<版本>_arm64.deb` |
   | Linux (Fedora/RHEL) | `smtc2web-<版本>-1.x86_64.rpm`、`smtc2web-<版本>-1.aarch64.rpm` |

2. 启动 smtc2web。正常情况下托盘会出现图标，正在播放的媒体会被自动读取。
3. OBS 中新建 **浏览器** 来源，URL 填：

   ```text
   http://localhost:3030
   ```

   如果你在设置里改过监听端口，这里同步改成对应端口即可。
4. 播放音乐，直播画面里就会出现正在播放的曲目浮层。

## 主题

- **默认主题**：编译时随二进制内嵌，风格参考 Spotify，仓库内以子模块 [`smtc2web-theme-spotify`](https://github.com/AkarinLiu/smtc2web-theme-spotify) 维护。
- **ZIP 安装**：在主题页上传 `.zip` 主题包，解压到本地主题目录后启用。
- **Git 安装**：填入主题仓库地址与分支（仅支持 `https`），由应用克隆并登记，之后可检查更新并一键升级。
- **本地主题目录**：`<config_dir>/smtc2web/theme/`，切换主题时优先读取磁盘目录，缺失则回退到内嵌主题。

## 配置

配置文件为 `<config_dir>/smtc2web/config.toml`，保存后会被自动监听并热重载。

| 系统 | 配置路径 |
| --- | --- |
| Windows | `%APPDATA%\smtc2web\config.toml` |
| Linux | `~/.config/smtc2web/config.toml` |

主要字段：

| 字段 | 默认值 | 说明 |
| --- | --- | --- |
| `server_port` | `3030` | HTTP 服务端口 |
| `address` | `127.0.0.1` | 监听地址，改成局域网地址会无鉴权暴露，请谨慎 |
| `current_theme` | 空（默认主题） | 当前启用的主题 |
| `locale` | `zh-CN` | 界面语言 |
| `process_filter` | `*` | 按行匹配播放器进程，用于挑选音源 |
| `auto_check_update` | `true` | 是否自动检查更新 |
| `minimize_to_tray` | `false` | 启动时最小化到托盘 |
| `lightweight_mode` | `false` | 关闭窗口时销毁 WebView，只保留后台进程 |
| `font_family` | 系统字体栈 | 主题浮层使用的字体 |

日志按天写入 `<data_dir>/smtc2web/logs/YYYY-MM-DD.log`，保留 30 天（Linux 为 `~/.local/share/smtc2web/logs/`）。

## 开发与构建

本地工具链由 [mise](https://mise.jdx.dev/) 管理，仓库内 [.mise.toml](./.mise.toml) 已锁定 Node 与 pnpm 版本；Rust 仍使用 rustup 安装的 stable 工具链。

```bash
# 1. 安装本项目锁定的 node / pnpm
mise install

# 2. 拉取子模块（默认主题）
git submodule update --init --recursive

# 3. 安装依赖
pnpm install

# 4. 开发模式（前端 + Rust 热重载）
pnpm tauri dev

# 5. 构建安装包
pnpm tauri build
```

常用校验命令：

```bash
pnpm typecheck   # vue-tsc 类型检查
cargo check      # 在 src-tauri 目录下检查 Rust
cargo clippy -- -D warnings
```

Windows 需要 MSVC 生成工具与 WebView2 运行时，Linux 需要 webkit2gtk-4.1 等系统依赖。完整构建说明见 <https://smtc2web.org/wiki/compile>。

推荐 IDE：

- [VS Code](https://code.visualstudio.com/) / [TRAE](https://trae.com.cn/) + [Tauri 扩展](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Zed](https://zed.dev/)
- [OpenCode](https://opencode.ai)

## Linux MPRIS2 支持

smtc2web 支持 Linux 下通过 **MPRIS2**（Media Player Remote Interfacing Specification）协议获取正在播放的媒体信息。大多数 Linux 桌面环境中的本地媒体播放器（如 VLC、Rhythmbox、Audacious 等）均原生支持 MPRIS2，开箱即用。

### Snap / Flatpak 沙箱问题

通过 Snap 或 Flatpak 安装的播放器（如 Firefox、Spotify）默认受到沙箱限制，**无法被外部程序通过 D-Bus 访问**。你需要在终端中运行以下命令来放行 D-Bus 访问。

#### Snap

1. 查看当前已安装的 Snap 包：
   ```bash
   snap list
   ```

2. 安装并连接 `dbus` 接口（以 Firefox 为例）：
   ```bash
   sudo snap connect firefox:dbus-daemon
   ```
   > 如果 `dbus-daemon` 接口不可用，可尝试：
   > ```bash
   > sudo snap connect firefox:session-dbus-observing
   > ```

3. 验证接口已连接：
   ```bash
   snap connections firefox | grep dbus
   ```

4. **重启播放器**使配置生效。

#### Flatpak

1. 使用 Flatseal（图形界面，推荐）：
   ```bash
   flatpak install flathub com.github.tchx84.Flatseal
   ```
   打开 Flatseal → 选择目标播放器 → **系统总线（System Bus）** 或 **会话总线（Session Bus）** 中添加 `org.mpris.MediaPlayer2.*`。

2. 或通过命令行重写权限（以 Firefox 为例）：
   ```bash
   sudo flatpak override --socket=session-bus org.mozilla.firefox
   ```

3. **重启播放器**使配置生效。

### 排查其他 MPRIS2 问题

```bash
# 查看是否有 MPRIS2 播放器在运行
busctl --user list | grep mpris

# 手动查询播放器状态
gdbus call --session \
    --dest org.mpris.MediaPlayer2.<播放器名> \
    --object-path /org/mpris/MediaPlayer2 \
    --method org.freedesktop.DBus.Properties.Get \
    org.mpris.MediaPlayer2.Player PlaybackStatus

# 查看 smtc2web 日志（定位在 ~/.local/share/smtc2web/logs/）
```

### 已知限制

- **Snap Firefox**：部分版本 Snaps 的 AppArmor 策略非常严格，即使连接 `dbus` 接口也可能无法访问 MPRIS2。此情况建议使用系统包管理器直接安装 Firefox（`apt install firefox` 或 `dnf install firefox`）。
- **Flatpak**：部分 Flatpak 应用没有暴露 `org.mpris.MediaPlayer2.*` D-Bus 接口，需要应用开发者主动支持。

## 平台支持

| 平台 | 音源 | 状态 |
| --- | --- | --- |
| Windows 10 / 11（x64、arm64） | SMTC | 支持 |
| Linux（x86_64、aarch64） | MPRIS2 | 支持 |
| macOS | — | 未实现媒体源，暂不提供构建 |

## 安全说明

- 服务默认只监听 `127.0.0.1`，改成局域网地址后同网段设备可无鉴权读取正在播放的信息与主题文件。
- 主题（含 ZIP 与 Git 安装的第三方主题）本质上是会在浏览器源中执行的前端代码，请只安装来源可信的主题。
- 更新包经过 minisign 签名校验后才会安装。

发现漏洞请按 [SECURITY.md](./SECURITY.md) 通过私密安全公告报告，只有最新补丁版本会获得安全更新。

## 仓库数据

<p align="center">
  <a href="https://repobeats.axiom.co">
    <img src="https://repobeats.axiom.co/api/embed/e6457313450a17186c46c99351306cb3f1fe820b.svg" alt="RepoBeats 仓库活动分析" title="Repobeats analytics image">
  </a>
</p>

## 许可与致谢

- 本项目以 [MIT](./LICENSE) 许可发布。
- 基于 [Tauri 2](https://v2.tauri.app/)、[Vue 3](https://vuejs.org/)、[Rust](https://www.rust-lang.org/) 与 [warp](https://github.com/seanmonstar/warp) 构建。
- Linux 音源依赖 [mpris](https://crates.io/crates/mpris) crate，Windows 音源依赖系统 SMTC 接口。
- 起源讨论帖（LinuxDO）：<https://linux.do/t/topic/937994>
- 参与方式见 [CONTRIBUTING.md](./CONTRIBUTING.md)，行为准则见 [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)。
