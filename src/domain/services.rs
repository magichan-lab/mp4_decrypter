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

    /// 変換中に使用する一時出力ファイルパス生成処理
    ///
    /// @param input 入力ファイルパス
    /// @return 最終出力ファイル名に `.tmp` を付与した一時ファイルパス
    pub fn build_temporary_output_path(input: &Path) -> PathBuf {
        let output = Self::build_output_path(input);
        let file_name = output
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "output_dec.mp4".to_string());

        output.with_file_name(format!("{file_name}.tmp"))
    }
}

#[cfg(test)]
mod tests {
    use super::OutputNamingService;
    use std::path::Path;

    #[test]
    fn builds_temporary_path_after_final_mp4_name() {
        let input = Path::new("/work/input.mp4");

        assert_eq!(
            OutputNamingService::build_temporary_output_path(input),
            Path::new("/work/input_dec.mp4.tmp")
        );
    }

    #[test]
    fn preserves_existing_output_naming_for_non_mp4_extensions() {
        let input = Path::new("/work/input.m4v");

        assert_eq!(
            OutputNamingService::build_temporary_output_path(input),
            Path::new("/work/input_dec.m4v.tmp")
        );
    }
}
