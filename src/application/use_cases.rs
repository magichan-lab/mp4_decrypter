//! アプリケーションユースケース群

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::application::ports::Mp4ProcessingPort;
use crate::domain::entities::FileEncryptionState;
use crate::domain::errors::AppError;

/// ファイル暗号化状態判定ユースケース
///
/// @property repository MP4 処理ポート参照
pub struct InspectFileUseCase<R>
where
    R: Mp4ProcessingPort,
{
    repository: Arc<R>,
}

impl<R> InspectFileUseCase<R>
where
    R: Mp4ProcessingPort,
{
    /// ユースケース生成処理
    ///
    /// @param repository MP4 処理ポート参照
    /// @return ユースケースインスタンス
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// 暗号化状態判定実行処理
    ///
    /// @param path 判定対象ファイルパス
    /// @return 暗号化状態またはアプリケーションエラー
    pub fn execute(&self, path: &Path) -> Result<FileEncryptionState, AppError> {
        self.repository.inspect_encryption(path)
    }
}

/// 出力パス競合検証ユースケース
///
/// @property repository MP4 処理ポート参照
pub struct ValidateOutputPathUseCase<R>
where
    R: Mp4ProcessingPort,
{
    repository: Arc<R>,
}

impl<R> ValidateOutputPathUseCase<R>
where
    R: Mp4ProcessingPort,
{
    /// ユースケース生成処理
    ///
    /// @param repository MP4 処理ポート参照
    /// @return ユースケースインスタンス
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// 出力パス検証処理
    ///
    /// @param input 入力ファイルパス
    /// @return 利用可能な出力ファイルパスまたはアプリケーションエラー
    pub fn execute(&self, input: &Path) -> Result<PathBuf, AppError> {
        let output = self.repository.output_path(input);
        if output.exists() {
            return Err(AppError::Validation("出力ファイルが既に存在しています".to_string()));
        }
        Ok(output)
    }
}
