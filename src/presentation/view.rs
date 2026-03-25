//! iced ベース View 定義

use std::path::Path;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    button, column, container, mouse_area, opaque, progress_bar, row, stack, text, text_input,
    tooltip,
};
use iced::{Border, Color, Element, Fill, Length, Theme};

use crate::presentation::dto::DialogState;
use crate::presentation::message::Message;
use crate::presentation::state::{AppModel, AppStatus};

/// メインビュー構築処理
///
/// @param model 画面状態 Model
/// @return メイン画面 Element
pub fn view(model: &AppModel) -> Element<'_, Message> {
    let base = container(
        column![
            main_filename(model),
            progress_section(model),
            text("ファイルをドラッグしてください").width(Fill).align_x(Horizontal::Center),
            status_bar(model),
        ]
        .spacing(16)
        .padding([24, 24])
        .max_width(720),
    )
    .width(Fill)
    .height(Fill)
    .center_x(Fill)
    .center_y(Fill);

    let interactive_base = if model.ui.status == AppStatus::Wait && model.ui.dialog.is_none() {
        mouse_area(base).on_right_press(Message::ContextMenuRequested).into()
    } else {
        base.into()
    };

    if let Some(dialog) = &model.ui.dialog {
        stack![interactive_base, opaque(dialog_overlay(dialog))].into()
    } else {
        interactive_base
    }
}

/// メインファイル名表示構築処理
///
/// @param model 画面状態 Model
/// @return ファイル名表示 Element
fn main_filename(model: &AppModel) -> Element<'_, Message> {
    const MAX_VISIBLE_CHARS: usize = 66;
    const FILENAME_AREA_HEIGHT: f32 = 72.0;

    let compact = compact_filename(&model.ui.filename, MAX_VISIBLE_CHARS);

    container(text(compact).width(Fill).wrapping(text::Wrapping::Glyph).align_x(Horizontal::Left))
        .width(Fill)
        .height(Length::Fixed(FILENAME_AREA_HEIGHT))
        .align_left(Fill)
        .center_y(Fill)
        .style(|_theme: &Theme| container::Style {
            border: Border { width: 0.5, color: Color::from_rgb8(70, 70, 70), ..Border::default() },
            ..container::Style::default()
        })
        .into()
}

/// プログレス表示構築処理
///
/// @param model 画面状態 Model
/// @return プログレス表示 Element
fn progress_section(model: &AppModel) -> Element<'_, Message> {
    let bar: Element<'_, Message> = progress_bar(0.0..=100.0, model.ui.progress_percent)
        .length(Fill)
        .girth(Length::Fixed(28.0))
        .into();

    if model.ui.status == AppStatus::Wait {
        bar
    } else {
        stack![
            bar,
            container(
                text(format!("{:.1}%", model.ui.progress_percent)).align_x(Horizontal::Center)
            )
            .width(Fill)
            .height(Length::Fixed(28.0))
            .center_x(Fill)
            .center_y(Fill)
        ]
        .into()
    }
}

/// ダイアログ背景オーバーレイ構築処理
///
/// @param dialog 表示対象ダイアログ
/// @return オーバーレイ Element
fn dialog_overlay(dialog: &DialogState) -> Element<'_, Message> {
    let overlay = container(dialog_view(dialog))
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba8(0, 0, 0, 0.45))),
            ..container::Style::default()
        });

    if matches!(dialog, DialogState::ContextMenu) {
        mouse_area(overlay).on_press(Message::ContextMenuDismissed).into()
    } else {
        overlay.into()
    }
}

/// ステータスバー構築処理
///
/// @param model 画面状態 Model
/// @return ステータスバー Element
fn status_bar(model: &AppModel) -> Element<'_, Message> {
    container(text(format!("状態: {}", model.ui.status.label())))
        .width(Fill)
        .padding([8, 12])
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(32, 32, 32))),
                text_color: Some(palette.background.base.text),
                border: Border {
                    width: 1.0,
                    color: Color::from_rgb8(70, 70, 70),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// ダイアログカード構築処理
///
/// @param dialog 表示対象ダイアログ
/// @return ダイアログ Element
fn dialog_view(dialog: &DialogState) -> Element<'_, Message> {
    let card = match dialog {
        DialogState::Info { title, message, .. } | DialogState::Error { title, message, .. } => {
            column![
                text(title).size(20).align_x(Horizontal::Center),
                text(message).align_x(Horizontal::Center),
                button("OK").on_press(Message::DialogAcknowledged)
            ]
            .spacing(12)
            .align_x(Horizontal::Center)
        }
        #[allow(unused_variables)]
        DialogState::KeyPrompt { path, value } => {
            let ok_button = if value.is_empty() {
                button("OK")
            } else {
                button("OK").on_press(Message::KeyInputSubmitted)
            };
            column![
                text("キー入力").size(20).align_x(Horizontal::Center),
                // dialog_filename(path),
                text("キーを入力してください\n(16進数・32文字）").align_x(Horizontal::Center),
                text_input("decryption_key", value)
                    .on_input(Message::KeyInputChanged)
                    .on_submit_maybe((!value.is_empty()).then_some(Message::KeyInputSubmitted))
                    .width(Length::Fixed(260.0)),
                row![ok_button, button("キャンセル").on_press(Message::KeyInputCancelled)]
                    .spacing(8)
                    .align_y(Vertical::Center),
            ]
            .spacing(12)
            .align_x(Horizontal::Center)
        }
        DialogState::ConfirmSwitch { path } => column![
            text("確認").size(20).align_x(Horizontal::Center),
            dialog_filename(path),
            text("復号化処理を中止しますか？").align_x(Horizontal::Center),
            row![
                button("YES").on_press(Message::DialogConfirmed),
                button("NO").on_press(Message::DialogDismissed)
            ]
            .spacing(8)
            .align_y(Vertical::Center),
        ]
        .spacing(12)
        .align_x(Horizontal::Center),
        DialogState::ContextMenu => {
            column![button("キークリア").on_press(Message::ClearKeyRequested)]
                .spacing(8)
                .align_x(Horizontal::Center)
        }
    };

    container(container(card).width(Fill).center_x(Fill))
        .padding(16)
        .width(Length::Fixed(320.0))
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(45, 45, 45))),
                text_color: Some(palette.background.base.text),
                border: Border {
                    width: 1.0,
                    color: Color::from_rgb8(90, 90, 90),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// ダイアログ内ファイル名表示構築処理
///
/// @param path 対象ファイルパス
/// @return ファイル名表示 Element
fn dialog_filename(path: &Path) -> Element<'_, Message> {
    const MAX_VISIBLE_CHARS: usize = 66;
    const FILENAME_AREA_HEIGHT: f32 = 72.0;

    let filename = path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let compact = compact_filename(&filename, MAX_VISIBLE_CHARS);

    let label = container(
        text(compact).width(Fill).wrapping(text::Wrapping::Glyph).align_x(Horizontal::Left),
    )
    .width(Fill)
    .height(Length::Fixed(FILENAME_AREA_HEIGHT))
    .align_left(Fill)
    .center_y(Fill);

    tooltip(
        label,
        container(
            text(filename).width(Fill).wrapping(text::Wrapping::Glyph).align_x(Horizontal::Left),
        )
        .padding([8, 12])
        .max_width(360.0)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(38, 38, 38))),
                text_color: Some(palette.background.base.text),
                border: Border {
                    width: 1.0,
                    color: Color::from_rgb8(90, 90, 90),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        }),
        tooltip::Position::Bottom,
    )
    .into()
}

fn compact_filename(filename: &str, max_visible_chars: usize) -> String {
    if filename.chars().count() > max_visible_chars {
        filename.chars().take(max_visible_chars - 1).collect::<String>() + "….mp4"
    } else {
        filename.to_string()
    }
}
