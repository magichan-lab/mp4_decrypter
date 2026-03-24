# MP4 Decrypter

MP4 Decrypter は、起動引数またはドラッグ&ドロップで指定した暗号化 MP4 ファイルを復号し、復号済みファイルを生成するデスクトップアプリです。

## 外部仕様

### 基本動作

- 起動引数に `decryption_key=...` と対象 MP4 ファイルパスの両方が渡された場合、復号処理を開始します。
- ファイルパスのみが渡された場合、対象ファイルが暗号化されているかを判定します。
  - 暗号化されていない場合は「このファイルは暗号化されていません」と表示します。
  - 暗号化されている場合はキー入力ダイアログを表示し、正しいキーが入力された場合のみ復号を開始します。
- ファイルパスが渡されない場合は待機状態で起動します。
- ウィンドウへファイルをドラッグ&ドロップできます。
- 復号中はファイル名、プログレスバー、進捗率を表示します。
- 復号開始時に出力ファイル (`*_dec.mp4`) が既に存在する場合はエラーを表示して停止します。

### キー入力仕様

- 入力可能文字は 16 進数 (`0-9`, `a-f`, `A-F`) のみです。
- 最大 32 文字まで入力できます。
- 空文字では OK ボタンは無効です。
- 32 文字未満の場合は 0 埋めして復号に使用します。

### 出力ファイル

- 出力ファイル名は入力ファイル名に `_dec` を付与した `*_dec.mp4` 形式です。

## 初期セットアップ

### 前提

- Rust / Cargo
- FFmpeg 開発用ファイル
  - `include`
  - `lib`
- Windows 向けビルドでは、FFmpeg の共有ライブラリと依存 DLL が実行環境から参照できること

### FFmpeg の配置方法

ビルド時の FFmpeg 探索順は次の通りです。

1. 環境変数 `FFMPEG_DIR` が定義されている場合
   - `FFMPEG_DIR/include`
   - `FFMPEG_DIR/lib`
2. `FFMPEG_DIR` が未定義の場合
   - `third_party/ffmpeg/include`
   - `third_party/ffmpeg/lib`

### セットアップ例

```bash
export FFMPEG_DIR=/opt/ffmpeg
cargo build
```

`FFMPEG_DIR` を利用しない場合は、リポジトリ配下に次の構成で FFmpeg を配置してください。

```text
third_party/
└── ffmpeg/
    ├── include/
    └── lib/
```

### 開発時の確認コマンド

```bash
cargo fmt --check
cargo test
cargo build
```

> `build.rs` は `FFMPEG_DIR` または `third_party/ffmpeg` に有効な FFmpeg の `include` / `lib` が無い場合は失敗します。

## ライセンス

本リポジトリのソースコードは **MIT License** です。詳細は `LICENSE` を参照してください。

## サードパーティライブラリ

このソフトウェアはFFmpegライブラリ（LGPL v2.1以降）を使用しています：
- libavformat
- libavcodec
- libavutil
- libswresample
- libswscale

FFmpegソースコード：
https://ffmpeg.org/

Note：
- 本アプリケーションは外部ライブラリとして **FFmpeg** を **動的リンク** または **静的リンク** で利用します。
FFmpeg 自体は本リポジトリの MIT ライセンスには含まれず、**LGPL-2.1-or-later** の条件に従います。
配布時のクレジットと注意事項は `THIRD_PARTY_NOTICES.md` を参照してください。
- ユーザーはFFmpegを修正版に置き換えてアプリケーションを再リンクすることができます。
ビルド手順はこのリポジトリに記載されています。

## 開発者向け内部資料

内部仕様・アーキテクチャ・モジュール構成は `docs/internal-architecture.md` を参照してください。
