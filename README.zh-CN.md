<p align="center">
  <img src="docs/images/poki-icon.png" width="72" height="72" alt="Poki 图标">
</p>

<h1 align="center">Poki for Linux</h1>

<p align="center">
  <a href="README.md">English</a> | 简体中文 | <a href="README.zh-TW.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0--only-blue" alt="许可证: AGPL-3.0-only"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20iOS-lightgrey" alt="平台: Linux 与 iOS">
</p>

**在 Linux 主机上用 MCP 智能体操控真实的 iPhone——不需要 Mac、不越狱、不需要额外硬件。** Poki 通过本地服务器把 iPhone 的无障碍树和 HID 输入（点按、滑动、打字、按键）开放给任意 MCP 客户端，让智能体像你一样操作手机。

> 本仓库是 [JosssphZhou/poki](https://github.com/JosssphZhou/poki) 的**非官方 Linux 移植**。上游赖以工作的 macOS 主机支持——Xcode、Vision OCR 辅助程序、模拟器——在这里被替换或跳过；所有设备侧能力都是上游的设计，只是改由 Linux 工具链驱动。移植的详细说明见 [LINUX.md](LINUX.md)。上游官网：https://poki.bot

## 工作原理

一个用 Rust 写成、基于 [idevice](https://github.com/jkcoxson/idevice) 的桥接程序通过 **usbmuxd** 经 USB 与 iPhone 通信，打开 CoreDevice 隧道，再通过 RemoteXPC 与开发者服务对话——这正是 Xcode 用的那条路径，只是不需要 Xcode。开发者磁盘镜像由 **pymobiledevice3** 安装，它负责下载镜像并为设备做个性化签名。MCP 服务器本身则是纯 Node.js 程序。

## 环境要求

- 一台 Linux 主机（已在 Arch Linux x86_64 上验证），且 usbmuxd 正在运行
- 一部已开启开发者模式和 UI 自动化的 iPhone（见[物理 iPhone 设置](docs/physical-iphone.md)，设备侧步骤与上游相同）
- Node.js 24+、Rust（通过 rustup 安装）、Python 3、ffmpeg、git
- pymobiledevice3（`pip install pymobiledevice3`，或装在虚拟环境里——如果它不在 `PATH` 中，用 `POKI_PYMOBILEDEVICE3` 指明其路径）

`./bin/setup` 会检查缺失的前置依赖，并告诉你每一项该怎么装。

## 快速开始

```sh
git clone https://github.com/carrotciallo/poki-for-linux.git
cd poki-for-linux
./bin/setup
```

`./bin/setup` 会构建原生桥接程序和 idevice 工具、安装 npm 依赖，并验证 sharp 的预编译二进制确实存在（在某些 Linux 的 npm 配置下它们会被悄悄跳过，setup 会修复这个问题）。

用 USB 连接已解锁的 iPhone，做一次性的设备准备：

```sh
node bin/prepare-device.mjs
```

它会通过 pymobiledevice3 安装开发者磁盘镜像；如果镜像已安装，它只会报告这一情况，不做任何事。

启动服务器并保持运行：

```sh
npm start
```

在同一目录下，把 MCP 服务器注册到你的客户端：

```sh
claude mcp add --scope user poki-mcp -- node "$PWD/src/stdio.mjs"   # Claude Code
codex mcp add poki-mcp -- node "$PWD/src/stdio.mjs"                 # Codex
```

状态数据存放在 `$XDG_STATE_HOME/PokiMCP`（默认 `~/.local/state/PokiMCP`），除非设置了 `POKI_MCP_STATE_DIR`。

## 试一个任务

```sh
node bin/call.mjs list_devices   # 找到你的设备 UUID
node bin/call.mjs list_elements '{"device":"<device UUID>"}'
```

> **注意：** Poki 操控的是你的真实 iPhone。你让它发消息或付款，它真会执行。屏幕内容会被发送到你配置的模型服务。敏感工具会先征求批准——在 Linux 上批准对话框走 `zenity`；没有 zenity 时，敏感工具会显式报错，而不是悄无声息地放行。

**Poki Agent。** 独立终端智能体在 Linux 上同样可用（`npm ci --prefix agent`，然后 `poki login`），见[运行 Poki Agent](agent/README.md)。唯一的例外是 iMessage 通道：它需要 BlueBubbles 服务器，而后者本身需要一台 Mac。Telegram 通道则没有问题。

**无线。** 通过 Tailscale 的远程使用（RemotePairing）已内建于桥接程序、与平台无关；[remote-iphone.md](docs/remote-iphone.md) 中的步骤同样适用，只需换成本仓库的状态目录。Tailscale 这条路线本身尚未在 Linux 主机上实测——不过同一套 RemotePairing 机制已在这里通过局域网验证（用 iPhone 的局域网地址代替 Tailscale 地址），此后在家使用就不再需要数据线。

## 真机验证情况

主机：Arch Linux（x86_64），usbmuxd 1.1.1，Node.js 24.22，pymobiledevice3 11.50。设备：iPhone18,4，运行 iOS 27.0.1，USB 连接。

| 检查项 | 结果 |
| --- | --- |
| 通过 usbmuxd 配对与 lockdown 通信 | 可用 |
| 通过 pymobiledevice3 安装开发者磁盘镜像 | 已安装，包括卸载后由 `bin/prepare-device.mjs` 完整重装 |
| `list_elements`（无障碍树） | 可用 |
| `take_screenshot`（无损设备截图） | 可用 |
| `tap`、`swipe`、`press_key` | 可用 |
| `type_text`（带校验的文本输入，完成后恢复原剪贴板） | 可用 |
| 通过 `zenity` 的批准对话框 | 可用 |
| 局域网无线操控（RemotePairing 到局域网 IP） | 可用 |
| `npm test` | 每个测试文件单独运行均通过；说明见 [LINUX.md](LINUX.md) |

## 已知限制

- OCR 由 tesseract 通过 bridge 中的一个小型 Rust 识别器承担（安装为 `build/ocr`），输出与 macOS Vision 辅助程序相同的 JSON 契约；截图提示检查与无障碍缺失时的回退都可工作。识别质量取决于 tesseract：清晰的界面文字读得准，小字或艺术字可能漏读。默认语言为简体中文、英文与日文，可用 `POKI_OCR_LANGS` 调整。
- `get_clipboard` 在 Linux 上无法对剪贴板图片分类。
- 少数物理设备工具在上游是调用 `xcrun devicectl` 的（`open_url`、`terminate_app`、崩溃日志访问、应用容器导出），在移植到 idevice-tools 等价实现之前，它们在 Linux 上会失败。
- 与上游相同，物理设备仅支持竖屏方向。

## 文档

| 主题 | 阅读 |
| --- | --- |
| Linux 移植说明（范围、与上游的差异） | [LINUX.md](LINUX.md) |
| 安装与故障排查 | [安装指南](docs/install.md) |
| Poki Agent 与通道 | [运行 Poki Agent](agent/README.md) |
| MCP 工具参考 | [工具行为](docs/tools.md) |
| 环境变量与会话 | [配置](docs/configuration.md) |
| USB 配对、开发者模式与 Wi-Fi | [物理 iPhone 设置](docs/physical-iphone.md) |
| 通过 Tailscale 使用 iPhone | [远程 iPhone](docs/remote-iphone.md) |
| 测试结果与已知限制 | [验证](docs/verification.md) |
| 桥接架构 | [原生桥接](native/NATIVE.md) |

`docs/` 下的部分文档是为 macOS 主机写的；其中关于主机安装的内容若与本文档或 LINUX.md 冲突，以 Linux 文档为准。

## 免责声明

Poki 是一个独立的开源项目，与 Apple、OpenAI 以及它所操作的任何应用或网站的厂商均无隶属关系，也未获其背书。iPhone 是 Apple Inc. 的商标。本移植是非官方 fork，与上游作者无隶属关系。

自动化操作第三方应用和网站可能违反其服务条款。是否将 Poki 用于某个服务，请自行判断。你需要为它在你的设备和账户上做的一切负责，包括它发出的消息、做出的更改，以及由此产生的任何账户限制。

Poki 按 AGPL-3.0 许可证以"现状"提供，不附任何担保。

## 许可证

Poki 以 [AGPL-3.0-only 许可证](LICENSE)开源。

版权所有 (c) 2026 Joseph Zhou 及贡献者。各开源依赖保留其各自的许可证，见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。贡献方式见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 致谢

<p align="center">
  <img src="docs/images/acknowledgments.png" width="860" alt="构建于开源之上：pi、idevice、idb、BlueBubbles、Hermes Agent、MCP SDK、sharp 与 FFmpeg"><br>
  <sub>开源致谢：<a href="https://github.com/earendil-works/pi">pi</a> | <a href="https://github.com/jkcoxson/idevice">idevice</a> | <a href="https://github.com/facebook/idb">idb</a> | <a href="https://github.com/BlueBubblesApp/bluebubbles-server">BlueBubbles</a> | <a href="https://github.com/NousResearch/hermes-agent">Hermes Agent</a> | <a href="https://github.com/modelcontextprotocol/typescript-sdk">MCP SDK</a> | <a href="https://github.com/lovell/sharp">sharp</a> | <a href="https://github.com/FFmpeg/FFmpeg">FFmpeg</a></sub>
</p>
