//! UI イベントメッセージ定義

use std::path::PathBuf;

use crate::presentation::dto::KeyInputMode;
use crate::presentation::intent::{InspectContext, InspectionOutcome};

/// UI メッセージ
#[derive(Debug, Clone)]
pub enum Message {
    /// タイマー tick
    Tick,
    /// ファイルドロップ
    FileDropped(PathBuf),
    /// ファイル検査完了
    FileInspected {
        inspect_id: u64,
        path: PathBuf,
        context: InspectContext,
        outcome: InspectionOutcome,
    },
    /// 情報／エラー確認ダイアログ OK
    DialogAcknowledged,
    /// 確認ダイアログ YES
    DialogConfirmed,
    /// 確認ダイアログ NO またはメニュー外クリック
    DialogDismissed,
    /// 右クリックメニュー表示要求
    ContextMenuRequested,
    /// コンテキストメニュー非表示要求
    ContextMenuDismissed,
    /// キークリア要求
    ClearKeyRequested,
    /// キー入力変更
    KeyInputChanged(String),
    /// キー入力確定
    KeyInputSubmitted,
    /// キー入力方式変更
    KeyInputModeChanged(KeyInputMode),
    /// キー入力キャンセル
    KeyInputCancelled,
}
