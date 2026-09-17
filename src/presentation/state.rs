//! 画面状態モデル定義

use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

use crate::domain::value_objects::DecryptionKey;
use crate::presentation::dto::{DialogState, KeyInputMode};

/// アプリ状態表示種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStatus {
    /// 待機中
    Wait,
    /// 実行中
    Running,
    /// 完了
    Finished,
    /// 一時停止中
    Pause,
    /// エラー
    Error,
}

impl AppStatus {
    /// ステータス表示ラベル取得処理
    ///
    /// @return ステータス表示文字列
    pub fn label(self) -> &'static str {
        match self {
            Self::Wait => "待機中",
            Self::Running => "処理中",
            Self::Pause => "中断",
            Self::Finished => "完了",
            Self::Error => "エラー",
        }
    }
}

/// 画面描画用 UI 状態
///
/// @property filename 表示中ファイル名
/// @property progress_percent 表示用進捗率
/// @property status 画面ステータス
/// @property dialog 表示中ダイアログ
#[derive(Debug, Clone)]
pub struct UiState {
    pub filename: String,
    pub progress_percent: f32,
    pub status: AppStatus,
    pub is_inspecting: bool,
    pub task_index: usize,
    pub task_total: usize,
    pub dialog: Option<DialogState>,
}

/// セッション継続用内部状態
///
/// @property has_key キー保持有無
/// @property last_key 最後に成功したキー
/// @property current_job_id 現在ジョブ識別子
/// @property task_queue 復号待ちタスク一覧
/// @property active_job 現在復号中ジョブの有無
/// @property pending_inspections 処理中のファイル検査数
/// @property inspection_ids 処理中ファイル検査の識別子集合
/// @property successful_tasks 正常終了したタスク数
/// @property failed_tasks エラー終了したタスク数
/// @property batch_active 現在のタスク受付単位が有効か
/// @property current_inspection_id 現在の検査要求識別子
#[derive(Debug, Clone)]
pub struct SessionState {
    pub has_key: bool,
    pub last_key: Option<DecryptionKey>,
    pub current_job_id: u64,
    pub task_queue: VecDeque<PathBuf>,
    pub active_job: bool,
    pub pending_inspections: usize,
    pub inspection_ids: HashSet<u64>,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
    pub batch_active: bool,
    pub current_inspection_id: u64,
}

/// MVI Model 全体状態
///
/// @property ui 画面表示状態
/// @property session セッション内部状態
#[derive(Debug, Clone)]
pub struct AppModel {
    pub ui: UiState,
    pub session: SessionState,
}

impl AppModel {
    /// 初期画面状態生成処理
    ///
    /// @return 初期化済み Model
    pub fn new() -> Self {
        let mut model = Self {
            ui: UiState {
                filename: String::new(),
                progress_percent: 0.0,
                status: AppStatus::Wait,
                is_inspecting: false,
                task_index: 0,
                task_total: 0,
                dialog: None,
            },
            session: SessionState {
                has_key: false,
                last_key: None,
                current_job_id: 0,
                task_queue: VecDeque::new(),
                active_job: false,
                pending_inspections: 0,
                inspection_ids: HashSet::new(),
                successful_tasks: 0,
                failed_tasks: 0,
                batch_active: false,
                current_inspection_id: 0,
            },
        };
        model.normalize_wait_display();
        model
    }

    /// 待機表示正規化処理
    pub fn normalize_wait_display(&mut self) {
        if self.ui.status == AppStatus::Wait {
            self.ui.filename.clear();
            self.ui.progress_percent = 0.0;
            self.ui.is_inspecting = false;
        }
    }

    /// 待機状態復帰処理
    ///
    /// @param has_key 復帰後キー保持有無
    pub fn reset_to_wait(&mut self, has_key: bool) {
        self.ui.status = AppStatus::Wait;
        self.session.has_key = has_key;
        if !has_key {
            self.session.last_key = None;
        }
        self.ui.dialog = None;
        self.session.task_queue.clear();
        self.session.active_job = false;
        self.session.pending_inspections = 0;
        self.session.inspection_ids.clear();
        self.session.successful_tasks = 0;
        self.session.failed_tasks = 0;
        self.session.batch_active = false;
        self.ui.task_index = 0;
        self.ui.task_total = 0;
        self.normalize_wait_display();
    }

    /// 情報ダイアログ表示処理
    ///
    /// @param title ダイアログタイトル
    /// @param message ダイアログ本文
    /// @param next_has_key OK 後キー保持有無
    pub fn show_info(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        next_has_key: bool,
    ) {
        self.ui.dialog =
            Some(DialogState::Info { title: title.into(), message: message.into(), next_has_key });
    }

    /// エラーダイアログ表示処理
    ///
    /// @param title ダイアログタイトル
    /// @param message ダイアログ本文
    /// @param next_has_key OK 後キー保持有無
    pub fn show_error(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        next_has_key: bool,
    ) {
        self.ui.status = AppStatus::Error;
        self.ui.is_inspecting = false;
        self.ui.dialog =
            Some(DialogState::Error { title: title.into(), message: message.into(), next_has_key });
    }

    /// キー入力ダイアログ表示処理
    ///
    /// @param path 対象ファイルパス
    pub fn show_key_prompt(&mut self, path: PathBuf) {
        self.ui.status = AppStatus::Wait;
        self.ui.is_inspecting = false;
        self.session.has_key = false;
        self.session.last_key = None;
        self.normalize_wait_display();
        self.ui.dialog = Some(DialogState::KeyPrompt {
            path,
            value: String::new(),
            mode: KeyInputMode::EncryptionKey,
        });
    }

    /// ファイル検査開始前状態更新処理
    ///
    /// @param path 対象ファイルパス
    pub fn prepare_inspection(&mut self, path: &PathBuf) -> u64 {
        self.session.current_inspection_id += 1;
        self.ui.filename = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        self.ui.progress_percent = 0.0;
        self.ui.status = AppStatus::Running;
        self.ui.is_inspecting = true;
        self.ui.dialog = None;
        self.session.batch_active = true;
        self.session.pending_inspections += 1;
        self.session.inspection_ids.insert(self.session.current_inspection_id);
        self.session.current_inspection_id
    }

    /// 追加ファイル検査要求の状態更新処理
    pub fn prepare_additional_inspection(&mut self) -> u64 {
        self.session.current_inspection_id += 1;
        self.session.pending_inspections += 1;
        self.session.inspection_ids.insert(self.session.current_inspection_id);
        self.session.current_inspection_id
    }

    /// 復号開始前状態更新処理
    ///
    /// @param path 対象ファイルパス
    /// @param key 復号キー
    /// @return 発行済みジョブ識別子
    pub fn prepare_decryption(&mut self, path: &PathBuf, key: &DecryptionKey) -> u64 {
        self.session.current_job_id += 1;
        self.ui.filename = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        self.ui.progress_percent = 0.0;
        self.ui.status = AppStatus::Running;
        self.ui.is_inspecting = false;
        self.session.has_key = true;
        self.session.active_job = true;
        self.ui.dialog = None;
        self.session.last_key = Some(key.clone());
        self.ui.task_index = self.session.successful_tasks + self.session.failed_tasks + 1;
        self.ui.task_total = self.ui.task_total.max(self.ui.task_index);
        self.session.current_job_id
    }
}
