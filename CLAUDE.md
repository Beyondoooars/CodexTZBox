# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概览

CodexTZBox 是 Windows 10/11 专享的桌面启动器：让 OpenAI Codex 桌面客户端以「进程级独立时区」运行——只向被拉起的 Codex 子进程注入 `TZ` 环境变量，不修改系统时区与硬件时钟。技术栈为 Tauri 2（Rust）+ Vue 3 + TypeScript（UI 基于 @macvue/core）。显示名「Codex 时区沙盒」（产品名 CodexTZBox），产物为仓库根目录的 `CodexTZBox.exe`。

## 常用命令

以下命令均在 `resource/` 目录执行（根目录没有 package.json）：

- `npm install` — 安装依赖；要集中缓存可加 `--cache ../environment/npm-cache`
- `npm run dev` — 开发模式（`desktop.mjs` → PowerShell 装配 VS 环境 → `tauri dev`，额外监听 backend 源文件变更）
- `npm run build` — 编译 Release 单文件产物，输出到仓库根目录 `CodexTZBox.exe`，并按 `CODEX_TOOLKIT_DIR`（默认 `${CODEX_TOOLKIT_DIR:-<工具箱目录>}`）向工具箱目录同步一份拷贝；目标被占用时自动结束后重试
- `npm run start` — 启动已构建的成品
- `npm run build:frontend` — 前端类型检查 + 构建（`vue-tsc --noEmit && vite build`）；`npm run dev:frontend` 只起 Vite

后端校验与测试（在 `resource/src-tauri/`）：

- `cargo check`；`cargo test`（单元测试位于 `resource/backend/common.rs` 的 `#[cfg(test)]`，例如 `cargo test legacy_settings_and_atomic_replace`）
- 建议带上离线缓存环境：`CARGO_HOME=<仓库>/environment/cargo`、`CARGO_TARGET_DIR=<仓库>/environment/cargo-target`

环境变量：`CODEX_TZ_DEV_ROOT`（本机 Windows 工具链根目录）、`CODEX_TZ_PROXY`（构建下载代理）、`CODEX_TOOLKIT_DIR`（构建后同步目标）。

平台约束：仅面向 Windows——`desktop.mjs` 在非 win32 平台直接报错；需要 VS 2022 C++ Build Tools、Rust stable、Node ≥ 22.12；运行需系统具备 WebView2（Win10/11 一般内置）。CI 见 `.github/workflows/build-desktop.yml`（windows-2022 构建，推 `v*` 标签自动发布 Release）。

## 架构

调用链：Vue 前端 `invoke('backend', { command, payload })` → `src-tauri/src/lib.rs` 的 `backend` 命令（spawn_blocking + 全局互斥串行执行）→ `resource/backend/win/mod.rs` 的 `execute()` 按命令分发：`bootstrap` / `discover` / `validate` / `save` / `launch`。注意 `backend/` 经 `#[path]` 模块引入，并不是独立 crate。

- `backend/win/mod.rs`（Windows 原生后端）：客户端发现走 WinRT `PackageManager` 扫 `OpenAI.Codex` MSIX 包 → 解析 `AppxManifest.xml` → 回退 `%LocalAppData%\Programs` 常规路径（已保存客户端路径时 bootstrap 跳过该扫描）；`validate()` 只接受带 Electron resources 的 `codex.exe` / `chatgpt.exe`（防止误选命令行程序）；运行检测用 ToolHelp32 快照 + `ProcessIdToSessionId` 限定当前用户会话；检测到客户端运行中时先返回 `needsExitConfirm`，前端确认后（`exitRunning: true`）对其窗口发送 `WM_CLOSE` 请求正常退出并等待 ≤15 秒再启动（绝不强杀）。
- `launch()` 是核心：先处理「运行中→确认→正常退出」流程，再以子进程方式启动客户端并仅注入 `TZ`（同时移除 `ELECTRON_RUN_AS_NODE`）；对 Codex 26.924+ 必须同时设置 `CODEX_SPARKLE_ENABLED=false` 与 `CODEX_CLI_PATH`（指向 bundled `resources/codex.exe` 或 `%LocalAppData%\OpenAI\Codex\bin`），否则无包标识启动会报 `APPMODEL_ERROR_NO_PACKAGE`；spawn 后 800ms 内早退则读 stderr 回传错误。
- `src-tauri/src/lib.rs`（Tauri 外壳）：单实例防多开（命名互斥锁 + 唤醒事件，二次启动把已有窗口前置后退出）；系统托盘（显示主窗口 / 保存并启动 Codex / 退出；关闭窗口只隐藏）；`launch` 成功且 `launched: true` 时自动隐藏主窗口，客户端运行中时把窗口带到前台并经 `tray-launch-running` 事件交由前端确认重启；托盘启动失败通过 `tray-launch-error` 事件回传前端。
- `backend/common.rs`：`Settings`（`mode: "zone" | "offset"`，JSON 字段 camelCase、兼容旧版 PascalCase 别名）；`tz()` 校验并生成 TZ 值——offset 模式映射为 `Etc/GMT` 反向符号（如 +8 → `Etc/GMT-8`）；`zones.json`（27 组 `label/id/windowsId`）既是下拉数据源也是校验白名单；配置保存为原子写入（临时文件 + `sync_all` + persist + 重试）；网络出口探测完全在前端（多源 fetch）。
- 前端 `resource/src/`：状态逻辑集中在 composables——`useLauncher`（bootstrap、保存/启动、脏数据与禁用判断）、`useClocks`（Intl 计算双时钟与时差、缓存 formatter）、`useNetworkInfo`（国内/国际多源竞速探测、localStorage 缓存）、`useAppearance`；时区选择由 `components/ui/ZoneSelect.vue` 实现（搜索型组合框，数据源 `src/zones.ts`，约 140 个常用时区，未收录的合法 IANA 名可直接输入）；开发预览用 URL 参数 `?ui-preview=1` 走演示数据。
- 配置存放：优先 exe 同级 `data/`（便携），不可写时降级到 `%LocalAppData%\CodexTZBox\data`，并自动迁移旧版 `CodexTimeZoneSwitcher` / `ChatGPTTimeZoneLauncher` 配置；WebView2 剖面缓存位于 `%LocalAppData%\CodexTZBox`（identifier，可重建可删）。
- 构建编排 `resource/desktop.mjs`：dev/build/start 统一入口；未带 `--configured` 时先经 `backend/win/desktop.ps1` 装配 VS 开发环境再自我重启；依赖与构建缓存集中在 `environment/`；release profile 开 LTO/strip/panic=abort。`tauri.conf.json` 的 CSP 白名单列明了所有 IP 探测域名——新增或更换探测源时必须同步更新，否则前端请求会被拦截。

## 约定与注意

- `.gitignore` 有意忽略：`environment/`、`data/`、`docs/`、`.github/`、根目录 exe。`docs/` 是本地文档区（交接记录、工作日志，不上传）；`.github/` 工作流仅本地保留；桌面产物由 CI/Release 分发，不进源码树。
- 界面文案与 README 均为中文；更新功能时同步维护根目录 `README.md` 与 `resource/README.md`（源码目录约定说明）。
- 视觉语言（2026-10-22 重构）：纸面底色 + 墨色文字，双时钟对比是唯一主角（`TimePreview.vue`），其余为细线分隔的安静表单；设计令牌在 `styles/tokens.css`（含深色模式），MacVue 组件覆盖在 `styles/macvue-adapter.css`。不再使用卡片阴影堆叠与状态徽章。
- 改动代码前先与用户对齐计划（迁移交接时用户明确指示「先不着急改代码」）；完成后建议跑 `npm run build:frontend` + `cargo check` 验证。
- Git：`main` 分支；远端 `github.com/Beyondoooars/CodexTZBox`（2026-10-22 起，旧仓库 `codexTimeZoneRev` 已删除）。

## 当前状态（2026-10-12）

- v0.1.3，最后提交 `93c2253`「support OpenAI Codex 26.924+ update」。工作区存在未提交改动：托盘常驻、单实例防多开、纯白极简双时钟条、健壮性加固等。
- 项目于 2026-10-12 从 `<旧工作目录>` 整体迁移至 `<工作目录>`（Git 历史与离线缓存完整保留），前端构建与 `cargo check` 已验证通过；交接记录在 `docs/handoffs/work/` 下。
- 2026-10-13：品牌与产物名已整体切换为 `CodexTZBox.exe` /「Codex 时区沙盒」——含 crate/包名 `codextzbox`、互斥锁/唤醒事件名 `CodexTZBox_*`、data 回退目录与 CI 产物名、工具箱默认目录 `${CODEX_TOOLKIT_DIR:-<工具箱目录>}`。
