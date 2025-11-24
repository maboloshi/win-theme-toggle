# Windows 10/11 主题切换工具 (Rust)

一个使用 **Rust** 语言开发的轻量级 Windows 10/11 明暗主题切换工具，支持标准版和极简版。

## 🦀 技术栈

- **语言**: Rust
- **构建系统**: Cargo
- **目标平台**: x86_64-pc-windows-msvc
- **依赖管理**: Cargo + Crates.io

## 功能特点

- 🔄 一键切换 Windows 明/暗主题
- ⚡ 即时生效，无需重启
- 🎯 精准切换应用和系统主题
- 📦 超小体积，无依赖
- 🦀 基于 Rust，内存安全，无运行时

## 版本对比

| 特性 | 标准版 | 极简版 |
|------|--------|--------|
| 文件大小 | ~100KB | ~3.5KB |
| Rust 特性 | 标准库 + windows-sys | `#![no_std]` + 直接系统调用 |
| 兼容性 | Windows 10/11 | Windows 10/11 |
| 错误处理 | ✅ 完整 | ⚠️ 静默失败 |
| 代码可维护性 | ✅ 优秀 | ⚠️ 较低 |

## 使用方法

### 标准版 (推荐)
```bash
# 下载标准版 exe 文件，双击运行即可
win-theme-toggle.exe
```

### 极简版
```bash
# 下载极简版 exe 文件，双击运行
win-theme-toggle-mini.exe
```

## 编译环境

### 系统要求
- **操作系统**: Windows 10/11
- **Rust 工具链**: 1.70.0+
- **构建工具**: Visual Studio Build Tools

### Rust 环境搭建

#### 使用 Scoop 包管理器（推荐）

1. **安装 Scoop**（如果尚未安装）
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   iwr -useb get.scoop.sh | iex
   ```

2. **安装 Rust (MSVC 版本)**
   ```powershell
   scoop install rustup-msvc
   ```
   *注意：`rustup-msvc` 会自动配置 MSVC 工具链，无需手动设置*

3. **安装 Visual Studio Build Tools**
   - 从 [Microsoft官网](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 下载 Visual Studio Build Tools
   - 运行安装程序，选择：
     - ~~**C++ 构建工具**~~
     - **Windows 11 SDK**（或 Windows 10 SDK）
     - **MSVC v143 - VS 2022 C++ x64/x86 构建工具**

#### 备选安装方式

如果不想使用 Scoop，也可以手动安装：

1. **安装 Rust**
   ```powershell
   # 使用 rustup 安装
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh
   ```

2. **安装 Visual Studio Build Tools**
   - 同上，从官网下载安装

### Rust 项目编译

#### 标准版编译
```bash
git clone <repository>
cd win-theme-toggle/standard
cargo build --release
# 输出: target/release/win-theme-toggle.exe (~100KB)
```

#### 极简版编译
```bash
cd win-theme-toggle/mini
cargo build --release
# 输出: target/release/win-theme-toggle.exe (~3.5KB)
```

### Rust 编译优化配置

项目包含针对 Rust 的优化编译配置：
- **LTO (链接时优化)** - 减小体积并提升性能
- **Panic abort** - 移除 unwinding 信息
- **Strip symbols** - 移除调试符号
- **代码大小优化** - 使用 `opt-level = "z"`
- **目标特定优化** - 针对 x86_64-pc-windows-msvc

## 🛠️ 技术实现

### Rust 代码特点

**标准版**:
```rust
// 使用 windows-sys 库进行安全的系统调用
use windows_sys::Win32::System::Registry::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
```

**极简版**:
```rust
// 使用 #![no_std] 和直接系统调用
#![no_std]
#![no_main]
#[link(name = "advapi32")]
extern "system" { /* ... */ }
```

### 技术原理

工具通过修改以下注册表值来切换主题：
- `AppsUseLightTheme` - 控制应用主题
- `SystemUsesLightTheme` - 控制系统主题

然后发送系统广播消息通知主题变更：
- `WM_SETTINGCHANGE`
- `WM_THEMECHANGED` 
- `WM_SYSCOLORCHANGE`

## 系统要求

- **操作系统**: Windows 10 或 Windows 11
- **架构**: x64
- **权限**: 普通用户权限即可

## 常见问题

**Q: 为什么选择 Rust 开发？**
A: Rust 提供内存安全、零成本抽象和极小运行时，适合系统工具开发

**Q: 工具运行后没有效果？**
A: 确保系统支持明暗主题（Windows 10 1709+）

**Q: 杀毒软件报毒？**
A: 这是误报，Rust 编译的代码完全开源可审查

**Q: 编译时出现链接错误？**
A: 确保已安装 Visual Studio Build Tools 并重启终端

**Q: 可以创建桌面快捷方式吗？**
A: 可以，建议固定到任务栏方便快速切换

## Rust 版本选择建议

- **日常使用**: 推荐标准版，更好的兼容性和错误处理
- **追求极致体积**: 选择极简版，适合技术爱好者
- **学习 Rust**: 标准版代码更易读和理解

## 🎯 Rust 开发优势

- **内存安全**: 无内存泄漏和缓冲区溢出风险
- **零运行时**: 编译为本地代码，无额外依赖
- **跨平台潜力**: 可轻松移植到其他平台
- **性能优异**: 接近 C/C++ 的性能

## 开发致谢

本 Rust 程序在开发过程中借助了以下 AI 助手的帮助：

- **GitHub Copilot** - 对`AutoDarkMode/Windows-Auto-Night-Mode`仓库学习，Rust 复现主体功能
- **ChatGPT** - 编译环境搭建，Rust 架构设计和问题解决
- **DeepSeek** - 极简版代码构建和调试协助

感谢这些 AI 工具在 Rust 开发过程中提供的技术支持！

## 许可证

MIT License - 可自由使用、修改和分发

## 贡献

欢迎提交 Issue 和 Pull Request！特别是 Rust 相关的优化建议。

---

**注意**: 极简版虽然体积极小，但在某些特殊环境下可能存在兼容性问题。如遇到问题请换用标准版。