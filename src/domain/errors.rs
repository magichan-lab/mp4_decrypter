//! ドメインおよびアプリケーション層共有エラー定義

use std::fmt;

/// アプリケーション全体で扱う型付きエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    /// 起動引数不正
    InvalidLaunchArgs(String),
    /// 入力値または事前条件違反
    Validation(String),
    /// ファイルシステム関連エラー
    FileSystem(String),
    /// 外部実装依存エラー
    Infrastructure(String),
    /// ユーザーキャンセル
    Cancelled,
}

impl AppError {
    /// UI 表示用メッセージ取得処理
    ///
    /// @return 表示用メッセージ文字列
    pub fn user_message(&self) -> String {
        match self {
            Self::InvalidLaunchArgs(message)
            | Self::Validation(message)
            | Self::FileSystem(message)
            | Self::Infrastructure(message) => message.clone(),
            Self::Cancelled => "処理がキャンセルされました".to_string(),
        }
    }
}

impl fmt::Display for AppError {
    /// 文字列表現生成処理
    ///
    /// @param f フォーマッタ
    /// @return フォーマット結果
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for AppError {}
