# Windows 10/11 主题切换工具 (Rust)

一个使用 **Rust** 语言开发的轻量级 Windows 10/11 明暗主题切换工具，双击即可切换，与系统设置原生的切换速度相当。支持标准版和极简版。

## 技术栈

- **语言**: Rust
- **构建系统**: Cargo
- **目标平台**: x86_64-pc-windows-msvc
- **依赖**: 标准版依赖 `windows-sys`；极简版零外部依赖

## 功能特点

- 🔄 一键切换 Windows 明/暗主题（应用 + 系统主题同时切换）
- ⚡ 即时生效，刷新速度与原生相当
- 📦 超小体积，无运行依赖
- 🦀 基于 Rust，内存安全

## 版本对比

| 特性 | 标准版 | 极简版 |
|------|--------|--------|
| 文件大小 | ~99KB | ~4KB |
| Rust 特性 | 标准库 + windows-sys | `#![no_std]` + 直接 FFI |
| 兼容性 | Windows 10/11 | Windows 10/11 |
| 错误处理 | ✅ 完整 | ⚠️ 静默失败 |
| 代码可维护性 | ✅ 优秀 | ⚠️ 较低 |

## 使用方法

### 标准版 (推荐)
```bash
# 下载或编译标准版 exe 文件，双击运行即可
win-theme-toggle.exe
```

### 极简版
```bash
# 下载或编译极简版 exe 文件，双击运行
win-theme-toggle-mini.exe
```

---

## 编译

### 系统要求
- **操作系统**: Windows 10/11
- **Rust 工具链**: 1.70.0+
- **构建工具**: Visual Studio Build Tools (含 Windows SDK + MSVC)

### 快速搭建 (Scoop)

```powershell
# 安装 Scoop
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
iwr -useb get.scoop.sh | iex

# 安装 Rust (MSVC)
scoop install rustup-msvc
```

然后从 [Microsoft 官网](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 安装 Visual Studio Build Tools，选择：
- Windows 11 SDK（或 Windows 10 SDK）
- MSVC v143 - VS 2022 C++ x64/x86 构建工具

### 编译命令

```bash
git clone https://github.com/maboloshi/win-theme-toggle.git

# 标准版
cd win-theme-toggle/standard
cargo build --release
# → target/release/win-theme-toggle.exe (~99KB)

# 极简版
cd win-theme-toggle/mini
cargo build --release
# → target/release/win-theme-toggle.exe (~4KB)
```

---

## 🛠️ 技术原理

### 三步刷新策略

通过分析 [AutoDarkMode/Windows-Auto-Night-Mode](https://github.com/AutoDarkMode/Windows-Auto-Night-Mode) 的 `DwmRefreshHandler` 和 `ThemeHandler` 实现，本工具采用以下策略：

**Step 1 — 注册表写入**
```
HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize
  ├── AppsUseLightTheme    (0=暗色, 1=亮色)
  └── SystemUsesLightTheme (0=暗色, 1=亮色)
```

**Step 2 — 触发主题服务重载**
读取 `HKCU\...\Themes\CurrentTheme`（当前 .theme 文件路径），**原值写回**。
`RegSetValueExW` 即使写入相同值也会更新键的写入时间戳，触发 Windows 主题服务（themeservice）重新应用当前 .theme 文件。这等效于 AutoDarkMode 通过 COM `IThemeManager2` 应用主题。

**Step 3 — 异步广播消息**
使用 `SendNotifyMessageW`（而非 `SendMessageTimeoutW`）向所有顶层窗口投递消息。由于本进程不创建任何窗口，所有目标窗口均在其它线程，`SendNotifyMessageW` 等价于 `PostMessageW`——投递到消息队列后立即返回，不等待窗口处理。

广播消息序列：

| 消息 | 值 | lParam | 作用 |
|---|---|---|---|
| `WM_SETTINGCHANGE` | `0x001A` | `"ImmersiveColorSet"` | 通知系统设置变更 |
| `WM_THEMECHANGED` | `0x031A` | `0` | 通知 DWM 刷新主题资源 |
| `WM_DWMCOLORIZATIONCOLORCHANGED` | `0x0320` | `0` | 通知 DWM 刷新着色 |

三条消息全部异步投递，工具总耗时 <5ms 即退出。各窗口在自己的线程中收到消息后各自刷新，因此整体刷新速度接近 Windows 设置原生切换。

### 与 AutoDarkMode 的对齐

| 方面 | AutoDarkMode | 本工具 |
|---|---|---|
| 注册表写入 | `AppsUseLightTheme` + `SystemUsesLightTheme` | 相同 |
| 主题应用 | COM `IThemeManager2.ApplyTheme()` | 写回 `CurrentTheme` 触发 themeservice |
| 广播方式 | `SendMessageTimeoutW` (5000ms 超时) | `SendNotifyMessageW` (异步投递) |
| 广播消息 | `WM_SETTINGCHANGE` + `WM_THEMECHANGED` | 另加 `WM_DWMCOLORIZATIONCOLORCHANGED` |

---

## 系统要求

- **操作系统**: Windows 10 1709+ 或 Windows 11
- **架构**: x64
- **权限**: 普通用户权限即可

## 常见问题

**Q: 为什么选择 Rust 开发？**
A: Rust 提供内存安全、零成本抽象和极小运行时，适合系统工具开发。

**Q: 极简版体积为什么这么小？**
A: 禁用了 Rust 标准库 (`#![no_std]`)，不链接任何 crate，直接通过 FFI 调用 Windows API。

**Q: 杀毒软件报毒？**
A: 这是误报。代码完全开源，可自行审查和编译。

**Q: 编译时出现链接错误？**
A: 确保已安装 Visual Studio Build Tools 并重启终端。

**Q: 可以创建桌面快捷方式吗？**
A: 可以，建议固定到任务栏方便快速切换。

## 许可证

MIT License - 可自由使用、修改和分发

## 贡献

欢迎提交 Issue 和 Pull Request。
