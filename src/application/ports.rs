//! アプリケーション層から見た外部ポート定義

use std::path::{Path, PathBuf};

use crate::domain::entities::{DecryptionProgress, FileEncryptionState};
use crate::domain::errors::AppError;
use crate::domain::value_objects::DecryptionKey;

/// MP4 判定・復号処理の外部ポート
pub trait Mp4ProcessingPort: Send + Sync + 'static {
    /// 暗号化状態判定処理
    ///
    /// @param path 判定対象ファイルパス
    /// @return 暗号化状態またはアプリケーションエラー
    fn inspect_encryption(&self, path: &Path) -> Result<FileEncryptionState, AppError>;

    /// 出力ファイルパス算出処理
    ///
    /// @param input 入力ファイルパス
    /// @return 出力ファイルパス
    fn output_path(&self, input: &Path) -> PathBuf;

    /// 復号処理実行
    ///
    /// @param input_path 入力ファイルパス
    /// @param key 復号キー
    /// @param on_progress 進捗通知コールバック
    /// @param is_cancelled キャンセル要求判定コールバック
    /// @param is_paused 一時停止要求判定コールバック
    /// @return 出力ファイルパスまたはアプリケーションエラー
    fn decrypt<F, C, P>(
        &self,
        input_path: &Path,
        key: &DecryptionKey,
        on_progress: F,
        is_cancelled: C,
        is_paused: P,
    ) -> Result<PathBuf, AppError>
    where
        F: FnMut(DecryptionProgress),
        C: Fn() -> bool,
        P: Fn() -> bool;
}
