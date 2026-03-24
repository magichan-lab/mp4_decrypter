//! プレゼンテーション用 DTO 定義

use std::path::PathBuf;

use crate::domain::errors::AppError;
use crate::domain::value_objects::DecryptionKey;

/// ダイアログ表示状態 DTO
///
/// @property title ダイアログ見出し
/// @property message ダイアログ本文
/// @property next_has_key ダイアログ終了後のキー保持状態
/// @property path 対象ファイルパス
/// @property value キー入力欄の現在値
#[derive(Debug, Clone)]
pub enum DialogState {
    /// 情報ダイアログ
    Info { title: String, message: String, next_has_key: bool },
    /// エラーダイアログ
    Error { title: String, message: String, next_has_key: bool },
    /// キー入力ダイアログ
    KeyPrompt { path: PathBuf, value: String },
    /// 実行中ジョブ切り替え確認ダイアログ
    ConfirmSwitch { path: PathBuf },
    /// 右クリックメニュー
    ContextMenu,
}

impl DialogState {
    /// 次キー保持状態取得処理
    ///
    /// @return 次キー保持状態
    pub fn next_has_key(&self) -> Option<bool> {
        match self {
            Self::Info { next_has_key, .. } | Self::Error { next_has_key, .. } => {
                Some(*next_has_key)
            }
            Self::KeyPrompt { .. } | Self::ConfirmSwitch { .. } | Self::ContextMenu => None,
        }
    }

    /// キー入力値更新処理
    ///
    /// @param value 更新後入力値
    pub fn update_key_input(&mut self, value: String) {
        if let Self::KeyPrompt { value: current, .. } = self {
            *current = value;
        }
    }

    /// キー入力内容取得処理
    ///
    /// @return 対象パスと復号キー変換結果
    pub fn key_prompt_submission(&self) -> Option<(PathBuf, Result<DecryptionKey, AppError>)> {
        match self {
            Self::KeyPrompt { path, value } => Some((
                path.clone(),
                DecryptionKey::from_padded_input(value)
                    .map_err(|error| AppError::Validation(error.to_string())),
            )),
            _ => None,
        }
    }
}
