<div align="center">

<img src="./src-tauri/icons/icon.png" alt="smtc2web" width="112" height="112">

# smtc2web

**Pipe whatever your system is playing straight into your OBS scene.**

Reads SMTC on Windows and MPRIS2 on Linux, then serves it over a local HTTP
endpoint for an OBS Browser source. Themes are swappable with hot reload, and
the UI speaks 11 languages.

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
[![Windows](https://img.shields.io/badge/Windows-%2311d0f5?logo=windows&logoColor=white)](#platform-support)
[![Linux](https://img.shields.io/badge/Linux-%23ccc?logo=linux&logoColor=black)](#platform-support)

**[中文](./README.md)** · **[English]**

</div>

---

![OBS Connect to smtc2web Screenshot](./screenshot.png)

> The new icon combines Bootstrap's headphone image with Rust's Ferris character to create an icon of listening to music with headphones.

## Table of Contents

- [Features](#features)
- [Quick start](#quick-start)
- [Themes](#themes)
- [Configuration](#configuration)
- [Development and build](#development-and-build)
- [Linux MPRIS2 support](#linux-mpris2-support)
- [Platform support](#platform-support)
- [Security](#security)
- [Repository analytics](#repository-analytics)
- [License and credits](#license-and-credits)

## Features

| Area | Capability |
| --- | --- |
| Media source | Windows: SMTC (GlobalSystemMediaTransportControls, event-driven); Linux: MPRIS2 (D-Bus, polling player discovery) |
| Stream hookup | Local HTTP server listening on `127.0.0.1:3030` by default; open it in an OBS Browser source, no plugin needed |
| Themes | Built-in Spotify-style default theme; install via ZIP upload; install from a Git repo with update checks |
| Hot reload | Switching themes does not restart the server; SSE tells every connected browser source to refresh instantly |
| Interface | System tray menu, lightweight mode that tears down the WebView on close, start-minimized to tray |
| Updates | Tauri updater with signature-verified automatic updates; auto-check can be turned off |
| Customization | Process filter (pick the audio source by app id / name), system font picker, 11 UI languages |
| Misc | Single-instance guard, album-art LRU cache, config hot reload, daily-rotated logs |

## Quick start

1. Download an installer from [Releases](https://github.com/smtc2web/smtc2web/releases/latest).

   | Platform | Available formats |
   | --- | --- |
   | Windows | `smtc2web_<version>_x64-setup.exe`, `smtc2web_<version>_arm64-setup.exe` |
   | Linux (Debian/Ubuntu) | `smtc2web_<version>_amd64.deb`, `smtc2web_<version>_arm64.deb` |
   | Linux (Fedora/RHEL) | `smtc2web-<version>-1.x86_64.rpm`, `smtc2web-<version>-1.aarch64.rpm` |

2. Launch smtc2web. A tray icon appears and currently playing media is picked up automatically.
3. In OBS, add a **Browser** source and set the URL to:

   ```text
   http://localhost:3030
   ```

   If you changed the listening port in settings, use that port here instead.
4. Play something and the now-playing overlay shows up in your scene.

## Themes

- **Default theme**: embedded into the binary at compile time, styled after Spotify, maintained in-repo as the [`smtc2web-theme-spotify`](https://github.com/AkarinLiu/smtc2web-theme-spotify) submodule.
- **ZIP install**: upload a `.zip` theme package from the themes page; it is extracted into the local theme directory and then enabled.
- **Git install**: provide a theme repo URL and branch (`https` only); the app clones it, registers it, and can later check for updates and upgrade in one click.
- **Local theme directory**: `<config_dir>/smtc2web/theme/`. On switch, the on-disk directory takes priority and the embedded theme is the fallback.

## Configuration

The config file lives at `<config_dir>/smtc2web/config.toml`, is watched after save, and hot-reloads.

| System | Config path |
| --- | --- |
| Windows | `%APPDATA%\smtc2web\config.toml` |
| Linux | `~/.config/smtc2web/config.toml` |

Key fields:

| Field | Default | Description |
| --- | --- | --- |
| `server_port` | `3030` | HTTP server port |
| `address` | `127.0.0.1` | Listen address; switching to a LAN address exposes the service without authentication — be careful |
| `current_theme` | empty (default theme) | Active theme |
| `locale` | `zh-CN` | UI language |
| `process_filter` | `*` | Line-based match against player processes to choose the audio source |
| `auto_check_update` | `true` | Whether to check for updates automatically |
| `minimize_to_tray` | `false` | Start minimized to the tray |
| `lightweight_mode` | `false` | Destroy the WebView when the window closes, keeping only the background process |
| `font_family` | system font stack | Font used by the theme overlay |

Logs are written per day to `<data_dir>/smtc2web/logs/YYYY-MM-DD.log` and kept for 30 days (on Linux, `~/.local/share/smtc2web/logs/`).

## Development and build

The local toolchain is managed by [mise](https://mise.jdx.dev/); [.mise.toml](./.mise.toml) pins Node and pnpm, while Rust still comes from the rustup stable toolchain.

```bash
# 1. Install the pinned node / pnpm
mise install

# 2. Fetch submodules (the default theme)
git submodule update --init --recursive

# 3. Install dependencies
pnpm install

# 4. Dev mode (frontend + Rust hot reload)
pnpm tauri dev

# 5. Build installers
pnpm tauri build
```

Common checks:

```bash
pnpm typecheck   # vue-tsc type checking
cargo check      # Rust check, run inside src-tauri
cargo clippy -- -D warnings
```

Windows needs the MSVC build tools and the WebView2 runtime; Linux needs system dependencies such as webkit2gtk-4.1. Full build docs are at <https://smtc2web.org/wiki/compile>.

Recommended IDEs:

- [VS Code](https://code.visualstudio.com/) / [TRAE](https://trae.com.cn/) + [Tauri extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Zed](https://zed.dev/)
- [OpenCode](https://opencode.ai)

## Linux MPRIS2 support

smtc2web reads currently playing media on Linux via the **MPRIS2** (Media Player Remote Interfacing Specification) protocol. Most native Linux media players (VLC, Rhythmbox, Audacious, etc.) support MPRIS2 out of the box.

### Snap / Flatpak Sandbox Issues

Players installed via Snap or Flatpak (e.g. Firefox, Spotify) are sandboxed by default and **cannot be accessed by external programs over D-Bus**. You need to run the following commands to allow D-Bus access.

#### Snap

1. List installed Snap packages:
   ```bash
   snap list
   ```

2. Connect the `dbus` interface (using Firefox as an example):
   ```bash
   sudo snap connect firefox:dbus-daemon
   ```
   > If the `dbus-daemon` interface is not available, try:
   > ```bash
   > sudo snap connect firefox:session-dbus-observing
   > ```

3. Verify the interface is connected:
   ```bash
   snap connections firefox | grep dbus
   ```

4. **Restart the player** for changes to take effect.

#### Flatpak

1. Using Flatseal (GUI, recommended):
   ```bash
   flatpak install flathub com.github.tchx84.Flatseal
   ```
   Open Flatseal → select the target player → under **System Bus** or **Session Bus**, add `org.mpris.MediaPlayer2.*`.

2. Or override permissions via command line (using Firefox as an example):
   ```bash
   sudo flatpak override --socket=session-bus org.mozilla.firefox
   ```

3. **Restart the player** for changes to take effect.

### Troubleshooting MPRIS2

```bash
# Check for running MPRIS2 players
busctl --user list | grep mpris

# Manually query player status
gdbus call --session \
    --dest org.mpris.MediaPlayer2.<player-name> \
    --object-path /org/mpris/MediaPlayer2 \
    --method org.freedesktop.DBus.Properties.Get \
    org.mpris.MediaPlayer2.Player PlaybackStatus

# View smtc2web logs (located at ~/.local/share/smtc2web/logs/)
```

### Known Limitations

- **Snap Firefox**: Some Snap versions have very strict AppArmor policies — even connecting the `dbus` interface may not grant MPRIS2 access. In this case, install Firefox directly via your system package manager (`apt install firefox` or `dnf install firefox`).
- **Flatpak**: Some Flatpak apps do not expose the `org.mpris.MediaPlayer2.*` D-Bus interface; this requires explicit support from the app developer.

## Platform support

| Platform | Media source | Status |
| --- | --- | --- |
| Windows 10 / 11 (x64, arm64) | SMTC | Supported |
| Linux (x86_64, aarch64) | MPRIS2 | Supported |
| macOS | — | No media source implemented; no builds published |

## Security

- The service binds to `127.0.0.1` by default. Switching to a LAN address lets any device on the network read now-playing info and theme files without authentication.
- Themes (including third-party ones installed via ZIP or Git) are front-end code that executes inside the browser source — only install themes from sources you trust.
- Update packages are installed only after minisign signature verification.

Report vulnerabilities privately via [SECURITY.md](./SECURITY.md); only the latest patch release receives security updates.

## Repository analytics

<p align="center">
  <a href="https://repobeats.axiom.co">
    <img src="https://repobeats.axiom.co/api/embed/e6457313450a17186c46c99351306cb3f1fe820b.svg" alt="RepoBeats analytics" title="Repobeats analytics image">
  </a>
</p>

## License and credits

- Released under the [MIT](./LICENSE) license.
- Built on [Tauri 2](https://v2.tauri.app/), [Vue 3](https://vuejs.org/), [Rust](https://www.rust-lang.org/) and [warp](https://github.com/seanmonstar/warp).
- The Linux media source relies on the [mpris](https://crates.io/crates/mpris) crate; the Windows source uses the system SMTC API.
- Origin discussion thread (LinuxDO): <https://linux.do/t/topic/937994>
- See [CONTRIBUTING.md](./CONTRIBUTING.md) to take part and [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) for conduct rules.
