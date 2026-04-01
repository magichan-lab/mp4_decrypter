//! MVI reducer 実装

use crate::domain::entities::{DecryptionResult, LaunchRequest};
use crate::domain::errors::AppError;
use crate::domain::value_objects::DecryptionKey;
use crate::presentation::dto::DialogState;
use crate::presentation::intent::{Effect, InspectContext, InspectionOutcome, Intent};
use crate::presentation::state::{AppModel, AppStatus};

/// 状態遷移導出処理
///
/// @param model 更新対象 Model
/// @param intent 解釈対象意図
/// @return 副作用命令一覧
pub fn reduce(model: &mut AppModel, intent: Intent) -> Vec<Effect> {
    match intent {
        Intent::LaunchParsed(result) => match result {
            Ok(LaunchRequest::KeyAndFile { key, path }) => {
                let job_id = model.prepare_decryption(&path, &key);
                vec![Effect::StartDecryption { job_id, path, key }]
            }
            Ok(LaunchRequest::FileOnly(path)) => {
                model.reset_to_wait(false);
                let inspect_id = model.prepare_inspection(&path);
                vec![Effect::InspectFile { inspect_id, path, context: InspectContext::WithoutKey }]
            }
            Ok(LaunchRequest::NoFile) => {
                model.reset_to_wait(false);
                vec![]
            }
            Err(error) => {
                model.show_error("エラー", error.user_message(), false);
                vec![]
            }
        },
        Intent::Tick => vec![],
        Intent::FileDropped(path) => match model.ui.status {
            AppStatus::Wait => {
                let context = if model.session.has_key {
                    InspectContext::WithKey
                } else {
                    InspectContext::WithoutKey
                };
                let inspect_id = model.prepare_inspection(&path);
                vec![Effect::InspectFile { inspect_id, path, context }]
            }
            AppStatus::Running => {
                model.ui.status = AppStatus::Pause;
                model.session.pending_drop = Some(path.clone());
                model.ui.dialog = Some(DialogState::ConfirmSwitch { path });
                vec![Effect::PauseWorker]
            }
            AppStatus::Finished => {
                if matches!(model.ui.dialog, Some(DialogState::Info { .. })) {
                    model.reset_to_wait(model.session.has_key);
                    let context = if model.session.has_key {
                        InspectContext::WithKey
                    } else {
                        InspectContext::WithoutKey
                    };
                    let inspect_id = model.prepare_inspection(&path);
                    vec![Effect::InspectFile { inspect_id, path, context }]
                } else {
                    vec![]
                }
            }
            AppStatus::Error | AppStatus::Pause => vec![],
        },
        Intent::FileInspected { inspect_id, path, context, outcome } => {
            if inspect_id != model.session.current_inspection_id {
                return vec![];
            }

            match (context, outcome) {
                (_, InspectionOutcome::Failed(error)) => {
                    model.ui.is_inspecting = false;
                    model.show_error("エラー", error.user_message(), model.session.has_key);
                    vec![]
                }
                (InspectContext::WithoutKey, InspectionOutcome::Plain) => {
                    model.ui.is_inspecting = false;
                    model.reset_to_wait(false);
                    model.show_info("確認", "このファイルは暗号化されていません", false);
                    vec![]
                }
                (InspectContext::WithKey, InspectionOutcome::Plain) => {
                    model.ui.is_inspecting = false;
                    model.reset_to_wait(true);
                    model.show_info("確認", "このファイルは暗号化されていません", true);
                    vec![]
                }
                (InspectContext::WithoutKey, InspectionOutcome::Encrypted) => {
                    model.ui.is_inspecting = false;
                    model.show_key_prompt(path);
                    vec![]
                }
                (InspectContext::WithKey, InspectionOutcome::Encrypted) => {
                    model.ui.is_inspecting = false;
                    if let Some(key) = model.session.last_key.clone() {
                        let job_id = model.prepare_decryption(&path, &key);
                        vec![Effect::StartDecryption { job_id, path, key }]
                    } else {
                        model.show_error("エラー", "復号できません", false);
                        vec![]
                    }
                }
            }
        }
        Intent::WorkerProgress { job_id, filename, ratio } => {
            if job_id == model.session.current_job_id {
                model.ui.filename = filename;
                model.ui.progress_percent = (ratio * 100.0).clamp(0.0, 100.0);
                model.ui.is_inspecting = false;
                if model.ui.status != AppStatus::Pause {
                    model.ui.status = AppStatus::Running;
                }
            }
            vec![]
        }
        Intent::WorkerFinished { job_id, result } => {
            if job_id != model.session.current_job_id {
                return vec![];
            }

            match result {
                DecryptionResult::Completed => {
                    model.ui.progress_percent = 100.0;
                    model.ui.status = AppStatus::Finished;
                    model.show_info("完了", "終了しました", true);
                    vec![]
                }
                DecryptionResult::Failed(error) => {
                    model.show_error("エラー", error.user_message(), model.session.has_key);
                    vec![]
                }
                DecryptionResult::Cancelled => {
                    if let Some(path) = model.session.pending_drop.take() {
                        if let Some(key) = model.session.last_key.clone() {
                            let job_id = model.prepare_decryption(&path, &key);
                            vec![Effect::StartDecryption { job_id, path, key }]
                        } else {
                            model.reset_to_wait(false);
                            vec![]
                        }
                    } else {
                        model.reset_to_wait(model.session.has_key);
                        vec![]
                    }
                }
            }
        }
        Intent::DialogAcknowledged => {
            if let Some(dialog) = model.ui.dialog.take() {
                if let Some(next_has_key) = dialog.next_has_key() {
                    model.reset_to_wait(next_has_key);
                }
            }
            vec![]
        }
        Intent::DialogConfirmed => {
            if matches!(model.ui.dialog, Some(DialogState::ConfirmSwitch { .. })) {
                model.ui.dialog = None;
                vec![Effect::CancelWorker]
            } else {
                vec![]
            }
        }
        Intent::DialogDismissed => {
            if matches!(model.ui.dialog, Some(DialogState::ConfirmSwitch { .. })) {
                model.ui.dialog = None;
                model.session.pending_drop = None;
                model.ui.status = AppStatus::Running;
                vec![Effect::ResumeWorker]
            } else {
                vec![]
            }
        }
        Intent::ContextMenuRequested => {
            if model.ui.status == AppStatus::Wait && model.ui.dialog.is_none() {
                model.ui.dialog = Some(DialogState::ContextMenu);
            }
            vec![]
        }
        Intent::ContextMenuDismissed => {
            if matches!(model.ui.dialog, Some(DialogState::ContextMenu)) {
                model.ui.dialog = None;
            }
            vec![]
        }
        Intent::ClearKeyRequested => {
            model.session.has_key = false;
            model.session.last_key = None;
            model.ui.dialog = None;
            model.ui.status = AppStatus::Wait;
            model.ui.is_inspecting = false;
            model.normalize_wait_display();
            vec![]
        }
        Intent::KeyInputChanged(value) => {
            if let Some(dialog) = model.ui.dialog.as_mut() {
                dialog.update_key_input(DecryptionKey::sanitize_input(&value));
            }
            vec![]
        }
        Intent::KeyInputSubmitted => {
            if let Some((path, submission)) =
                model.ui.dialog.as_ref().and_then(DialogState::key_prompt_submission)
            {
                match submission {
                    Ok(key) => {
                        let job_id = model.prepare_decryption(&path, &key);
                        vec![Effect::StartDecryption { job_id, path, key }]
                    }
                    Err(error) => {
                        model.show_error("エラー", error.user_message(), false);
                        vec![]
                    }
                }
            } else {
                vec![]
            }
        }
        Intent::KeyInputCancelled => {
            model.show_error(
                "エラー",
                AppError::Validation("復号できません".to_string()).user_message(),
                false,
            );
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    //! reducer ユニットテスト

    use std::path::PathBuf;

    use super::reduce;
    use crate::presentation::dto::DialogState;
    use crate::presentation::intent::{Effect, InspectContext, InspectionOutcome, Intent};
    use crate::presentation::state::{AppModel, AppStatus};

    /// 暗号化ファイル検知時のキー入力ダイアログ遷移確認
    #[test]
    fn prompt_key_when_encrypted_without_key() {
        let mut model = AppModel::new();
        let effects = reduce(
            &mut model,
            Intent::FileInspected {
                inspect_id: 0,
                path: PathBuf::from("movie.mp4"),
                context: InspectContext::WithoutKey,
                outcome: InspectionOutcome::Encrypted,
            },
        );

        assert!(effects.is_empty());
        assert!(matches!(
            model.ui.dialog,
            Some(crate::presentation::dto::DialogState::KeyPrompt { .. })
        ));
    }

    /// 起動引数のキー付きファイル指定時の開始命令返却確認
    #[test]
    fn start_decryption_on_launch_with_key() {
        let mut model = AppModel::new();
        let key = crate::domain::value_objects::DecryptionKey::parse("0011aa22").unwrap();
        let effects = reduce(
            &mut model,
            Intent::LaunchParsed(Ok(crate::domain::entities::LaunchRequest::KeyAndFile {
                key: key.clone(),
                path: PathBuf::from("movie.mp4"),
            })),
        );

        assert!(
            matches!(effects.as_slice(), [Effect::StartDecryption { key: effect_key, .. }] if effect_key == &key)
        );
    }

    /// 待機中ドロップ時に検査中表示へ遷移すること
    #[test]
    fn move_to_inspecting_on_drop_while_waiting() {
        let mut model = AppModel::new();
        let path = PathBuf::from("movie.mp4");

        let effects = reduce(&mut model, Intent::FileDropped(path.clone()));

        assert!(matches!(
            effects.as_slice(),
            [Effect::InspectFile { path: effect_path, .. }] if effect_path == &path
        ));
        assert_eq!(model.ui.status, AppStatus::Running);
        assert!(model.ui.is_inspecting);
        assert_eq!(model.ui.filename, "movie.mp4");
    }

    /// 完了ダイアログ表示中のドロップ時に再検査を開始すること
    #[test]
    fn restart_inspection_when_drop_while_finished_dialog() {
        let mut model = AppModel::new();
        model.session.has_key = true;
        model.ui.status = AppStatus::Finished;
        model.ui.dialog = Some(DialogState::Info {
            title: "完了".to_string(),
            message: "終了しました".to_string(),
            next_has_key: true,
        });
        let path = PathBuf::from("next.mp4");

        let effects = reduce(&mut model, Intent::FileDropped(path.clone()));

        assert!(matches!(
            effects.as_slice(),
            [Effect::InspectFile { path: effect_path, context: InspectContext::WithKey }]
            if effect_path == &path
        ));
        assert_eq!(model.ui.status, AppStatus::Running);
        assert!(model.ui.is_inspecting);
        assert!(model.ui.dialog.is_none());
        assert_eq!(model.ui.filename, "next.mp4");
    }

    /// 古い検査結果は反映しないこと
    #[test]
    fn ignore_stale_inspection_result() {
        let mut model = AppModel::new();
        let path = PathBuf::from("movie.mp4");

        let effects = reduce(&mut model, Intent::FileDropped(path.clone()));
        let inspect_id = match effects.first() {
            Some(Effect::InspectFile { inspect_id, .. }) => *inspect_id,
            _ => panic!("inspect effect is expected"),
        };

        let stale_effects = reduce(
            &mut model,
            Intent::FileInspected {
                inspect_id: inspect_id.saturating_sub(1),
                path: path.clone(),
                context: InspectContext::WithoutKey,
                outcome: InspectionOutcome::Encrypted,
            },
        );
        assert!(stale_effects.is_empty());
        assert!(model.ui.is_inspecting);

        let current_effects = reduce(
            &mut model,
            Intent::FileInspected {
                inspect_id,
                path,
                context: InspectContext::WithoutKey,
                outcome: InspectionOutcome::Plain,
            },
        );
        assert!(current_effects.is_empty());
        assert!(matches!(
            model.ui.dialog,
            Some(crate::presentation::dto::DialogState::Info { .. })
        ));
    }
}
