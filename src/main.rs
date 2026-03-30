//! MP4 復号デスクトップアプリ実行エントリ
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{application, window, Subscription, Task, Theme};

use mp4_decrypter::application::runtime::DecryptionRuntime;
use mp4_decrypter::application::use_cases::{InspectFileUseCase, ValidateOutputPathUseCase};
use mp4_decrypter::application::worker::WorkerEvent;
use mp4_decrypter::domain::entities::{DecryptionResult, FileEncryptionState, LaunchRequest};
use mp4_decrypter::domain::errors::AppError;
use mp4_decrypter::infrastructure::cli::CliLaunchArgumentParser;
use mp4_decrypter::infrastructure::ffmpeg::repository::FfmpegMp4ProcessingRepository;
use mp4_decrypter::presentation::intent::{Effect, InspectionOutcome, Intent};
use mp4_decrypter::presentation::message::Message;
use mp4_decrypter::presentation::reducer::reduce;
use mp4_decrypter::presentation::state::AppModel;

/// Presentation と Application を束ねる統合ランタイム
///
/// @property model 画面表示用 Model
/// @property decryption_runtime アプリケーション層ランタイム
struct AppRuntime {
    model: AppModel,
    decryption_runtime: DecryptionRuntime<FfmpegMp4ProcessingRepository>,
}

impl AppRuntime {
    /// 統合ランタイム生成処理
    ///
    /// @return 初期化済み統合ランタイム
    fn new() -> Self {
        Self {
            model: AppModel::new(),
            decryption_runtime: DecryptionRuntime::new(FfmpegMp4ProcessingRepository),
        }
    }

    /// 起動要求反映処理
    ///
    /// @param launch_request 起動要求または起動エラー
    /// @return iced 初期 Task
    fn initialize(&mut self, launch_request: Result<LaunchRequest, AppError>) -> Task<Message> {
        self.dispatch(Intent::LaunchParsed(launch_request))
    }

    /// reducer 経由意図反映処理
    ///
    /// @param intent 反映対象意図
    fn dispatch(&mut self, intent: Intent) -> Task<Message> {
        let effects = reduce(&mut self.model, intent);
        let mut tasks = Vec::new();
        for effect in effects {
            tasks.push(self.run_effect(effect));
        }
        Task::batch(tasks)
    }

    /// 副作用実行処理
    ///
    /// @param effect 実行対象副作用命令
    fn run_effect(&mut self, effect: Effect) -> Task<Message> {
        match effect {
            Effect::InspectFile { path, context } => self.inspect_file_task(path, context),
            Effect::StartDecryption { job_id, path, key } => {
                let validate_use_case =
                    ValidateOutputPathUseCase::new(self.decryption_runtime.repository());
                match validate_use_case.execute(&path) {
                    Ok(_) => self.decryption_runtime.start_decryption(job_id, path, key),
                    Err(error) => {
                        return self.dispatch(Intent::WorkerFinished {
                            job_id,
                            result: DecryptionResult::Failed(error),
                        });
                    }
                }
                Task::none()
            }
            Effect::PauseWorker => {
                self.decryption_runtime.pause();
                Task::none()
            }
            Effect::ResumeWorker => {
                self.decryption_runtime.resume();
                Task::none()
            }
            Effect::CancelWorker => {
                self.decryption_runtime.cancel();
                Task::none()
            }
        }
    }

    /// ファイル検査 Task 生成処理
    ///
    /// @param path 対象ファイルパス
    /// @param context ファイル検査文脈
    /// @return 検査完了メッセージ返却 Task
    fn inspect_file_task(
        &self,
        path: std::path::PathBuf,
        context: mp4_decrypter::presentation::intent::InspectContext,
    ) -> Task<Message> {
        let repository = self.decryption_runtime.repository();
        Task::perform(
            async move {
                let inspect_use_case = InspectFileUseCase::new(repository);
                let outcome = match inspect_use_case.execute(&path) {
                    Ok(FileEncryptionState::Encrypted) => InspectionOutcome::Encrypted,
                    Ok(FileEncryptionState::Plain) => InspectionOutcome::Plain,
                    Err(error) => InspectionOutcome::Failed(error),
                };
                (path, context, outcome)
            },
            |(path, context, outcome)| Message::FileInspected { path, context, outcome },
        )
    }

    /// ワーカーイベント反映処理
    fn drain_worker_events(&mut self) -> Task<Message> {
        let mut tasks = Vec::new();
        for event in self.decryption_runtime.drain_events() {
            match event {
                WorkerEvent::Progress { job_id, progress } => {
                    tasks.push(self.dispatch(Intent::WorkerProgress {
                        job_id,
                        filename: progress.filename,
                        ratio: progress.ratio,
                    }));
                }
                WorkerEvent::Finished { job_id, result } => {
                    tasks.push(self.dispatch(Intent::WorkerFinished { job_id, result }));
                }
            }
        }
        Task::batch(tasks)
    }
}

/// iced 初期化処理
///
/// @return アプリケーションランタイムと初期 Task
fn initialize() -> (AppRuntime, Task<Message>) {
    let mut app = AppRuntime::new();
    let launch_request = CliLaunchArgumentParser::parse_env();
    let task = app.initialize(launch_request);
    (app, task)
}

/// UI 更新処理
///
/// @param app アプリケーションランタイム
/// @param message 受信 UI メッセージ
/// @return 次の iced Task
fn update(app: &mut AppRuntime, message: Message) -> Task<Message> {
    match message {
        Message::Tick => Task::batch(vec![app.drain_worker_events(), app.dispatch(Intent::Tick)]),
        Message::FileDropped(path) => app.dispatch(Intent::FileDropped(path)),
        Message::FileInspected { path, context, outcome } => {
            app.dispatch(Intent::FileInspected { path, context, outcome })
        }
        Message::DialogAcknowledged => app.dispatch(Intent::DialogAcknowledged),
        Message::DialogConfirmed => app.dispatch(Intent::DialogConfirmed),
        Message::DialogDismissed => app.dispatch(Intent::DialogDismissed),
        Message::ContextMenuRequested => app.dispatch(Intent::ContextMenuRequested),
        Message::ContextMenuDismissed => app.dispatch(Intent::ContextMenuDismissed),
        Message::ClearKeyRequested => app.dispatch(Intent::ClearKeyRequested),
        Message::KeyInputChanged(value) => app.dispatch(Intent::KeyInputChanged(value)),
        Message::KeyInputSubmitted => app.dispatch(Intent::KeyInputSubmitted),
        Message::KeyInputCancelled => app.dispatch(Intent::KeyInputCancelled),
    }
}

/// サブスクリプション生成処理
///
/// @param app アプリケーションランタイム
/// @return 画面状態対応 Subscription
fn subscription(app: &AppRuntime) -> Subscription<Message> {
    mp4_decrypter::presentation::subscription::subscription(&app.model)
}

/// 画面描画処理
///
/// @param app アプリケーションランタイム
/// @return 描画 Element
fn view(app: &AppRuntime) -> iced::Element<'_, Message> {
    mp4_decrypter::presentation::view::view(&app.model)
}

/// Windows用のアイコンの読み込み処理
///
/// @return Windowsアイコン
fn load_window_icon() -> window::Icon {
    window::icon::from_file_data(include_bytes!("../assets/app-icon.png"), None)
        .expect("failed to load window icon")
}

/// アプリケーション起動処理
///
/// @return iced 実行結果
fn main() -> iced::Result {
    let icon = load_window_icon();
    application(initialize, update, view)
        .subscription(subscription)
        .theme(Theme::Dark)
        .title("MP4 Decrypter")
        .window(window::Settings { icon: Some(icon), ..window::Settings::default() })
        .window_size((300.0, 280.0))
        .resizable(false)
        .run()
}
