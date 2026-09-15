# Codex 时区启动器 (Windows 专享版)

> 本项目是一款面向 Windows 10/11 的桌面工具，专为 Codex 客户端提供进程级独立时区启动能力。核心源码基于 Tauri 2 (Rust) + Vue 3 (MacVue) 构建，深度集成 Windows 底层系统 API。

---

## 📖 项目简介与工作原理

在多时区办公、跨国远程协作或测试调试时，如果直接修改 Windows 操作系统的系统时区，会导致系统日历、浏览器、即时通讯工具（微信、企业微信、飞书、Teams 等）的所有时间显示全部错乱。

**Codex 时区启动器** 采用**进程级时区隔离**技术：
- 在启动 Codex 进程时，仅针对其子进程环境注入特定的 `TZ` 环境变量，使 Codex 客户端（基于 Electron / Chromium 架构）按照指定的目标时区渲染、解析并发送时间戳。
- **不修改**操作系统的全局系统时区与硬件时钟，完全不干扰其他正在运行的桌面应用。

---

## ✨ 核心特性

- **进程级时区隔离**：目标时区仅在由本启动器拉起的 Codex 实例中生效，隔离安全、绿色无污染。
- **灵活的双模式时区**：
  - **地区规则模式 (IANA)**：支持如 `Asia/Shanghai`、`America/New_York`、`Europe/London` 等标准 IANA 时区，自动遵循当地夏令时 (DST) 切换规则；
  - **固定 UTC 偏移模式**：支持从 `UTC−12:00` 至 `UTC+14:00` 的整小时偏移自由选择。
- **智能客户端识别与校验**：
  - 通过 WinRT `PackageManager` 自动扫描已安装的 UWP/MSIX 应用包（`OpenAI.Codex`）；
  - 自动回退探测 `%LocalAppData%\Programs` 传统安装路径；
  - 严格校验 `icudtl.dat` 关键文件，防止用户误选命令行 CLI 程序。
- **多会话进程防冲突**：
  - 结合 Windows ToolHelp32 进程快照与用户会话 ID（`ProcessIdToSessionId`），精准判断当前用户桌面上是否已有正在运行的客户端，杜绝重复启动导致时区失效。
- **原生桌面快捷方式一键生成**：
  - 调用 Windows COM 组件（`IShellLinkW` / `IPersistFile`）直接在桌面生成 `.lnk` 快捷方式，启动即生效。
- **双通道公网 IP 与地理位置感知**：
  - 分离探测国内线路（腾讯、IPIP.net、又拍云、Cloudflare CN）与国际线路（AWS、Cloudflare Workers、IPify、IP.SB）；
  - 自动根据 IP 解析目标归属地与所在时区，支持一键将识别出的时区填入当前配置草稿；
- **系统托盘常驻与右键快启**：
  - 支持 Windows 任务栏系统托盘常驻运行，点击窗口右上角关闭按钮自动无感最小化到托盘；
  - 单击或双击托盘图标快速唤起/前置主界面；
  - 右键托盘图标可呼出快捷菜单，支持一键调出主窗口、以当前时区快速启动 Codex 以及彻底退出程序。
  - 配置保存采用临时文件落盘 + `sync_all()` 强制刷盘 + 原子重命名替换，防止突然断电或程序崩溃造成配置文件损坏。
  - 支持便携式运行：配置文件自动保存在可执行文件旁的 `data/` 目录中。

---

## 🖥 界面交互与美学

基于 **MacVue** 现代桌面组件库开发，提供媲美原生系统的视觉质感：
- **实时双时钟对照**：左右并列卡片实时展示“目标时区时间”与“Windows 本地时间”，支持毫秒级对齐刷新。
- **智能节电优化**：前端监听 `visibilitychange` 事件，窗口最小化或后台隐藏时自动挂起计时器，重返前台时自适应整秒补齐，无无效资源开销。
- **主题外观**：支持跟随系统、浅色模式与深色模式，Portal 弹窗及菜单深度继承全局主题。
- **无障碍与自适应**：支持 1280×900、760×900 及 600×800 各种窗口尺寸动态响应式排版。

---

## 🚀 快速上手

### 直接运行（便携版）

1. 下载或从根目录直接双击 `CodexTimeZoneLauncher.exe` 即可运行；
2. 启动器将配置保存在同级 `data/` 目录，可随文件夹便携移动；
3. 运行前请确保当前系统已安装 **Microsoft Edge WebView2 Runtime**（Windows 10/11 通常已内置）。

---

## 🛠 从源码编译构建

### 环境要求

- **操作系统**：Windows 10 / 11 x64
- **Node.js**：`>= 22.12.0`（推荐 Node.js 24）
- **npm**：`>= 10.0.0`
- **Rust**：`stable` 工具链（包含 `cargo` 与 `rustc`）
- **C++ 编译环境**：Visual Studio 2022 Build Tools（安装时勾选“使用 C++ 的桌面开发”及 Windows 10/11 SDK）

### 本地编译步骤

1. **克隆仓库并进入源码目录**：
   ```powershell
   cd resource
   ```

2. **安装依赖**：
   ```powershell
   npm install
   ```

3. **开发模式（支持实时热更新）**：
   ```powershell
   npm run dev
   ```

4. **编译生成 Windows 便携版单文件 Release 产物**：
   ```powershell
   npm run build
   ```
   *构建完成后，优化后的单文件可执行程序将自动输出至仓库一级目录：`../CodexTimeZoneLauncher.exe`。*

5. **直接启动已构建的成品**：
   ```powershell
   npm run start
   ```

---

## 📂 项目目录结构

```text
.
├── CodexTimeZoneLauncher.exe       # 编译生成的 Windows 便携版单文件应用
├── README.md                       # 项目说明文档
├── LICENSE                         # MIT 开源许可证
└── resource/                       # 核心工程源码目录
    ├── backend/                    # 后端实现与系统调用
    │   ├── win/                    # Windows 原生实现 (WinRT, Win32, COM, 自动探测, 快捷方式)
    │   ├── common.rs               # 通用时区计算、安全配置落盘、网络信息获取
    │   └── zones.json              # 内置收录的全球 IANA 时区与 Windows 时区映射表
    ├── src/                        # Vue 3 前端界面源码
    │   ├── components/             # 界面组件 (设置面板, 双时钟卡片, 网络面板, 底部操作栏)
    │   ├── composables/            # 响应式状态与业务逻辑 (useLauncher, useClocks, useNetworkInfo)
    │   ├── styles/                 # 样式系统与 MacVue 适配样式
    │   └── App.vue                 # 根组件
    ├── src-tauri/                  # Tauri 2 容器配置与 Rust 入口
    ├── desktop.mjs                 # Windows 本地开发、编译编排脚本
    └── package.json                # 前端与构建脚本依赖配置
```

---

## 🔒 隐私与安全性声明

- **零账户依赖**：本项目无需注册或登录任何账户。
- **纯本地运行**：除用户主动点击“刷新网络信息”会请求公开的公共 IP 探测服务（如腾讯、Cloudflare、AWS 等）获取出口 IP 外，本软件不会向任何第三方服务器上传任何配置、使用习惯或系统隐私信息。
- **剪贴板安全**：复制 IP 仅限用户显式点击，且在操作后对写入内容进行一致性校验。

---

## 📄 License

本项目遵循 [MIT License](LICENSE) 开源协议。

> *免责声明：本项目为独立开源工具，并非 OpenAI 官方出品或支持的软件。*
