<p align="center">
  <img src="docs/images/poki-icon.png" width="72" height="72" alt="Poki icon">
</p>

<h1 align="center">Poki for Linux</h1>

<p align="center">
  English | <a href="README.zh-CN.md">简体中文</a> | <a href="README.zh-TW.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0--only-blue" alt="License: AGPL-3.0-only"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20iOS-lightgrey" alt="Platform: Linux and iOS">
</p>

**Control a real iPhone from an MCP agent on a Linux host — no Mac, no
jailbreak, no extra hardware.** Poki exposes the iPhone's accessibility
tree and HID input (tap, swipe, type, keys) to any MCP client through a
local server, so an agent operates the phone the way you do.

> This repository is an **unofficial Linux port** of
> [JosssphZhou/poki](https://github.com/JosssphZhou/poki). The macOS host
> support upstream is built around — Xcode, the Vision OCR helper, the
> simulator — is replaced or skipped here; everything device-facing is
> upstream's design, driven through Linux tooling instead. The detailed
> port notes live in [LINUX.md](LINUX.md). Upstream: https://poki.bot

## How it works

A Rust bridge (built on [idevice](https://github.com/jkcoxson/idevice))
talks to the iPhone over USB through **usbmuxd**, opens the CoreDevice
tunnel, and speaks RemoteXPC to the developer services — the same path
Xcode uses, without Xcode. The DeveloperDiskImage is installed by
**pymobiledevice3**, which downloads it and personalizes it for the
device. The MCP server itself is plain Node.js.

## Requirements

- A Linux host (verified on Arch Linux, x86_64) with usbmuxd running
- An iPhone with Developer Mode and UI Automation enabled
  (see [Physical iPhone setup](docs/physical-iphone.md); the device
  steps are the same as upstream)
- Node.js 24+, Rust (via rustup), Python 3, ffmpeg, git
- pymobiledevice3 (`pip install pymobiledevice3`, or a virtualenv —
  point `POKI_PYMOBILEDEVICE3` at the binary if it is not on `PATH`)

`./bin/setup` checks for missing prerequisites and says how to install
each one.

## Quick start

```sh
git clone https://github.com/carrotciallo/poki-for-linux.git
cd poki-for-linux
./bin/setup
```

`./bin/setup` builds the native bridge and the idevice tools, installs
the npm dependencies, and verifies that sharp's prebuilt binaries are
actually present (on some Linux npm setups they get skipped silently;
setup repairs that).

Prepare the iPhone once, connected and unlocked over USB:

```sh
node bin/prepare-device.mjs
```

This installs the DeveloperDiskImage through pymobiledevice3. If the
image is already installed, it reports that and does nothing.

Start the server and keep it running:

```sh
npm start
```

Register the MCP server with your client, from the same directory:

```sh
claude mcp add --scope user poki-mcp -- node "$PWD/src/stdio.mjs"   # Claude Code
codex mcp add poki-mcp -- node "$PWD/src/stdio.mjs"                 # Codex
```

State lives in `$XDG_STATE_HOME/PokiMCP` (default
`~/.local/state/PokiMCP`) unless `POKI_MCP_STATE_DIR` is set.

## Try a task

```sh
node bin/call.mjs list_devices   # Find your device UUID
node bin/call.mjs list_elements '{"device":"<device UUID>"}'
```

> **Note:** Poki controls your real iPhone. If you ask it to send
> messages or make payments, it will. Screen contents are sent to your
> configured model service. Sensitive tools ask for approval first —
> on Linux the approval dialog uses `zenity`; without it, sensitive
> tools raise an explicit error instead of silently proceeding.

**Poki Agent.** The standalone terminal agent works on Linux the same
way (`npm ci --prefix agent`, then `poki login`); see
[Run Poki Agent](agent/README.md). The iMessage channel is the one
exception: it needs a BlueBubbles server, which itself requires a Mac.
Telegram works.

**Wireless.** Remote use over Tailscale (RemotePairing) is built into
the bridge and is platform-independent; the setup steps in
[remote-iphone.md](docs/remote-iphone.md) apply, with this repository's
state directory. The Tailscale route itself is not yet verified on a
Linux host — but the same RemotePairing mechanism has been verified
here over the local network, using the iPhone's LAN address in place
of a Tailscale one, after which the cable is no longer needed at home.

## Verified on real hardware

Host: Arch Linux (x86_64), usbmuxd 1.1.1, Node.js 24.22,
pymobiledevice3 11.50. Device: iPhone18,4 running iOS 27.0.1 over USB.

| Check | Result |
| --- | --- |
| Pairing and lockdown over usbmuxd | works |
| DeveloperDiskImage via pymobiledevice3 | installed, including a reinstall through `bin/prepare-device.mjs` after an uninstall |
| `list_elements` (accessibility tree) | works |
| `take_screenshot` (lossless device capture) | works |
| `tap`, `swipe`, `press_key` | works |
| `type_text` (verified text input, clipboard restored afterwards) | works |
| Screenshot prompt check (OCR via tesseract) | works |
| Approval dialog via `zenity` | works |
| Wireless control over the LAN (RemotePairing to the LAN IP) | works |
| `npm test` | every test file passes on its own; see the note in [LINUX.md](LINUX.md) |

## Known limitations

- OCR runs on tesseract through a small Rust recognizer in the bridge
  crate (installed as `build/ocr`), printing the same JSON contract
  as the macOS Vision helper; the screenshot prompt check and the
  accessibility-miss fallback work. Recognition quality is
  tesseract's — clear UI text reads well, small or stylized text may
  be missed. Languages default to Simplified Chinese, English and
  Japanese (`POKI_OCR_LANGS`).
- `get_clipboard` cannot classify clipboard image types on Linux.
- A few physical-device tools shell out to `xcrun devicectl` upstream
  (`open_url`, `terminate_app`, crash-log access, app-container export)
  and fail on Linux until they are ported to idevice-tools equivalents.
- Physical devices support portrait orientation only, as upstream.

## Docs

| Topic | Read |
| --- | --- |
| Linux port notes (scope, differences from upstream) | [LINUX.md](LINUX.md) |
| Installation and troubleshooting | [Install guide](docs/install.md) |
| Poki Agent and channels | [Run Poki Agent](agent/README.md) |
| MCP tool reference | [Tool behavior](docs/tools.md) |
| Environment variables and sessions | [Configuration](docs/configuration.md) |
| USB pairing, Developer Mode, and Wi-Fi | [Physical iPhone setup](docs/physical-iphone.md) |
| iPhone over Tailscale | [Remote iPhone](docs/remote-iphone.md) |
| Test results and known limits | [Verification](docs/verification.md) |
| Bridge architecture | [Native bridge](native/NATIVE.md) |

Some documents under `docs/` were written for the macOS host; where
they conflict with this README or LINUX.md on host setup, the Linux
documents win.

## Disclaimer

Poki is an independent open source project. It is not affiliated with,
or endorsed by, Apple, OpenAI, or the makers of any app or website it
operates. iPhone is a trademark of Apple Inc. This port is an
unofficial fork and is not affiliated with the upstream author.

Automating third-party apps and websites may violate their terms of
service. Decide for yourself whether to use Poki with a given service.
You are responsible for what it does on your device and accounts,
including messages it sends, changes it makes, and any account
restrictions that result.

Poki is provided "as is", without warranty, as set out in the
AGPL-3.0 license.

## License

Poki is open source under the [AGPL-3.0-only license](LICENSE).

Copyright (c) 2026 Joseph Zhou and contributors. Open-source
dependencies retain their respective licenses; see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). To contribute, see
[CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgments

<p align="center">
  <img src="docs/images/acknowledgments.png" width="860" alt="Built on open source: pi, idevice, idb, BlueBubbles, Hermes Agent, MCP SDK, sharp, and FFmpeg"><br>
  <sub>Open source acknowledgments: <a href="https://github.com/earendil-works/pi">pi</a> | <a href="https://github.com/jkcoxson/idevice">idevice</a> | <a href="https://github.com/facebook/idb">idb</a> | <a href="https://github.com/BlueBubblesApp/bluebubbles-server">BlueBubbles</a> | <a href="https://github.com/NousResearch/hermes-agent">Hermes Agent</a> | <a href="https://github.com/modelcontextprotocol/typescript-sdk">MCP SDK</a> | <a href="https://github.com/lovell/sharp">sharp</a> | <a href="https://github.com/FFmpeg/FFmpeg">FFmpeg</a></sub>
</p>
