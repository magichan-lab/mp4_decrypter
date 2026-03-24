//! ドメイン中心エンティティ定義

use std::path::PathBuf;

use crate::domain::errors::AppError;
use crate::domain::value_objects::DecryptionKey;

/// 起動要求
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchRequest {
    /// 復号キー付きファイル起動要求
    KeyAndFile { key: DecryptionKey, path: PathBuf },
    /// ファイルのみ起動要求
    FileOnly(PathBuf),
    /// ファイル指定なし起動要求
    NoFile,
}

/// ファイル暗号化状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEncryptionState {
    /// 暗号化済み状態
    Encrypted,
    /// 平文状態
    Plain,
}

/// 復号進捗
///
/// @property filename UI 表示用ファイル名
/// @property ratio 0.0..=1.0 の進捗率
#[derive(Debug, Clone, PartialEq)]
pub struct DecryptionProgress {
    pub filename: String,
    pub ratio: f32,
}

/// 復号完了結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecryptionResult {
    /// 正常完了
    Completed,
    /// ユーザーキャンセル
    Cancelled,
    /// 型付きエラー失敗
    Failed(AppError),
}
