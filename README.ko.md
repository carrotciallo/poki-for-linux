<p align="center">
  <img src="docs/images/poki-icon.png" width="72" height="72" alt="Poki 아이콘">
</p>

<h1 align="center">Poki for Linux</h1>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a> | <a href="README.zh-TW.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | 한국어 | <a href="README.es.md">Español</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0--only-blue" alt="라이선스: AGPL-3.0-only"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20iOS-lightgrey" alt="플랫폼: Linux 및 iOS">
</p>

**Linux 호스트에서 MCP 에이전트로 실제 iPhone을 조종하세요. Mac도, 탈옥도, 추가 하드웨어도 필요 없습니다.** Poki는 로컬 서버를 통해 iPhone의 접근성 트리와 HID 입력(탭, 스와이프, 타이핑, 키 입력)을 모든 MCP 클라이언트에 공개하여, 에이전트가 당신처럼 전화기를 조작할 수 있게 합니다.

> 이 저장소는 [JosssphZhou/poki](https://github.com/JosssphZhou/poki)의 **비공식 Linux 포팅**입니다. 업스트림이 기반으로 삼는 macOS 호스트 지원(Xcode, Vision OCR 헬퍼, 시뮬레이터)은 여기에서 교체되거나 건너뜁니다. 기기 측 기능은 모두 업스트림의 설계 그대로이며, 구동하는 툴체인만 Linux용으로 바뀌었습니다. 포팅에 대한 자세한 내용은 [LINUX.md](LINUX.md)를 참조하세요. 업스트림 웹사이트: https://poki.bot

## 동작 방식

Rust로 작성되고 [idevice](https://github.com/jkcoxson/idevice)를 기반으로 한 브리지가 **usbmuxd**를 통해 USB로 연결된 iPhone과 통신하며, CoreDevice 터널을 열고 RemoteXPC로 개발자 서비스와 대화합니다. Xcode가 사용하는 바로 그 경로이지만 Xcode는 필요 없습니다. DeveloperDiskImage는 **pymobiledevice3**가 설치하며, 이미지를 다운로드하고 기기에 맞게 개인화합니다. MCP 서버 자체는 순수한 Node.js 프로그램입니다.

## 요구 사항

- usbmuxd가 실행 중인 Linux 호스트(Arch Linux x86_64에서 검증됨)
- 개발자 모드와 UI 자동화가 켜진 iPhone([실물 iPhone 설정](docs/physical-iphone.md) 참조. 기기 측 절차는 업스트림과 같습니다)
- Node.js 24+, Rust(rustup을 통해), Python 3, ffmpeg, git
- pymobiledevice3(`pip install pymobiledevice3`, 또는 가상환경에 설치. `PATH`에 없으면 `POKI_PYMOBILEDEVICE3`로 위치를 지정하세요)

`./bin/setup`은 누락된 필수 항목을 검사하고 각각의 설치 방법을 알려줍니다.

## 빠른 시작

```sh
git clone https://github.com/carrotciallo/poki-for-linux.git
cd poki-for-linux
./bin/setup
```

`./bin/setup`은 네이티브 브리지와 idevice 도구를 빌드하고, npm 의존성을 설치하며, sharp의 사전 빌드된 바이너리가 실제로 존재하는지 검증합니다(일부 Linux npm 설정에서는 이들이 조용히 누락되는데, setup이 이를 복구합니다).

USB로 잠금 해제된 iPhone을 연결하고 일회성 기기 준비를 수행합니다.

```sh
node bin/prepare-device.mjs
```

pymobiledevice3를 통해 DeveloperDiskImage를 설치합니다. 이미 설치되어 있으면 그 사실만 보고하고 아무것도 하지 않습니다.

서버를 시작하고 계속 실행합니다.

```sh
npm start
```

같은 디렉터리에서 MCP 서버를 클라이언트에 등록합니다.

```sh
claude mcp add --scope user poki-mcp -- node "$PWD/src/stdio.mjs"   # Claude Code
codex mcp add poki-mcp -- node "$PWD/src/stdio.mjs"                 # Codex
```

상태 데이터는 `POKI_MCP_STATE_DIR`가 설정되지 않은 한 `$XDG_STATE_HOME/PokiMCP`(기본값 `~/.local/state/PokiMCP`)에 저장됩니다.

## 작업 시도하기

```sh
node bin/call.mjs list_devices   # 기기의 UUID 찾기
node bin/call.mjs list_elements '{"device":"<device UUID>"}'
```

> **참고:** Poki는 당신의 실제 iPhone을 조종합니다. 메시지를 보내거나 결제하라고 하면 실제로 실행합니다. 화면 내용은 당신이 설정한 모델 서비스로 전송됩니다. 민감한 도구는 먼저 승인을 요청합니다. Linux에서는 승인 대화 상자가 `zenity`를 사용하며, zenity가 없으면 민감한 도구는 조용히 진행하지 않고 명시적으로 오류를 반환합니다.

**Poki Agent.** 독립 터미널 에이전트도 Linux에서 동일하게 동작합니다(`npm ci --prefix agent` 후 `poki login`). [Poki Agent 실행](agent/README.md)을 참조하세요. 유일한 예외는 iMessage 채널인데, BlueBubbles 서버가 필요하고 그 서버 자체에 Mac이 필요합니다. Telegram 채널은 문제없습니다.

**무선.** Tailscale을 통한 원격 사용(RemotePairing)은 브리지에 내장되어 있으며 플랫폼에 의존하지 않습니다. [remote-iphone.md](docs/remote-iphone.md)의 절차가 이 저장소의 상태 디렉터리에도 그대로 적용됩니다. Tailscale 경유 경로 자체는 Linux 호스트에서 아직 검증되지 않았지만, 동일한 RemotePairing 메커니즘은 로컬 네트워크를 통해 검증되었습니다(Tailscale 주소 대신 iPhone의 LAN 주소 사용). 이후로는 집에서 케이블이 필요 없습니다.

## 실물 기기에서 검증됨

호스트: Arch Linux(x86_64), usbmuxd 1.1.1, Node.js 24.22, pymobiledevice3 11.50. 기기: iPhone18,4, iOS 27.0.1, USB 연결.

| 검사 항목 | 결과 |
| --- | --- |
| usbmuxd를 통한 페어링 및 lockdown 통신 | 동작 |
| pymobiledevice3를 통한 DeveloperDiskImage | 설치됨. 제거 후 `bin/prepare-device.mjs`를 통한 완전한 재설치 포함 |
| `list_elements`(접근성 트리) | 동작 |
| `take_screenshot`(무손실 기기 캡처) | 동작 |
| `tap`, `swipe`, `press_key` | 동작 |
| `type_text`(검증되는 텍스트 입력, 완료 후 이전 클립보드 복원) | 동작 |
| `zenity`를 통한 승인 대화 상자 | 동작 |
| LAN을 통한 무선 조종(LAN IP로의 RemotePairing) | 동작 |
| `npm test` | 모든 테스트 파일이 단독으로는 통과합니다. 비고는 [LINUX.md](LINUX.md) 참조 |

## 알려진 제한 사항

- OCR 헬퍼는 macOS Vision 바이너리이므로, Linux에서는 스크린샷 프롬프트 검사가 `unavailable`을 반환하고 관찰은 접근성 트리에 의존합니다(업스트림에서 헬퍼가 없을 때의 폴백과 동일합니다).
- `get_clipboard`는 Linux에서 클립보드 이미지의 종류를 분류할 수 없습니다.
- 몇몇 실물 기기 도구는 업스트림에서 `xcrun devicectl`을 호출합니다(`open_url`, `terminate_app`, 크래시 로그 접근, 앱 컨테이너 내보내기). idevice-tools와 동등한 구현으로 포팅되기 전까지는 Linux에서 실패합니다.
- 업스트림과 마찬가지로 실물 기기는 세로 방향만 지원합니다.

## 문서

| 주제 | 읽을 문서 |
| --- | --- |
| Linux 포팅 노트(범위, 업스트림과의 차이) | [LINUX.md](LINUX.md) |
| 설치 및 문제 해결 | [설치 가이드](docs/install.md) |
| Poki Agent와 채널 | [Poki Agent 실행](agent/README.md) |
| MCP 도구 참조 | [도구 동작](docs/tools.md) |
| 환경 변수와 세션 | [구성](docs/configuration.md) |
| USB 페어링, 개발자 모드, Wi-Fi | [실물 iPhone 설정](docs/physical-iphone.md) |
| Tailscale을 통한 iPhone | [원격 iPhone](docs/remote-iphone.md) |
| 테스트 결과와 알려진 제한 | [검증](docs/verification.md) |
| 브리지 아키텍처 | [네이티브 브리지](native/NATIVE.md) |

`docs/` 아래 문서 중 일부는 macOS 호스트를 대상으로 작성되었습니다. 호스트 설정에 관해 이 문서나 LINUX.md와 충돌하는 경우, Linux 문서가 우선합니다.

## 면책 조항

Poki는 독립적인 오픈 소스 프로젝트로, Apple, OpenAI, 그리고 그것이 조작하는 앱이나 웹사이트의 제작사와 제휴 관계가 없으며 그들의 보증을 받지도 않습니다. iPhone은 Apple Inc.의 상표입니다. 이 포팅은 비공식 포크이며 업스트림 제작자와 제휴 관계가 없습니다.

타사 앱과 웹사이트를 자동화하면 해당 서비스 약관을 위반할 수 있습니다. 특정 서비스에서 Poki를 사용할지는 스스로 판단하세요. 전송되는 메시지, 이루어지는 변경, 그로 인해 발생하는 계정 제한을 포함하여, Poki가 당신의 기기와 계정에서 하는 일에 대한 책임은 당신에게 있습니다.

Poki는 AGPL-3.0 라이선스에 명시된 대로 "있는 그대로" 보증 없이 제공됩니다.

## 라이선스

Poki는 [AGPL-3.0-only 라이선스](LICENSE)로 오픈 소스입니다.

Copyright (c) 2026 Joseph Zhou and contributors. 오픈 소스 의존성은 각자의 라이선스를 유지합니다. [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)를 참조하세요. 기여 방법은 [CONTRIBUTING.md](CONTRIBUTING.md)를 참조하세요.

## 감사의 글

<p align="center">
  <img src="docs/images/acknowledgments.png" width="860" alt="오픈 소스 위에 구축됨: pi, idevice, idb, BlueBubbles, Hermes Agent, MCP SDK, sharp, FFmpeg"><br>
  <sub>오픈 소스 감사: <a href="https://github.com/earendil-works/pi">pi</a> | <a href="https://github.com/jkcoxson/idevice">idevice</a> | <a href="https://github.com/facebook/idb">idb</a> | <a href="https://github.com/BlueBubblesApp/bluebubbles-server">BlueBubbles</a> | <a href="https://github.com/NousResearch/hermes-agent">Hermes Agent</a> | <a href="https://github.com/modelcontextprotocol/typescript-sdk">MCP SDK</a> | <a href="https://github.com/lovell/sharp">sharp</a> | <a href="https://github.com/FFmpeg/FFmpeg">FFmpeg</a></sub>
</p>
