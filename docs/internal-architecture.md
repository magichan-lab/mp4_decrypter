# Internal Architecture

このドキュメントは、MP4 Decrypter の内部仕様と実装構成を開発者向けにまとめたものです。

## レイヤー構成

コードベースは次の 4 層で構成されています。

- `presentation`
  - iced UI の MVI 構成です。
  - `message` / `intent` / `state` / `reducer` / `view` / `subscription` / `dto` に分割しています。
  - DTO 変換、画面状態、イベント入力、表示組み立てを担当します。
- `application`
  - Presentation から独立した `use_cases` / `runtime` / `ports` で構成しています。
  - `InspectFileUseCase`、`ValidateOutputPathUseCase`、`DecryptionRuntime` がユースケース境界と実行制御を担当します。
  - エラーは `AppError` として型付きで伝播し、キャンセルも文字列比較ではなく `Cancelled` として表現します。
- `domain`
  - `LaunchRequest`、`DecryptionKey`、`DecryptionResult`、`AppError` などアプリ中核の概念を保持します。
  - UI や FFmpeg など実装都合のポートは持たず、純粋な概念とサービスに限定しています。
  - 出力ファイル命名規則はドメインサービスへ分離しています。
- `infrastructure`
  - CLI 引数解析と FFmpeg / ファイル IO 実装を担当します。
  - Application 層のポートを実装する FFmpeg アダプタと、起動引数パーサーを格納しています。

## ディレクトリ構成

```text
src/
├── application/
│   ├── mod.rs
│   ├── ports.rs
│   ├── runtime.rs
│   ├── use_cases.rs
│   └── worker.rs
├── domain/
│   ├── entities.rs
│   ├── errors.rs
│   ├── mod.rs
│   ├── services.rs
│   └── value_objects.rs
├── infrastructure/
│   ├── cli.rs
│   ├── ffmpeg/
│   │   ├── ffi.rs
│   │   ├── mod.rs
│   │   └── repository.rs
│   └── mod.rs
├── presentation/
│   ├── dto.rs
│   ├── intent.rs
│   ├── message.rs
│   ├── mod.rs
│   ├── reducer.rs
│   ├── state.rs
│   ├── subscription.rs
│   └── view.rs
├── lib.rs
├── main.rs
└── ffmpeg_shim.c
```

## Presentation / MVI 構成

presentation 層では、UI の責務を次のように分離しています。

- `message.rs`: iced のイベント入力を表現します。
- `intent.rs`: reducer が解釈する UI 意図と Application 呼び出し命令を表現します。
- `state.rs`: `UiState` と `SessionState` を分けて、表示状態と内部セッション状態を保持します。
- `dto.rs`: ダイアログ表示の DTO を保持します。
- `reducer.rs`: 純粋関数として状態遷移を記述します。
- `view.rs`: Model から View を組み立てます。
- `subscription.rs`: Tick とファイルドロップ監視を管理します。

## 開発メモ

- `src/main.rs` は Presentation と Application の配線だけを担当します。
- Application は Presentation 非依存で、UseCase と Runtime だけを公開します。
- FFmpeg の具体処理は `infrastructure::ffmpeg::repository::FfmpegMp4ProcessingRepository` へ隔離しています。
- キーの正規化・検証は `domain::value_objects::DecryptionKey` に集約しています。

## ドキュメンテーション方針

各モジュール・構造体・関数・主要な定義へ Rust のドキュメンテーションコメント (`///`, `//!`) を付与し、KDoc 的に責務が読み取れる状態を目指しています。
