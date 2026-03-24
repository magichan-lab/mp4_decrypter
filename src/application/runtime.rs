//! バックグラウンド復号処理管理ランタイム

use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::application::ports::Mp4ProcessingPort;
use crate::application::worker::{spawn_decryption_worker, WorkerControl, WorkerEvent};
use crate::domain::value_objects::DecryptionKey;

/// 復号ワーカーの起動・停止・イベント取得を扱うランタイム
///
/// @property repository 復号ポート実装参照
/// @property tx ワーカーイベント送信用チャネル
/// @property rx ワーカーイベント受信用チャネル
/// @property worker_control 実行中ワーカーの制御ハンドル
pub struct DecryptionRuntime<R>
where
    R: Mp4ProcessingPort,
{
    repository: Arc<R>,
    tx: Sender<WorkerEvent>,
    rx: Receiver<WorkerEvent>,
    worker_control: Option<WorkerControl>,
}

impl<R> DecryptionRuntime<R>
where
    R: Mp4ProcessingPort,
{
    /// ランタイム生成処理
    ///
    /// @param repository MP4 処理ポート実装
    /// @return 初期化済みランタイム
    pub fn new(repository: R) -> Self {
        let (tx, rx) = unbounded();
        Self { repository: Arc::new(repository), tx, rx, worker_control: None }
    }

    /// 共有リポジトリ参照取得処理
    ///
    /// @return ユースケース共有用リポジトリ参照
    pub fn repository(&self) -> Arc<R> {
        Arc::clone(&self.repository)
    }

    /// バックグラウンド復号ジョブ開始処理
    ///
    /// @param job_id ジョブ識別子
    /// @param path 入力ファイルパス
    /// @param key 復号キー
    pub fn start_decryption(&mut self, job_id: u64, path: PathBuf, key: DecryptionKey) {
        let control = WorkerControl::new();
        spawn_decryption_worker(
            Arc::clone(&self.repository),
            self.tx.clone(),
            job_id,
            path,
            key,
            control.clone(),
        );
        self.worker_control = Some(control);
    }

    /// 一時停止要求送信処理
    pub fn pause(&self) {
        if let Some(control) = &self.worker_control {
            control.pause();
        }
    }

    /// 再開要求送信処理
    pub fn resume(&self) {
        if let Some(control) = &self.worker_control {
            control.resume();
        }
    }

    /// キャンセル要求送信処理
    pub fn cancel(&self) {
        if let Some(control) = &self.worker_control {
            control.cancel();
        }
    }

    /// 受信済みイベント一括取得処理
    ///
    /// @return 受信済みワーカーイベント一覧
    pub fn drain_events(&mut self) -> Vec<WorkerEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            if matches!(event, WorkerEvent::Finished { .. }) {
                self.worker_control = None;
            }
            events.push(event);
        }
        events
    }
}
