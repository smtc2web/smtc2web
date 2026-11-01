# smtc2web 安全审计与修复报告

- **审计日期**：2026-10-27
- **项目版本**：0.6.8
- **审计分支**：`security/fix-audit-2026-10`
- **审计范围**：Rust 后端（`src-tauri`）、Vue 前端依赖、主题安装/解压链路、Tauri 配置、GitHub Actions 发布流程
- **数据来源**：GitHub CLI（Dependabot Alerts / 仓库安全设置）、OSV.dev（RustSec + GHSA）、`pnpm audit`、人工代码审计

---

## 1. 执行摘要

| 项目 | 修复前 | 修复后 |
| --- | --- | --- |
| GitHub Dependabot open 告警 | 9 条（另有 21 条历史已修复） | 本地已修复 8 条；1 条（glib）受上游阻塞 |
| Rust 依赖 OSV 记录 | 38 条（16 个包版本） | 5 条（4 个包，均不可利用或仅“停止维护”） |
| npm `pnpm audit` 告警 | 7 条（3 high / 3 moderate / 1 low） | **0** |
| 代码级安全问题 | 12 项 | 11 项已修复 / 1 项已加固并按设计接受（C-10） |

**主要结论**

1. **主题安装/解压链路存在严重问题**：旧实现允许通过构造 ZIP 条目在 Windows 上写出主题目录（Zip Slip 变种），也允许通过 `theme.toml` 的 `screenshot` 读取主题目录外的任意文件；主题目录名可包含 `..`，可导致 `remove_dir_all` 删除配置目录。
2. **Git 主题安装存在参数注入/命令执行风险**：仓库 URL 与分支名未校验，`git clone` 可能执行本地命令。
3. **依赖侧存在大量已公开漏洞**：`zip 0.6.6`、`h2 0.3.27`、`rustls 0.23.41`、`tokio 1.50.0`、`quick-xml 0.38.4`、`zstd` 等均受影响；`warp 0.3` 还间接带入受影响的 `h2`。
4. 修复后仅剩 4 个包存在安全公告：其中 2 个是“停止维护”类公告（`proc-macro-error`、`ttf-parser`），另外 2 个（`glib`、`block-buffer`）因上游依赖尚未迁移而保留，但当前代码路径不可利用。

---

## 2. 扫描结果

### 2.1 GitHub Dependabot（`gh api`）

通过 GitHub CLI 读取 `repos/smtc2web/smtc2web/dependabot/alerts`，审计时共有 **30 条**告警：21 条状态为 `fixed`，9 条为 `open`。

9 条 open 告警及本地修复状态：

| # | 生态 | 包 | 当前版本 | 严重性 | 公告 | 本地状态 |
| --- | --- | --- | --- | --- | --- | --- |
| 33 | npm | vite | 8.1.0 | low | GHSA-9jrq-w75r-8gcw | 已升级至 8.3.5 |
| 32 | npm | vite | 8.1.0 | medium | GHSA-rq7h-c2jc-7f22 | 已升级至 8.3.5 |
| 31 | npm | vite | 8.1.0 | medium | GHSA-r825-w8mm-5pxw | 已升级至 8.3.5 |
| 30 | npm | vite | 8.1.0 | high | GHSA-587h-7wmj-h5pm | 已升级至 8.3.5 |
| 29 | npm | vite | 8.1.0 | medium | GHSA-698v-4r3x-3vcv | 已升级至 8.3.5 |
| 28 | rust | rustls | 0.23.41 | medium | GHSA-2mjx-qc3c-rqvc | 已升级至 0.23.46 |
| 27 | npm | source-map-js | 1.2.1 | high | GHSA-68fv-2mgg-jv7q | 已通过 override 提升至 1.2.2 |
| 26 | npm | @vue/server-renderer | 3.5.39 | high | GHSA-g2v6-rqmx-r4w6 | 随 vue 升级至 3.5.44 |
| 21 | rust | glib | 0.18.5 | medium | GHSA-wrw7-89jp-8q8g | **保留**，见 §5 |

> Dependabot 是服务端扫描结果，推送本分支后 GitHub 会重新扫描并自动关闭已修复的告警。`glib` 为 Linux 目标依赖，Windows 本地构建不会编译它。

### 2.2 Rust 依赖（OSV.dev）

原始 `Cargo-osv.json.out` 共 **38 条**（包，公告）记录，涉及 16 个包版本。修复后复扫 `Cargo.lock`：

| 包 | 修复前 | 修复后 | 处理方式 |
| --- | --- | --- | --- |
| anyhow | 1.0.102 | **1.0.104** | `cargo update` |
| h2 | 0.3.27 | **0.4.20** | `warp 0.3 → 0.4`（hyper 1.x） |
| quick-xml | 0.38.4 | **0.42.0** | 随 `plist 1.8 → 1.10.3` 升级 |
| rustls | 0.23.41 | **0.23.46** | `cargo update` |
| rustls-webpki | 0.103.13 | **0.103.16** | `cargo update` |
| tokio | 1.50.0 | **1.54.1** | `cargo update` |
| webpki-root-certs | 1.0.9 | **1.0.11** | `cargo update` |
| zip | 0.6.6 + 4.6.1 | **9.1.2（仅一份）** | 直接依赖升级；关闭 updater 的 zip 特性 |
| zstd-safe / zstd-sys | 5.0.2 / 2.0.16 | **已移除** | 不再启用 zstd 解压 |
| unic-*（5 个） | 0.9.0 | **已移除** | 随 tauri 2.12.5 升级移除 |
| block-buffer | 0.10.4 | 0.10.4（另有 0.12.1） | 见 §5，上游未迁移 |
| glib | 0.18.5 | 0.18.5 | 见 §5，GTK 上游阻塞 |
| proc-macro-error | 1.0.4 | 1.0.4 | 停止维护，仅 Linux 构建期 |
| ttf-parser | 0.25.1 | 0.25.1 | 停止维护，输入为本地系统字体 |

**已消除的 33 条记录（30 个唯一公告）主要包括：**

- **zip（6 条 / 3 个公告）**：RUSTSEC-2026-0367（保留 SUID/SGID）、RUSTSEC-2026-0368（符号链接逃逸）、RUSTSEC-2026-0369（通过符号链接应用权限）——修复版本 9.0.0。
- **rustls（5 条）**：RUSTSEC-2026-0285 / GHSA-2mjx-qc3c-rqvc（TLS 1.3 握手跨加密层级）、RUSTSEC-2026-0346（Debug 泄露会话密钥）、RUSTSEC-2026-0348（Brotli 证书解压最大 1 GiB）、RUSTSEC-2026-0349（未认证 ServerHello 降级）——修复版本 0.23.45 / 0.23.46。
- **zstd-safe（9 条）**：RUSTSEC-2026-0395 / 0397 / 0399 / 0400 / 0401 / 0402 / 0403 / 0404 / 0406（UAF、OOB、未初始化内存、double free）——修复版本 8.2.0；本项目中已不再引入 zstd。
- **zstd-sys（1 条）**：RUSTSEC-2026-0410（wasm 分配器 shim 破坏内存）——修复版本 2.1.2。
- **quick-xml（2 条）**：RUSTSEC-2026-0194 / 0195（重复属性 O(n²)、命名空间无界分配导致 DoS）——修复版本 0.41.0。
- **h2（1 条）**：RUSTSEC-2026-0258（无界空 DATA 帧）——修复版本 0.4.16。
- **tokio（1 条）**：RUSTSEC-2026-0386（`AsyncFd` 违反 I/O safety）——修复版本 1.51.6 / 1.53.3。
- **anyhow（1 条）**：RUSTSEC-2026-0190（`Error::downcast_mut` 违反借用规则）——修复版本 1.0.103。
- **rustls-webpki / webpki-root-certs（2 条）**：RUSTSEC-2026-0382 / 0383（信任已泄露私钥的 emSign 中间证书）——修复版本 0.103.16 / 1.0.10。
- **unic-*（5 条）**：RUSTSEC-2025-0075 / 0080 / 0081 / 0098 / 0100（停止维护）——随 Tauri 升级移除。

### 2.3 npm 依赖（`pnpm audit`）

修复前 7 条：`vite` 5 条（含 Windows 下 launch-editor 任意命令执行、`server.fs.allow` 绕过等）、`@vue/server-renderer` XSS、`source-map-js` 事件循环 DoS。

修复动作：

- `vite` `8.1.0 → 8.3.5`（`package.json` 范围同步更新）
- `vue` `3.5.39 → 3.5.44`，同时将 `@vue/server-renderer` 提升至 3.5.44
- `pnpm-workspace.yaml` 新增 `source-map-js@<1.2.2: ^1.2.2` override

修复后 `pnpm audit`：**0 条**。

---

## 3. 代码级安全问题与修复

### C-01 主题上传文件名路径穿越（任意文件写入）

- **位置**：`src-tauri/src/lib.rs` `upload_theme_from_bytes`
- **严重性**：High（纵深防御；仅可由应用 WebView 调用）
- **问题**：直接把前端传入的 `file_name` 拼到系统临时目录：`temp_dir.join(&file_name)`。若 `file_name` 为 `..\..\some\path`，可在临时目录之外创建/覆盖文件；临时文件名可预测，也存在符号链接预置风险。
- **修复**：
  - 只取 `Path::file_name()`，拒绝包含路径的文件名；
  - 强制 `.zip` 扩展名；
  - 临时文件改为 `smtc2web-theme-<pid>-<nanos>.zip`，使用 `create_new(true)`，防止覆盖与符号链接；
  - 无论成功失败都删除临时文件。
- **验证**：`cargo test` 通过；逻辑审阅确认不再使用用户提供的路径拼接。

### C-02 主题 ZIP 解压路径穿越 / Zip Slip（任意文件写入）

- **位置**：`src-tauri/src/theme_manager.rs` `extract_theme`
- **严重性**：**High（Windows 上可写出主题目录）**
- **问题**：旧实现只检查原始条目名中是否包含 `..` 或以 `/` 开头，然后执行 `theme_dir.join(relative_path)`。Windows 下以下条目可逃逸：
  - `root/C:/Windows/...`（盘符前缀，`PathBuf::join` 会替换整个路径）；
  - `root/\Windows\...`（反斜杠根路径）；
  - 含 NTFS ADS（`file.txt:stream`）的条目。
- **修复**：
  - 解压前完整遍历并校验所有条目；
  - 统一 `/` 分隔符，要求存在且仅存在一个根目录；
  - 用 `Path::components()` 只接受 `Component::Normal`，拒绝 `..`、`.`、根路径、盘符、UNC、`:`、NUL、Windows 保留设备名、尾随空格/点；
  - 拒绝重复路径、符号链接条目；
  - 解压到 `.staging-*` 临时目录，全部成功后再 `rename` 到正式目录；失败清理临时目录。
- **验证**：新增 5 个单元测试覆盖 `..`、盘符、NUL、隐藏路径与正常路径；`cargo test` 全部通过。

### C-03 ZIP 炸弹 / 无界解压（磁盘或内存耗尽）

- **位置**：`theme_manager.rs` `extract_theme`
- **严重性**：Medium
- **问题**：旧实现直接 `io::copy`，未限制条目数、单文件大小与总解压大小。
- **修复**：
  - 最多 4096 个条目；
  - 单文件 32 MiB、总解压 128 MiB；
  - `theme.toml` 最多 1 MiB；
  - 使用 `Read::take` 做运行时限制（不信任 ZIP 头中的 `size`）。
- **验证**：代码审阅 + 单元测试；超限返回明确错误并清理临时目录。

### C-04 `theme.toml` 截图路径任意文件读取

- **位置**：`theme_manager.rs` `parse_theme_config`
- **严重性**：Medium
- **问题**：`screenshot` 字段直接 `themes_dir.join(theme_folder).join(screenshot)`，恶意主题可写 `screenshot = "../../../../Users/x/.ssh/id_rsa"` 或绝对路径，把任意文件读入内存并 Base64 返回给前端。
- **修复**：
  - 对主题目录与目标文件分别 `canonicalize`，要求目标以主题目录为前缀；
  - 仅允许 `.png/.jpg/.jpeg/.webp/.gif`；
  - 文件大小上限 16 MiB；
  - 拒绝网络 URL。
- **验证**：路径校验逻辑审阅；不存在的文件与越界路径均返回空截图而不是报错中断。

### C-05 主题目录名穿越（任意目录删除）

- **位置**：`theme_manager.rs` `delete_theme`、`update_git_theme`、`check_git_update`、`get_theme_server_path`；`lib.rs` 对应命令
- **严重性**：High
- **问题**：`theme_folder` 直接 `themes_dir.join(theme_folder)`。传入 `..` 时：
  - `delete_theme` 会 `remove_dir_all(config/smtc2web/..)`，删除整个应用配置目录；
  - `update/check` 会操作配置目录之外的仓库。
  - ZIP 文件名如 `...zip`（`file_stem` 为 `..`）或 Git URL 最后一段为 `..` 也能触发同类问题。
- **修复**：
  - 新增 `validate_theme_folder`：仅允许单层、Unicode 字母数字/`- _ . 空格`，拒绝 `default`、`.`、`..`、分隔符、盘符、控制字符、Windows 保留名、尾随空格/点、超长名称；
  - `delete/update/check/get_server_path/extract` 全部先校验；
  - Git 主题目录名从 URL 解析后同样校验。
- **验证**：`rejects_unsafe_theme_names` 等单元测试覆盖。

### C-06 Git 主题安装参数注入 / 本地命令执行

- **位置**：`theme_manager.rs` `install_from_git`、`update_git_theme`、`check_git_update`
- **严重性**：High（需要用户粘贴恶意仓库地址）
- **问题**：
  - `git clone <repo_url> <dir>` 未使用 `--`，以 `-` 开头的 URL 会被当作 git 选项；
  - 允许 `file://`、`ext::` 等协议；配合 `--upload-pack=<cmd>` 或 `ext::sh -c ...` 可执行本地命令；
  - 分支名未校验，`--branch --upload-pack=...` 同样构成参数注入。
- **修复**：
  - URL 必须能被 `url` 解析且 scheme 严格为 `https`；拒绝用户名/密码；
  - 分支名白名单：字母数字、`- _ / .`，拒绝以 `-` / `/` 开头、以 `/` 结尾、包含 `..`、控制字符、超长；
  - `git clone ... -- <url> <dir>`；
  - 设置 `GIT_TERMINAL_PROMPT=0`、`GIT_ALLOW_PROTOCOL=https`；
  - `pull/fetch` 追加 `-c protocol.ext.allow=never`，并同样限制协议。
- **验证**：`repo_url_requires_https`、`git_branch_rejects_argument_injection` 测试通过。

### C-07 主题目录隐藏文件 / `.git` 信息泄露

- **位置**：`src-tauri/src/theme.rs` `serve_theme_file`
- **严重性**：Medium（仅在监听 `0.0.0.0` 时对局域网可见）
- **问题**：自定义主题通过 `canonicalize` 限制在主题目录内，但仍会提供 `.git/config`、`.git/objects` 等文件；Git 主题的 remote URL、历史元数据可被读取。
- **修复**：拒绝任何以 `.` 开头的路径段（`.git`、`.env` 等），并仅允许 `GET` 请求。
- **验证**：`cargo check`/`clippy` 通过；路径匹配逻辑审阅。

### C-08 `open_url` 允许任意协议

- **位置**：`src-tauri/src/lib.rs` `open_url`
- **严重性**：Medium
- **问题**：任意字符串交给 `open::that`，可打开本地文件、可执行文件或自定义协议处理器；若 WebView 出现 XSS，可被滥用。
- **修复**：仅允许 `http`、`https`、`mailto`。
- **验证**：`cargo check` 通过。

### C-09 Tauri WebView 未配置 CSP

- **位置**：`src-tauri/tauri.conf.json`
- **严重性**：Medium
- **问题**：`"csp": null` 且 `withGlobalTauri: true`。一旦主题元数据/更新说明等出现注入点，脚本可加载远程资源并调用本地命令。
- **修复**：加入生产 CSP（`default-src 'self'`、`script-src 'self'`、`style-src 'self' 'unsafe-inline'`、`img-src ... data:`、`connect-src ipc: http://ipc.localhost`），`devCsp` 保持 `null` 以免影响 Vite HMR。前端未发现 `v-html`/`innerHTML` 使用。
- **验证**：`pnpm build` 通过；建议在真实安装包中回归检查更新、主题截图、字体列表等页面。

### C-10 非回环地址监听无鉴权

- **位置**：`src-tauri/src/lib.rs` `start_server` / `save_config`
- **严重性**：Low–Medium（取决于部署）
- **问题**：配置允许 `0.0.0.0`，此时 `/api/now`（媒体元数据）与主题文件对局域网完全开放，无鉴权。
- **处理**：保留该功能（OBS 跨设备使用），启动时增加明确告警；在 `SECURITY.md` 中说明安全模型。**如果未来需要更强的隔离，建议增加访问令牌或反向代理。**
- **状态**：已加固（可视化告警 + 文档），风险按设计接受。

### C-11 发布流程执行未固定版本的 `wrangler@latest`

- **位置**：`.github/workflows/release.yml`
- **严重性**：Medium（供应链）
- **问题**：在持有 `CLOUDFLARE_API_TOKEN` 的发布任务中执行 `npx --yes wrangler@latest`；上游发布被投毒时可窃取凭据或篡改发布物。
- **修复**：固定为 `wrangler@4.153.0`。
- **后续建议**：将所有 GitHub Actions 从 `@vN` 标签改为 commit SHA 固定；发布任务最小化权限。

### C-12 静态响应缺少 `X-Content-Type-Options`

- **位置**：`src-tauri/src/theme.rs`
- **严重性**：Low
- **修复**：为所有主题静态响应增加 `x-content-type-options: nosniff`，降低 MIME 嗅探导致的脚本执行风险。

---

## 4. 关键代码变更

| 文件 | 变更 |
| --- | --- |
| `src-tauri/src/theme_manager.rs` | 主题名/路径校验、截图路径限制、ZIP 安全解压、Zip 炸弹限制、Git URL/分支校验、Git 协议限制、单元测试 |
| `src-tauri/src/lib.rs` | 临时文件安全创建、`open_url` 协议白名单、warp 0.4 API 适配、非回环监听告警、GET 限制 |
| `src-tauri/src/theme.rs` | 隐藏文件拦截、`nosniff` 响应头 |
| `src-tauri/Cargo.toml` / `Cargo.lock` | `warp 0.4`、`zip 9.1`、关闭 updater 的 zip 特性、`url 2`，整体依赖更新 |
| `src-tauri/tauri.conf.json` | 生产 CSP |
| `package.json` / `pnpm-lock.yaml` / `pnpm-workspace.yaml` | `vue 3.5.44`、`vite 8.3.5`、`source-map-js` override |
| `.github/workflows/release.yml` | 固定 `wrangler@4.153.0` |
| `SECURITY.md` | 重写支持的版本、私密报告渠道与安全模型 |

---

## 5. 残余风险与处置理由

修复后 OSV 仍有 5 条记录（4 个包）：

### 5.1 `glib 0.18.5` — GHSA-wrw7-89jp-8q8g / RUSTSEC-2024-0429

- **性质**：`VariantStrIter` 的 `Iterator`/`DoubleEndedIterator` 实现存在 UB，修复版本 `glib 0.20.0`。
- **为何保留**：`glib 0.18` 由 Linux 端 GTK 0.18 栈引入（`gtk → tray-icon / muda / tauri`）。Tauri 2.12.5 / wry 0.57.1 尚未迁移到 GTK-rs 0.20；强行 patch 会破坏 ABI/API。
- **可利用性**：应用代码未直接使用 `glib::VariantStrIter`，无攻击者可控输入进入该 API。
- **处置**：接受并监控 Tauri/wry/gtk-rs 升级；Windows 构建不受影响。

### 5.2 `block-buffer 0.10.4` — GHSA-qwgh-2vcv-g2f7

- **性质**：捕获 panic 后 `EagerBuffer`/`ReadBuffer` 游标损坏，可导致越界读写；修复版本 `0.12.1`。
- **为何保留**：`warp 0.4` 的 `headers 0.4 → sha1 0.10` 与 `tauri-codegen → sha2 0.10` 仍使用 `digest 0.10`；上游尚未迁移到 `digest 0.11`。
- **可利用性**：需要“哈希过程中发生 panic 且被捕获”的前置条件；本项目不对攻击者输入做哈希，`sha1` 仅用于未启用的 WebSocket 头处理，`sha2` 仅构建期使用。
- **处置**：接受并跟踪 `headers`/`sha1`/`sha2` 更新。

### 5.3 `proc-macro-error 1.0.4` — RUSTSEC-2024-0370

- 停止维护公告；仅 Linux 构建期由 GTK 宏依赖引入，不进入运行时，也无已知 CVE。接受。

### 5.4 `ttf-parser 0.25.1` — RUSTSEC-2026-0192

- 停止维护公告（无已知 CVE）；仅用于扫描系统字体目录中的本地字体文件，输入不受远程控制。
- 后续可将 `font.rs` 迁移到公告推荐的 `skrifa`，但会引入新的字体解析依赖，建议作为独立改动并在多平台字体目录上回归。

### 5.5 其他设计性风险

- **主题即代码**：第三方主题会在 OBS 浏览器源中执行任意 JavaScript，可读取媒体信息并向外发起请求。ZIP/Git 安装只校验结构，不校验内容。建议只安装可信主题，后续可考虑主题签名。
- **本地 HTTP 无鉴权**：默认 `127.0.0.1` 风险可接受；改为 `0.0.0.0` 时请使用可信局域网，或增加访问令牌。
- **CI 供应链**：GitHub Actions 仍使用 major tag（`@v4`/`@v2` 等），建议后续统一 pin 到 commit SHA；仓库已启用 secret scanning，但 push protection 处于关闭状态，建议开启。
- **Linux 构建未在本地验证**：本次在 Windows 主机完成编译/测试；CI 的 Linux 矩阵会验证 `gtk/glib` 路径。

---

## 6. 验证结果

| 检查 | 结果 |
| --- | --- |
| `cargo check` | ✅ 通过 |
| `cargo test --lib` | ✅ 9 passed, 0 failed |
| `cargo clippy --all-targets -- -D warnings` | ✅ 通过 |
| `pnpm build`（`vue-tsc` + Vite） | ✅ 通过 |
| `pnpm audit` | ✅ 0 vulnerabilities |
| OSV 复扫 `Cargo.lock` | 5 条记录 / 4 个包（均为 §5 评估项） |
| 依赖版本核验 | `warp 0.4.3`、`zip 9.1.2`、`h2 0.4.20`、`rustls 0.23.46`、`tokio 1.54.1`、`quick-xml 0.42.0`、`plist 1.10.3`；`zip@4.6.1` 与 `zstd` 已从依赖树移除 |

---

## 7. 后续建议

1. **合并并推送后确认 Dependabot**：GitHub 会重新扫描，预期关闭 8 条 open 告警；`glib` 保持 open 直至 Tauri/GTK 升级。
2. **在 CI 中加入 `cargo audit` / `cargo osv`**，配合 `.cargo/audit.toml` 对 4 个已知残余项做带理由的忽略，防止新增漏洞无人处理。
3. **固定 GitHub Actions 到 commit SHA**，尤其是 `tauri-action`、`pnpm/action-setup`、`rust-cache` 等第三方 Action。
4. **启用 secret scanning push protection**。
5. **主题签名/来源校验**（可选）：对官方主题市场下发的 ZIP 增加签名，降低“安装恶意主题”的社会工程风险。
6. **若需要局域网访问**：为 `/api/now` 与主题服务增加可选的访问令牌，避免 `0.0.0.0` 裸奔。

---

## 8. 复现命令

```powershell
# 1. 查看 GitHub Dependabot 告警
gh api "repos/smtc2web/smtc2web/dependabot/alerts?state=open"
gh api "repos/smtc2web/smtc2web/dependabot/alerts?state=fixed"

# 2. Rust 依赖审计（需安装 cargo-audit 或 cargo-osv）
cd src-tauri
cargo audit

# 3. npm 依赖审计
pnpm audit

# 4. 编译与测试
cargo check
cargo test --lib
cargo clippy --all-targets -- -D warnings
pnpm build
```
