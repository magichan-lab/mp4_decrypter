//! OS 標準の機密情報ストア利用実装

use keyring::Entry;

use crate::domain::value_objects::DecryptionKey;

const SERVICE_NAME: &str = "mp4_decrypter";
const USER_NAME: &str = "decryption_key";

/// 復号キー永続化サービス
///
/// ストアが利用できない場合も、呼び出し元がセッション内のメモリ保持へ
/// フォールバックできるよう、ストア操作のエラーを外部へ返さない。
#[derive(Debug, Clone, Copy, Default)]
pub struct SecretStore;

impl SecretStore {
    /// 保存済み復号キーの取得処理
    pub fn load_key(&self) -> Option<DecryptionKey> {
        let entry = Entry::new(SERVICE_NAME, USER_NAME).ok()?;
        let value = entry.get_password().ok()?;
        DecryptionKey::parse(value).ok()
    }

    /// 復号キーの保存処理
    pub fn save_key(&self, key: &DecryptionKey) {
        if let Ok(entry) = Entry::new(SERVICE_NAME, USER_NAME) {
            let _ = entry.set_password(key.as_str());
        }
    }

    /// 保存済み復号キーの削除処理
    pub fn delete_key(&self) {
        if let Ok(entry) = Entry::new(SERVICE_NAME, USER_NAME) {
            let _ = entry.delete_credential();
        }
    }
}
