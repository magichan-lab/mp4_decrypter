//! CLI 起動引数パーサー実装

use std::path::PathBuf;

use crate::domain::entities::LaunchRequest;
use crate::domain::errors::AppError;
use crate::domain::value_objects::DecryptionKey;

/// 実プロセス引数読み取りパーサー
pub struct CliLaunchArgumentParser;

impl CliLaunchArgumentParser {
    /// 実行環境引数解析処理
    ///
    /// @return 起動要求または起動引数エラー
    pub fn parse_env() -> Result<LaunchRequest, AppError> {
        Self::parse_from(std::env::args())
    }

    /// 任意引数列解析処理
    ///
    /// @param args 引数列
    /// @return 起動要求または起動引数エラー
    pub fn parse_from<I>(args: I) -> Result<LaunchRequest, AppError>
    where
        I: IntoIterator<Item = String>,
    {
        let args = args.into_iter().skip(1).collect::<Vec<_>>();
        let key = args.iter().find_map(|arg| Self::extract_key(arg));
        let file_args = args
            .iter()
            .filter(|arg| Self::extract_key(arg).is_none())
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        match (key, file_args.as_slice()) {
            (Some(key), [path, ..]) => Ok(LaunchRequest::KeyAndFile {
                key: DecryptionKey::parse(key)
                    .map_err(|error| AppError::InvalidLaunchArgs(error.to_string()))?,
                path: path.clone(),
            }),
            (None, [path]) => Ok(LaunchRequest::FileOnly(path.clone())),
            (None, []) => Ok(LaunchRequest::NoFile),
            (Some(_), []) => Err(AppError::InvalidLaunchArgs(
                "復号キーが指定されていますが、対象ファイルがありません".to_string(),
            )),
            _ => Err(AppError::InvalidLaunchArgs("起動引数の組み合わせが不正です".to_string())),
        }
    }

    /// 復号キー引数抽出処理
    ///
    /// @param arg 解析対象引数
    /// @return 抽出済みキー文字列
    pub fn extract_key(arg: &str) -> Option<String> {
        let marker = "decryption_key=";
        let position = arg.find(marker)?;
        let key = arg[(position + marker.len())..].trim();
        if key.is_empty() {
            None
        } else {
            Some(key.to_string())
        }
    }
}
