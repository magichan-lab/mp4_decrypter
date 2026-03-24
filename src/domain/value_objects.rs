//! ドメイン値オブジェクト群

use anyhow::{bail, Result};

/// 復号キー値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecryptionKey(String);

impl DecryptionKey {
    /// 復号キー最大文字数
    pub const MAX_LEN: usize = 32;

    /// 生文字列検証処理
    ///
    /// @param value 入力キー文字列
    /// @return 検証済み復号キーまたは検証エラー
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if Self::is_valid_hex(&value) {
            Ok(Self(value))
        } else {
            bail!("decryption_key は偶数桁の16進文字列である必要があります")
        }
    }

    /// UI 入力正規化処理
    ///
    /// @param value 入力文字列
    /// @return 16 進数のみへ正規化した文字列
    pub fn sanitize_input(value: &str) -> String {
        value.chars().filter(|c| c.is_ascii_hexdigit()).take(Self::MAX_LEN).collect()
    }

    /// 0 埋め入力値変換処理
    ///
    /// @param value 入力文字列
    /// @return 32 文字へ正規化済み復号キーまたは検証エラー
    pub fn from_padded_input(value: &str) -> Result<Self> {
        let sanitized = Self::sanitize_input(value);
        if sanitized.is_empty() {
            bail!("復号できません")
        }

        let padded =
            if sanitized.len() >= Self::MAX_LEN { sanitized } else { format!("{sanitized:0<32}") };

        Self::parse(padded)
    }

    /// 16 進文字列妥当性判定処理
    ///
    /// @param value 判定対象文字列
    /// @return 妥当性判定結果
    pub fn is_valid_hex(value: &str) -> bool {
        !value.is_empty() && value.len() % 2 == 0 && value.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// FFI 向け文字列参照取得処理
    ///
    /// @return 16 進文字列参照
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
