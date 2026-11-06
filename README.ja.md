<p align="center">
  <img src="docs/images/poki-icon.png" width="72" height="72" alt="Poki アイコン">
</p>

<h1 align="center">Poki for Linux</h1>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a> | <a href="README.zh-TW.md">繁體中文</a> | 日本語 | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0--only-blue" alt="ライセンス: AGPL-3.0-only"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20iOS-lightgrey" alt="プラットフォーム: Linux と iOS">
</p>

**Linux ホストから MCP エージェントで実物の iPhone を操作できます。Mac も、ジェイルブレイクも、追加のハードウェアも不要です。** Poki はローカルサーバーを通じて iPhone のアクセシビリティツリーと HID 入力（タップ、スワイプ、文字入力、キー入力）を任意の MCP クライアントに公開し、エージェントがあなたと同じように電話を操作できるようにします。

> このリポジトリは [JosssphZhou/poki](https://github.com/JosssphZhou/poki) の**非公式 Linux 移植**です。上流が土台とする macOS ホストサポート（Xcode、Vision OCR ヘルパー、シミュレータ）はここでは置き換えられるか省略されています。デバイス側の機能はすべて上流の設計そのもので、駆動するツールチェーンが Linux 用になっただけです。移植の詳細は [LINUX.md](LINUX.md) を参照してください。上流の公式サイト: https://poki.bot

## 仕組み

Rust で書かれ [idevice](https://github.com/jkcoxson/idevice) を基盤とするブリッジが、**usbmuxd** 経由で USB 接続した iPhone と通信し、CoreDevice トンネルを開いて RemoteXPC で開発者サービスと対話します。これは Xcode が使うのと同じ経路ですが、Xcode は要りません。DeveloperDiskImage は **pymobiledevice3** がインストールします（イメージのダウンロードとデバイス向けのパーソナライズを行います）。MCP サーバー自体は純粋な Node.js プログラムです。

## 要件

- usbmuxd が動作している Linux ホスト（Arch Linux x86_64 で検証済み）
- 開発者モードと UI オートメーションを有効にした iPhone（[実機 iPhone の設定](docs/physical-iphone.md) を参照。デバイス側の手順は上流と同じです）
- Node.js 24+、Rust（rustup 経由）、Python 3、ffmpeg、git
- pymobiledevice3（`pip install pymobiledevice3`、または仮想環境へインストール。`PATH` にない場合は `POKI_PYMOBILEDEVICE3` で場所を指定できます）

`./bin/setup` は不足している前提条件をチェックし、それぞれのインストール方法を教えてくれます。

## クイックスタート

```sh
git clone https://github.com/carrotciallo/poki-for-linux.git
cd poki-for-linux
./bin/setup
```

`./bin/setup` はネイティブブリッジと idevice ツールをビルドし、npm 依存関係をインストールし、sharp のプリビルドバイナリが実際にあることを検証します（一部の Linux の npm 設定ではそれらが黙って省略されますが、setup が修復します）。

USB でロック解除済みの iPhone を接続し、一度だけデバイスの準備を行います。

```sh
node bin/prepare-device.mjs
```

pymobiledevice3 経由で DeveloperDiskImage をインストールします。すでにインストール済みの場合は、その旨を報告するだけで何もしません。

サーバーを起動し、動かし続けます。

```sh
npm start
```

同じディレクトリから、MCP サーバーをクライアントに登録します。

```sh
claude mcp add --scope user poki-mcp -- node "$PWD/src/stdio.mjs"   # Claude Code
codex mcp add poki-mcp -- node "$PWD/src/stdio.mjs"                 # Codex
```

状態データは `POKI_MCP_STATE_DIR` が設定されていない限り `$XDG_STATE_HOME/PokiMCP`（デフォルトは `~/.local/state/PokiMCP`）に保存されます。

## タスクを試す

```sh
node bin/call.mjs list_devices   # デバイスの UUID を確認
node bin/call.mjs list_elements '{"device":"<device UUID>"}'
```

> **注意:** Poki はあなたの実際の iPhone を操作します。メッセージの送信や支払いを頼めば、本当に実行します。画面の内容は、あなたが設定したモデルサービスに送信されます。機微なツールは先に承認を求めます。Linux では承認ダイアログに `zenity` を使い、zenity がなければ機微なツールは黙って続行せず、明示的にエラーを返します。

**Poki Agent。** スタンドアロンのターミナルエージェントも Linux でそのまま動作します（`npm ci --prefix agent` の後 `poki login`）。[Poki Agent の実行](agent/README.md) を参照してください。唯一の例外は iMessage チャネルで、BlueBubbles サーバーが必要ですが、それ自体に Mac が必要です。Telegram チャネルは問題ありません。

**ワイヤレス。** Tailscale 経由のリモート利用（RemotePairing）はブリッジに組み込まれておりプラットフォームに依存しません。[remote-iphone.md](docs/remote-iphone.md) の手順が、本リポジトリの状態ディレクトリでもそのまま使えます。Tailscale 経由の経路自体は Linux ホストでは未検証ですが、同じ RemotePairing の仕組みはローカルネットワーク経由で検証済みです（Tailscale のアドレスの代わりに iPhone の LAN アドレスを使用）。以後、自宅ではケーブルが不要になります。

## 実機での検証

ホスト: Arch Linux（x86_64）、usbmuxd 1.1.1、Node.js 24.22、pymobiledevice3 11.50。デバイス: iPhone18,4、iOS 27.0.1、USB 接続。

| チェック項目 | 結果 |
| --- | --- |
| usbmuxd 経由のペアリングと lockdown 通信 | 動作 |
| pymobiledevice3 による DeveloperDiskImage | インストール済み。アンインストール後に `bin/prepare-device.mjs` で完全な再インストールも実施 |
| `list_elements`（アクセシビリティツリー） | 動作 |
| `take_screenshot`（ロスレスなデバイスキャプチャ） | 動作 |
| `tap`、`swipe`、`press_key` | 動作 |
| `type_text`（検証付きテキスト入力、完了後に元のクリップボードを復元） | 動作 |
| `zenity` による承認ダイアログ | 動作 |
| LAN 経由のワイヤレス操作（LAN IP への RemotePairing） | 動作 |
| `npm test` | テストファイルは個別にはすべて通過。注記は [LINUX.md](LINUX.md) を参照 |

## 既知の制限

- OCR は bridge 内の小さな Rust 製認識器が tesseract を使って担い（`build/ocr` としてインストール）、macOS Vision ヘルパーと同じ JSON 形式を出力します。スクリーンショットのプロンプトチェックと、アクセシビリティで読めなかった場合のフォールバックが動作します。認識品質は tesseract 次第で、明瞭な UI テキストはよく読めますが、小さな文字や装飾文字は漏れることがあります。デフォルトの言語は簡体字中国語・英語・日本語で、`POKI_OCR_LANGS` で変更できます。
- `get_clipboard` は Linux ではクリップボード画像の種別を判別できません。
- 一部の実機ツールは上流で `xcrun devicectl` を呼び出しており（`open_url`、`terminate_app`、クラッシュログへのアクセス、アプリコンテナのエクスポート）、idevice-tools の等価実装へ移植されるまでは Linux で失敗します。
- 上流と同様、実機は縦向きのみ対応です。

## ドキュメント

| トピック | 読むもの |
| --- | --- |
| Linux 移植ノート（範囲、上流との差分） | [LINUX.md](LINUX.md) |
| インストールとトラブルシューティング | [インストールガイド](docs/install.md) |
| Poki Agent とチャネル | [Poki Agent の実行](agent/README.md) |
| MCP ツールリファレンス | [ツールの動作](docs/tools.md) |
| 環境変数とセッション | [設定](docs/configuration.md) |
| USB ペアリング、開発者モード、Wi-Fi | [実機 iPhone の設定](docs/physical-iphone.md) |
| Tailscale 経由の iPhone | [リモート iPhone](docs/remote-iphone.md) |
| テスト結果と既知の制限 | [検証](docs/verification.md) |
| ブリッジアーキテクチャ | [ネイティブブリッジ](native/NATIVE.md) |

`docs/` 以下の一部の文書は macOS ホスト向けに書かれています。ホストのセットアップについて本文書や LINUX.md と食い違う場合は、Linux 向け文書が優先します。

## 免責事項

Poki は独立したオープンソースプロジェクトであり、Apple、OpenAI、およびそれが操作するアプリやウェブサイトのメーカーとは提携しておらず、それらの推奨も受けていません。iPhone は Apple Inc. の商標です。本移植は非公式フォークであり、上流の作者とは提携していません。

サードパーティのアプリやウェブサイトの自動化は、それらの利用規約に違反する可能性があります。あるサービスで Poki を使うかどうかはご自身で判断してください。送信されるメッセージ、行われる変更、それによって生じるアカウント制限を含め、Poki があなたのデバイスとアカウントに対して行うことの責任はあなたにあります。

Poki は AGPL-3.0 ライセンスに定めるとおり「現状のまま」無保証で提供されます。

## ライセンス

Poki は [AGPL-3.0-only ライセンス](LICENSE) のもとでオープンソースです。

Copyright (c) 2026 Joseph Zhou and contributors. オープンソースの依存関係はそれぞれのライセンスを保持します。[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) を参照してください。コントリビューションについては [CONTRIBUTING.md](CONTRIBUTING.md) を参照してください。

## 謝辞

<p align="center">
  <img src="docs/images/acknowledgments.png" width="860" alt="オープンソースの上に構築: pi、idevice、idb、BlueBubbles、Hermes Agent、MCP SDK、sharp、FFmpeg"><br>
  <sub>オープンソース謝辞: <a href="https://github.com/earendil-works/pi">pi</a> | <a href="https://github.com/jkcoxson/idevice">idevice</a> | <a href="https://github.com/facebook/idb">idb</a> | <a href="https://github.com/BlueBubblesApp/bluebubbles-server">BlueBubbles</a> | <a href="https://github.com/NousResearch/hermes-agent">Hermes Agent</a> | <a href="https://github.com/modelcontextprotocol/typescript-sdk">MCP SDK</a> | <a href="https://github.com/lovell/sharp">sharp</a> | <a href="https://github.com/FFmpeg/FFmpeg">FFmpeg</a></sub>
</p>
