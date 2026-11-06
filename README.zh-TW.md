<p align="center">
  <img src="docs/images/poki-icon.png" width="72" height="72" alt="Poki 圖示">
</p>

<h1 align="center">Poki for Linux</h1>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a> | 繁體中文 | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0--only-blue" alt="授權: AGPL-3.0-only"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20iOS-lightgrey" alt="平台: Linux 與 iOS">
</p>

**在 Linux 主機上用 MCP 智能體操控真實的 iPhone——不需要 Mac、不越獄、不需要額外硬體。** Poki 透過本機伺服器把 iPhone 的無障礙樹和 HID 輸入（點按、滑動、打字、按鍵）開放給任意 MCP 用戶端，讓智能體像你一樣操作手機。

> 本倉庫是 [JosssphZhou/poki](https://github.com/JosssphZhou/poki) 的**非官方 Linux 移植**。上游賴以運作的 macOS 主機支援——Xcode、Vision OCR 輔助程式、模擬器——在這裡被替換或略過；所有裝置端能力都是上游的設計，只是改由 Linux 工具鏈驅動。移植的詳細說明見 [LINUX.md](LINUX.md)。上游官網：https://poki.bot

## 工作原理

一個以 Rust 撰寫、基於 [idevice](https://github.com/jkcoxson/idevice) 的橋接程式透過 **usbmuxd** 經 USB 與 iPhone 通訊，打開 CoreDevice 隧道，再透過 RemoteXPC 與開發者服務對話——這正是 Xcode 用的那條路徑，只是不需要 Xcode。開發者磁碟映像由 **pymobiledevice3** 安裝，它負責下載映像並為裝置做個人化簽名。MCP 伺服器本身則是純 Node.js 程式。

## 環境要求

- 一台 Linux 主機（已在 Arch Linux x86_64 上驗證），且 usbmuxd 正在執行
- 一部已開啟開發者模式和 UI 自動化的 iPhone（見[實體 iPhone 設定](docs/physical-iphone.md)，裝置端步驟與上游相同）
- Node.js 24+、Rust（透過 rustup 安裝）、Python 3、ffmpeg、git
- pymobiledevice3（`pip install pymobiledevice3`，或裝在虛擬環境裡——若它不在 `PATH` 中，用 `POKI_PYMOBILEDEVICE3` 指明其路徑）

`./bin/setup` 會檢查缺少的前置依賴，並告訴你每一項該如何安裝。

## 快速開始

```sh
git clone https://github.com/carrotciallo/poki-for-linux.git
cd poki-for-linux
./bin/setup
```

`./bin/setup` 會建置原生橋接程式和 idevice 工具、安裝 npm 依賴，並驗證 sharp 的預編譯二進位確實存在（在某些 Linux 的 npm 設定下它們會被悄悄略過，setup 會修復這個問題）。

用 USB 連接已解鎖的 iPhone，做一次性的裝置準備：

```sh
node bin/prepare-device.mjs
```

它會透過 pymobiledevice3 安裝開發者磁碟映像；若映像已安裝，它只會回報此情況，不做任何事。

啟動伺服器並保持執行：

```sh
npm start
```

在同一目錄下，把 MCP 伺服器註冊到你的用戶端：

```sh
claude mcp add --scope user poki-mcp -- node "$PWD/src/stdio.mjs"   # Claude Code
codex mcp add poki-mcp -- node "$PWD/src/stdio.mjs"                 # Codex
```

狀態資料存放於 `$XDG_STATE_HOME/PokiMCP`（預設 `~/.local/state/PokiMCP`），除非設定了 `POKI_MCP_STATE_DIR`。

## 試一個任務

```sh
node bin/call.mjs list_devices   # 找到你的裝置 UUID
node bin/call.mjs list_elements '{"device":"<device UUID>"}'
```

> **注意：** Poki 操控的是你的真實 iPhone。你讓它傳訊息或付款，它真的會執行。畫面內容會被傳送到你設定的模型服務。敏感工具會先徵求核准——在 Linux 上核准對話框走 `zenity`；沒有 zenity 時，敏感工具會明確報錯，而不是悄無聲息地放行。

**Poki Agent。** 獨立終端智能體在 Linux 上同樣可用（`npm ci --prefix agent`，然後 `poki login`），見[執行 Poki Agent](agent/README.md)。唯一的例外是 iMessage 通道：它需要 BlueBubbles 伺服器，而後者本身需要一台 Mac。Telegram 通道則沒有問題。

**無線。** 透過 Tailscale 的遠端使用（RemotePairing）已內建於橋接程式、與平台無關；[remote-iphone.md](docs/remote-iphone.md) 中的步驟同樣適用，只需換成本倉庫的狀態目錄。Tailscale 這條路線本身尚未在 Linux 主機上實測——不過同一套 RemotePairing 機制已在這裡透過區域網路驗證（用 iPhone 的區網位址代替 Tailscale 位址），此後在家使用就不再需要傳輸線。

## 真機驗證情況

主機：Arch Linux（x86_64），usbmuxd 1.1.1，Node.js 24.22，pymobiledevice3 11.50。裝置：iPhone18,4，執行 iOS 27.0.1，USB 連接。

| 檢查項 | 結果 |
| --- | --- |
| 透過 usbmuxd 配對與 lockdown 通訊 | 可用 |
| 透過 pymobiledevice3 安裝開發者磁碟映像 | 已安裝，包括解除安裝後由 `bin/prepare-device.mjs` 完整重裝 |
| `list_elements`（無障礙樹） | 可用 |
| `take_screenshot`（無損裝置截圖） | 可用 |
| `tap`、`swipe`、`press_key` | 可用 |
| `type_text`（附驗證的文字輸入，完成後還原原剪貼簿） | 可用 |
| 透過 `zenity` 的核准對話框 | 可用 |
| 區網無線操控（RemotePairing 到區網 IP） | 可用 |
| `npm test` | 每個測試檔案單獨執行均通過；說明見 [LINUX.md](LINUX.md) |

## 已知限制

- OCR 由 tesseract 透過 bridge 中的一個小型 Rust 辨識器承擔（安裝為 `build/ocr`），輸出與 macOS Vision 輔助程式相同的 JSON 契約；截圖提示檢查與無障礙缺失時的回退都可運作。辨識品質取決於 tesseract：清晰的介面文字讀得準，小字或藝術字可能漏讀。預設語言為簡體中文、英文與日文，可用 `POKI_OCR_LANGS` 調整。
- `get_clipboard` 在 Linux 上無法對剪貼簿圖片分類。
- 少數實體裝置工具在上游是呼叫 `xcrun devicectl` 的（`open_url`、`terminate_app`、當機記錄存取、應用程式容器匯出），在移植到 idevice-tools 等價實作之前，它們在 Linux 上會失敗。
- 與上游相同，實體裝置僅支援直向方向。

## 文件

| 主題 | 閱讀 |
| --- | --- |
| Linux 移植說明（範圍、與上游的差異） | [LINUX.md](LINUX.md) |
| 安裝與疑難排解 | [安裝指南](docs/install.md) |
| Poki Agent 與通道 | [執行 Poki Agent](agent/README.md) |
| MCP 工具參考 | [工具行為](docs/tools.md) |
| 環境變數與工作階段 | [設定](docs/configuration.md) |
| USB 配對、開發者模式與 Wi-Fi | [實體 iPhone 設定](docs/physical-iphone.md) |
| 透過 Tailscale 使用 iPhone | [遠端 iPhone](docs/remote-iphone.md) |
| 測試結果與已知限制 | [驗證](docs/verification.md) |
| 橋接架構 | [原生橋接](native/NATIVE.md) |

`docs/` 下的部分文件是為 macOS 主機所寫；其中關於主機安裝的內容若與本文件或 LINUX.md 衝突，以 Linux 文件為準。

## 免責聲明

Poki 是一個獨立的開源專案，與 Apple、OpenAI 以及它所操作的任何應用程式或網站的廠商均無隸屬關係，也未獲其背書。iPhone 是 Apple Inc. 的商標。本移植是非官方 fork，與上游作者無隸屬關係。

自動化操作第三方應用程式和網站可能違反其服務條款。是否將 Poki 用於某個服務，請自行判斷。你需要為它在你的裝置和帳戶上做的一切負責，包括它發出的訊息、做出的變更，以及由此產生的任何帳戶限制。

Poki 按 AGPL-3.0 授權以「現狀」提供，不附任何擔保。

## 授權

Poki 以 [AGPL-3.0-only 授權](LICENSE)開源。

版權所有 (c) 2026 Joseph Zhou 及貢獻者。各開源依賴保留其各自的授權，見 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。貢獻方式見 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 致謝

<p align="center">
  <img src="docs/images/acknowledgments.png" width="860" alt="建構於開源之上：pi、idevice、idb、BlueBubbles、Hermes Agent、MCP SDK、sharp 與 FFmpeg"><br>
  <sub>開源致謝：<a href="https://github.com/earendil-works/pi">pi</a> | <a href="https://github.com/jkcoxson/idevice">idevice</a> | <a href="https://github.com/facebook/idb">idb</a> | <a href="https://github.com/BlueBubblesApp/bluebubbles-server">BlueBubbles</a> | <a href="https://github.com/NousResearch/hermes-agent">Hermes Agent</a> | <a href="https://github.com/modelcontextprotocol/typescript-sdk">MCP SDK</a> | <a href="https://github.com/lovell/sharp">sharp</a> | <a href="https://github.com/FFmpeg/FFmpeg">FFmpeg</a></sub>
</p>
