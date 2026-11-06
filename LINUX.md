# Poki MCP on Linux (unofficial port)

This fork adds Linux host support for **physical iPhones** to
[Poki](https://github.com/JosssphZhou/poki). Upstream supports macOS only;
the device protocol stack it is built on (usbmuxd, the CoreDeviceProxy
software tunnel, RemoteXPC/RSD, the accessibility audit service and
CoreDevice HID) is platform-independent, so the port is a set of small
host-side adaptations, not a rewrite. The port tracks upstream `main`
(it is based on the v0.1.1 development line) and stays AGPL-3.0-only,
like upstream.

## Scope

Works on Linux:

- The MCP daemon and all physical-device tools that go through the Rust
  bridge or idevice-tools: device discovery, the accessibility element
  tree (`list_elements`), screenshots and recording, taps and gestures,
  typing, app list/launch, and the rest of the bridge-backed tool set
- Remote connections (the upstream Tailscale/RemotePairing path lives in
  the bridge and is not macOS-specific)
- The Telegram and terminal channels of Poki Agent

Not supported on Linux:

- The iOS Simulator (there is no iOS Simulator for Linux)
- The iMessage channel (its gateway is a macOS app)
- `build_and_run` (building an iOS app requires Xcode)

## Requirements

- Linux with usbmuxd installed and running (Arch: `pacman -S usbmuxd`;
  Debian/Ubuntu: `apt install usbmuxd`)
- Node.js 24 or newer, the Rust toolchain, FFmpeg, git, python3
- [pymobiledevice3](https://github.com/doronz88/pymobiledevice3)
  (`pip install pymobiledevice3`) for the one-time DeveloperDiskImage
  installation — Linux has no Xcode to take the image from
- Optional: `zenity`, used for the human-approval dialogs of sensitive
  tools

## Setup

```sh
./bin/setup            # checks prerequisites, fetches the pinned idevice
                       # checkout, builds idevice-tools, installs npm
                       # dependencies, builds the native bridge
```

Then prepare the iPhone (once):

1. Connect it by USB, unlock it, and tap **Trust** on the pairing prompt.
2. Turn on **Developer Mode** (Settings → Privacy & Security →
   Developer Mode). On a device with a passcode this can only be done on
   the device itself; `pymobiledevice3 amfi reveal-developer-mode` makes
   the toggle appear if it is missing. The device restarts and asks for
   the passcode once more. Also enable **UI Automation** under
   Settings → Developer.
3. Install the DeveloperDiskImage:

   ```sh
   node bin/prepare-device.mjs
   ```

   On Linux this delegates to `pymobiledevice3 cryptex auto-install`,
   which downloads the image, has Apple personalize it for the device,
   and installs it. Set `POKI_PYMOBILEDEVICE3` if the binary is not on
   `PATH`. Run it with a single iPhone connected.

Run the daemon the same way as on macOS (`npm start`, or
`node bin/call.mjs <tool>` for one-off calls). State lives in
`$XDG_STATE_HOME/PokiMCP` (default `~/.local/state/PokiMCP`) unless
`POKI_MCP_STATE_DIR` is set.

## Verified on real hardware

Host: Arch Linux (x86_64), usbmuxd 1.1.1, Node.js 24.22, pymobiledevice3
11.50. Device: iPhone18,4 running iOS 27.0.1, USB connection.

| Check | Result |
| --- | --- |
| Pairing and lockdown over usbmuxd | works |
| DeveloperDiskImage via pymobiledevice3 (`MobileAsset.DDI 27.1.5228.8`) | installed, including a reinstall through `bin/prepare-device.mjs` after an uninstall |
| `list_elements` (accessibility tree, 420×912 pt) | works |
| `take_screenshot` (lossless device capture) | works |
| `tap` (open and close an app sidebar, verified by tree changes) | works |
| Approval dialog via `zenity` (dialog timeout leaves the action unapproved) | works |
| Wireless control over the LAN (RemotePairing to the iPhone's LAN address; live TCP tunnel, tree and screenshot over Wi-Fi) | works |
| `type_text`, `swipe`, `press_key` on the real device | works |
| `npm test` | every test file passes on its own; see the note below |

Test-suite note: the full suite also needs the agent dependencies
(`npm ci --prefix agent`), which the update tests exercise through the
CLI. In a single full `npm test` run on the reference host, 9
timing-sensitive tests in `test/protocol.test.mjs` (app management and
the approval dialog) failed while daemon fixtures from the earlier
files were still winding down; the same file passes 16/16 when run on
its own. The other files passed inside the full run as well.

## Known limitations

- The OCR helper is a macOS Vision binary, so on Linux the screenshot
  prompt check reports `unavailable` and observation relies on the
  accessibility tree (the same fallback upstream uses when the helper is
  missing).
- `get_clipboard` cannot classify clipboard image types on Linux; that
  check uses the macOS simulator helper upstream.
- A few physical-device tools shell out to `xcrun devicectl` upstream
  (`open_url`, `terminate_app`, crash-log access, app-container export)
  and fail on Linux until they are ported to idevice-tools equivalents.
- Human approval for sensitive tools uses a `zenity` question dialog.
  Without zenity the tools raise an explicit error instead of silently
  proceeding; `POKI_MCP_APPROVAL_MODE=allow` remains available for hosts
  where unapproved sensitive actions are acceptable.
- On some Linux npm setups, `npm ci` silently skips sharp's prebuilt
  `@img` packages and sharp then fails trying to build from source.
  `bin/setup` installs with `--ignore-scripts` on Linux and runs
  `bin/setup-linux-deps.mjs`, which verifies that sharp loads and
  installs the pinned `@img` packages explicitly when it does not.

## Changes from upstream

- `bin/setup`: Linux prerequisite flow (no Xcode requirement; requires
  usbmuxd), prerequisite hints worded per platform, shared idevice/Node
  steps, script-free npm install plus the sharp verification above;
  the setup test in `test/protocol.test.mjs` asserts each platform's
  own expectations
- `bin/build-native`: builds only the Rust bridge on Linux; the Swift
  OCR and Objective-C simulator helpers are macOS-only and skipped
- `bin/prepare-device.mjs`: on Linux, installs the DeveloperDiskImage
  through pymobiledevice3 instead of the Xcode `iOS_DDI` directory
- `src/config.mjs`: default state directory follows the XDG state
  convention on Linux (macOS path unchanged)
- `src/human.mjs`: approval dialogs fall back to `zenity` on Linux

Everything else is unmodified upstream code. Issues that reproduce on
macOS belong upstream; Linux-specific issues belong here.
