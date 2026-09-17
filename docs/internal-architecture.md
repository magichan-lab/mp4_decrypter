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

## 復号出力ファイルのライフサイクル

- 出力ファイルの最終名はドメインサービス `OutputNamingService` が生成します。
- FFmpeg は最終出力名（`*_dec.mp4`）へ直接書き込まず、`*_dec.mp4.tmp` へ書き込みます。
- FFmpeg のトレーラー書き込みまで正常に完了し、出力コンテキストを解放した後、`fs::rename` で一時ファイルを最終名へ移動します。
- 復号処理が失敗またはキャンセルされた場合は、RAII ガードが不完全な一時ファイルを削除します。

## 複数ファイル復号キュー

- ファイルドロップごとに非同期で暗号化状態を検査し、`Encrypted` のファイルだけを `SessionState.task_queue` に追加します。
- `Plain` および検査失敗などの対象外ファイルはタスクキューへ追加せず、処理中のキューを継続します。
- タスクはキュー先頭から1件ずつ復号し、完了後に次のタスクを開始します。復号中の追加ドロップはキュー末尾へ追加されます。
- `task_total` は対象ファイルが検出されるたびに増加し、`task_index` と合わせて `[処理中] (1/5)` 形式で表示します。
- 成功件数とエラー件数はセッション内で集計し、全検査・全復号タスクが終了した時点でのみ結果ダイアログを表示します。成功件数が1件以上なら完了ダイアログ、成功件数が0件なら `復号できませんでした` のエラーダイアログを表示します。
- 最初の暗号化済みファイル検出時に入力されたキーを、同一タスクキュー内の全ファイルへ使用します。

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

## キー操作

- キーが有効な待機状態で画面を右クリックすると、設定メニューに「キーをコピー」と「キーをクリア」を表示します。
- 「キーをコピー」は `CopyKeyRequested` 意図を経由して `Effect::CopyKey` を生成し、Presentation と iced の接続部でクリップボードへ書き込みます。
- キーが無効な場合は「キーをコピー」を表示せず、コピー操作も実行しません。

## 開発メモ

- `src/main.rs` は Presentation と Application の配線だけを担当します。
- Application は Presentation 非依存で、UseCase と Runtime だけを公開します。
- FFmpeg の具体処理は `infrastructure::ffmpeg::repository::FfmpegMp4ProcessingRepository` へ隔離しています。
- キーの正規化・検証は `domain::value_objects::DecryptionKey` に集約しています。

## ドキュメンテーション方針

各モジュール・構造体・関数・主要な定義へ Rust のドキュメンテーションコメント (`///`, `//!`) を付与し、KDoc 的に責務が読み取れる状態を目指しています。
