//! ドメインサービス定義

use std::path::{Path, PathBuf};

/// 復号出力先命名サービス
pub struct OutputNamingService;

impl OutputNamingService {
    /// 出力ファイルパス生成処理
    ///
    /// @param input 入力ファイルパス
    /// @return `_dec` サフィックス付き出力ファイルパス
    pub fn build_output_path(input: &Path) -> PathBuf {
        let stem = input
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        let ext = input
            .extension()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "mp4".to_string());

        input.with_file_name(format!("{stem}_dec.{ext}"))
    }
}
