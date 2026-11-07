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
  installation — Linux has no Xcode to take the image from — and, at
  runtime, for the device services behind `terminate_app`, crash-log
  access and app-container export, which macOS reaches through
  `xcrun devicectl`
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

## Running the daemon as a user service

On a host that should keep Poki available, run the daemon under
systemd's user manager instead of a terminal. Save as
`~/.config/systemd/user/poki-mcp.service`:

```ini
[Unit]
Description=Poki MCP daemon
After=network.target

[Service]
Type=simple
WorkingDirectory=/path/to/poki-for-linux
ExecStart=/usr/bin/node /path/to/poki-for-linux/src/daemon.mjs
Restart=on-failure

[Install]
WantedBy=default.target
```

Then:

```sh
systemctl --user daemon-reload
systemctl --user enable --now poki-mcp.service
loginctl enable-linger "$USER"   # keep it running without an open session
```

If Node.js lives somewhere else (a version manager, a private
prefix), point `ExecStart` at that binary. Set `POKI_MCP_STATE_DIR`
with an `Environment=` line in the unit when the state directory
should differ from the default — and pass the same value to every
client, as described next.

The Linux device-service paths (`terminate_app`, crash reports,
app-container export) shell out to `pymobiledevice3`. When it is
installed in a virtualenv it is not on the service's `PATH`, and
those tools fail with `spawn pymobiledevice3 ENOENT`. Point the
service at the binary with a drop-in
(`~/.config/systemd/user/poki-mcp.service.d/pymobiledevice3.conf`):

```ini
[Service]
Environment=POKI_PYMOBILEDEVICE3=/path/to/venv/bin/pymobiledevice3
```

## Connecting an MCP client

The daemon serves HTTP on a random loopback port with a random
bearer token, and rewrites both on every start in `mcp.json` inside
the state directory. **Never copy that URL or token into a client
configuration** — it is stale the moment the daemon restarts.
Clients connect through the stdio bridge in this repository,
`src/stdio.mjs`, which reads the current endpoint file on startup
and re-reads it when the daemon rejects a stale endpoint (401) or
is unreachable, so a daemon restart needs no client-side change.

Claude Code:

```sh
claude mcp add --scope user poki-mcp -- node /path/to/poki-for-linux/src/stdio.mjs
```

Pi (an entry in `~/.pi/agent/mcp.json`):

```json
{
  "mcpServers": {
    "poki": {
      "command": "node",
      "args": ["/path/to/poki-for-linux/src/stdio.mjs"],
      "cwd": "/path/to/poki-for-linux"
    }
  }
}
```

With a non-default state directory, give the bridge the same
`POKI_MCP_STATE_DIR` the daemon runs with: the `--env` flag for
`claude mcp add`, or an `env` object next to `args` for Pi.

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
| `terminate_app`, `list_crashes`, `get_crash` over USB (pymobiledevice3 paths) | works, including the `already_stopped` result for an app that is not running |
| `install_app` / `uninstall_app` over USB with a locally signed (free provisioning) package | works; the installed app launches once the user trusts the developer certificate under Settings → General → VPN & Device Management |
| First device call after a daemon restart (RemotePairing target, startup warm-up) | works; a screenshot returns in seconds instead of paying session setup inline |
| Screenshot prompt check through OCR (tesseract recognizer as `build/ocr`; reports `checked`, and flows that force an OCR read such as `hide_keyboard` work) | works |
| `npm test` | every test file passes on its own; see the note below |

Test-suite note: the full suite also needs the agent dependencies
(`npm ci --prefix agent`), which the update tests exercise through the
CLI. Every test file passes on the reference host (136 tests in
total). An earlier revision of this note blamed 9 failing
coordination tests on timing; they were in fact two porting gaps —
the app-management paths called the macOS-only `plutil` and `ditto`,
and the test fixture did not stub `zenity` — both fixed since.

## Known limitations

- OCR recognition quality on Linux is tesseract's, not Vision's: clear
  UI text reads well, small or stylized text may be missed, so the
  accessibility tree stays the primary observation source (the same
  ordering upstream uses).
- `get_clipboard` cannot classify clipboard image types on Linux; that
  check uses the macOS simulator helper upstream.
- `open_url` has no Linux implementation on any transport: upstream
  it calls `xcrun devicectl`, and neither pymobiledevice3 (lockdown,
  DVT and CoreDevice command groups) nor the idevice tools expose an
  open-URL service.
- RemotePairing carries only the bridge-backed tools. For a remote
  target, poki's transport gate rejects the platform tools —
  `terminate_app`, `open_url`, `get_clipboard`, crash-log access,
  app-container export, install/uninstall, settings, location —
  before any host tool runs, on every host OS. The pymobiledevice3
  paths in this port therefore cover USB connections; over
  RemotePairing those tools stay unavailable, even though
  pymobiledevice3 could reach several of the same services over a
  tunnel of its own (it would need its own RemotePairing pairing
  record next to poki's, which this port does not set up). Clipboard
  writes are not affected: the bridge sets and restores the
  pasteboard inside its own input path; only the standalone
  `get_clipboard` read is gated.
- Crash-report listings on Linux come from pymobiledevice3, which
  reports paths only: entries carry no modification time or size,
  and ordering falls back to the report names, which embed their
  timestamps.
- `download_app_container` reads the data container through the
  house-arrest service, which only vends containers of
  development-signed apps; the device refuses App Store apps
  (`InstallationLookupFailed`), the same restriction devicectl
  container copies have on macOS. The export pulls the container's
  `Documents`, `Library` and `tmp` directories one by one: the
  container-manager metadata file at the container root is
  protected (`PERM_DENIED`), so pulling the root itself always
  aborts. `SystemData` is system-managed and the service denies
  reading it, so it is not part of the export.
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
  usbmuxd, tesseract, and libplist's `plistutil` plus zip/unzip),
  prerequisite hints worded per platform, shared idevice/Node
  steps, script-free npm install plus the sharp verification above;
  the setup test in `test/protocol.test.mjs` asserts each platform's
  own expectations
- `bin/build-native`: builds the Rust bridge on Linux, including the
  Rust OCR recognizer installed as `build/ocr`; the Swift OCR and
  Objective-C simulator helpers are macOS-only and skipped
- `native/bridge/src/bin/ocr.rs` (new): a tesseract-backed recognizer
  printing the same JSON contract as `native/ocr.swift` — word rows
  grouped into screen lines, narrow captures upscaled first;
  `POKI_OCR_LANGS`, `POKI_OCR_PSM` and `POKI_OCR_UPSCALE` tune it
  (defaults `chi_sim+eng+jpn`, `11`, `2`)
- `bin/prepare-device.mjs`: on Linux, installs the DeveloperDiskImage
  through pymobiledevice3 instead of the Xcode `iOS_DDI` directory
- `src/config.mjs`: default state directory follows the XDG state
  convention on Linux (macOS path unchanged)
- `src/human.mjs`: approval dialogs fall back to `zenity` on Linux
- `src/platform.mjs` and `src/work.mjs`: app-package and property-list
  work uses `plistutil` and zip/unzip on Linux in place of the
  macOS-only `plutil` and `ditto` (the macOS tools are unchanged);
  `execute()` passes a working directory through for the zip call
- `src/platform.mjs`: the physical-device paths upstream drives
  through `xcrun devicectl` — process termination, crash-report
  listing and download, app-container export — use pymobiledevice3
  on Linux instead (DVT process control; the crash-report and
  app-container AFC services), located through
  `POKI_PYMOBILEDEVICE3` as in `bin/prepare-device.mjs`; the macOS
  calls are unchanged, and `open_url` has no equivalent and stays
  on devicectl
- `src/remote.mjs` and `src/daemon.mjs`: the daemon warms each
  configured RemotePairing helper at startup with a `device_info`
  request, so the first device call no longer pays for helper start
  and session setup inline (previously that cost landed inside the
  first call and could outlast a client's request timeout)
- `test/fixture.mjs`: the fixture stubs `zenity` alongside `osascript`,
  so the approval-dialog flow is exercised on the Linux branch too;
  it also stubs `pymobiledevice3`, so the coordination suite covers
  the Linux device-service paths above
- `.github/workflows/tests.yml` (new): CI on Ubuntu — the full
  `./bin/setup`, the agent dependencies, then each test file in its
  own process

Everything else is unmodified upstream code. Issues that reproduce on
macOS belong upstream; Linux-specific issues belong here.
