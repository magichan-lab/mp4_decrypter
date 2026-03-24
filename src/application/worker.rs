//! バックグラウンド復号ワーカー

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam_channel::Sender;

use crate::application::ports::Mp4ProcessingPort;
use crate::domain::entities::{DecryptionProgress, DecryptionResult};
use crate::domain::errors::AppError;
use crate::domain::value_objects::DecryptionKey;

/// ワーカー通知イベント
pub enum WorkerEvent {
    /// 進捗更新イベント
    Progress {
        /// 対象ジョブ ID
        job_id: u64,
        /// 進捗詳細
        progress: DecryptionProgress,
    },
    /// 完了イベント
    Finished {
        /// 対象ジョブ ID
        job_id: u64,
        /// 完了結果
        result: DecryptionResult,
    },
}

/// ワーカー制御フラグ
///
/// @property cancel キャンセル要求フラグ
/// @property pause 一時停止要求フラグ
#[derive(Debug, Clone)]
pub struct WorkerControl {
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
}

impl WorkerControl {
    /// 制御ハンドル生成処理
    ///
    /// @return 制御ハンドル
    pub fn new() -> Self {
        Self { cancel: Arc::new(AtomicBool::new(false)), pause: Arc::new(AtomicBool::new(false)) }
    }

    /// キャンセル要求設定処理
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
        self.pause.store(false, Ordering::SeqCst);
    }

    /// 一時停止要求設定処理
    pub fn pause(&self) {
        self.pause.store(true, Ordering::SeqCst);
    }

    /// 再開要求設定処理
    pub fn resume(&self) {
        self.pause.store(false, Ordering::SeqCst);
    }
}

/// 復号ワーカースレッド起動処理
///
/// @param repository MP4 処理ポート実装参照
/// @param tx イベント送信用チャネル
/// @param job_id ジョブ識別子
/// @param path 入力ファイルパス
/// @param key 復号キー
/// @param control ワーカー制御ハンドル
pub fn spawn_decryption_worker<R>(
    repository: Arc<R>,
    tx: Sender<WorkerEvent>,
    job_id: u64,
    path: PathBuf,
    key: DecryptionKey,
    control: WorkerControl,
) where
    R: Mp4ProcessingPort,
{
    std::thread::spawn(move || {
        let result = repository.decrypt(
            &path,
            &key,
            |progress| {
                let _ = tx.send(WorkerEvent::Progress { job_id, progress });
            },
            || control.cancel.load(Ordering::SeqCst),
            || control.pause.load(Ordering::SeqCst),
        );

        let result = match result {
            Ok(_) => DecryptionResult::Completed,
            Err(AppError::Cancelled) => DecryptionResult::Cancelled,
            Err(error) => DecryptionResult::Failed(error),
        };

        let _ = tx.send(WorkerEvent::Finished { job_id, result });
    });
}
