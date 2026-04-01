//! iced ベース View 定義

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    button, column, container, mouse_area, opaque, progress_bar, radio, row, stack, svg, text,
    text_input,
};
use iced::{Border, Color, Element, Fill, Length, Theme};

use crate::presentation::dto::{DialogState, KeyInputMode};
use crate::presentation::message::Message;
use crate::presentation::state::{AppModel, AppStatus};

/// メインビュー構築処理
///
/// @param model 画面状態 Model
/// @return メイン画面 Element
pub fn view(model: &AppModel) -> Element<'_, Message> {
    let main_content = container(
        column![main_filename(model), inspection_indicator(model), progress_section(model),]
            .spacing(16)
            .padding([16, 16])
            .max_width(720),
    )
    .width(Fill)
    .height(Fill)
    .center_x(Fill)
    .center_y(Fill);

    let base = column![main_content, status_bar(model),].width(Fill).height(Fill).spacing(0);

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

/// ファイル解析中インジケータ表示構築処理
///
/// @param model 画面状態 Model
/// @return インジケータ Element
fn inspection_indicator(model: &AppModel) -> Element<'_, Message> {
    if model.ui.is_inspecting {
        container(
            row![
                text(inspection_spinner_glyph()).size(18),
                text("ファイル解析中...").size(14).color(Color::from_rgb8(170, 170, 170))
            ]
            .spacing(8)
            .align_y(Vertical::Center),
        )
        .width(Fill)
        .center_x(Fill)
        .into()
    } else {
        container(text("")).width(Fill).height(Length::Fixed(18.0)).into()
    }
}

/// 解析中スピナー文字列取得処理
///
/// @return 表示用スピナー文字
fn inspection_spinner_glyph() -> &'static str {
    const GLYPHS: [&str; 4] = ["◐", "◓", "◑", "◒"];
    let frame = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_millis() / 200)
        .unwrap_or(0)
        % GLYPHS.len() as u128) as usize;
    GLYPHS[frame]
}

/// メインファイル名表示構築処理
///
/// @param model 画面状態 Model
/// @return ファイル名表示 Element
fn main_filename(model: &AppModel) -> Element<'_, Message> {
    const MAX_VISIBLE_CHARS: usize = 100;
    const FILENAME_AREA_HEIGHT: f32 = 72.0;

    let content: Element<'_, Message> = if model.ui.status == AppStatus::Wait {
        text("ファイルをドロップしてください")
            .size(14)
            .color(Color::from_rgb8(90, 90, 90))
            .width(Fill)
            .align_x(Horizontal::Center)
            .into()
    } else {
        let compact = compact_filename(&model.ui.filename, MAX_VISIBLE_CHARS);
        text(compact)
            .size(16)
            .width(Fill)
            .wrapping(text::Wrapping::Glyph)
            .align_x(Horizontal::Left)
            .into()
    };

    container(content)
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
    container(
        row![
            text(format!("[{}]", model.ui.status.label()))
                .size(15)
                .color(Color::from_rgb8(180, 180, 180)),
            container(text("")).width(Length::Fill),
            key_status_icon(model.session.has_key),
        ]
        .width(Fill)
        .align_y(Vertical::Center),
    )
    .width(Fill)
    .padding([8, 12])
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(32, 32, 32))),
            text_color: Some(palette.background.base.text),
            border: Border { width: 1.0, color: Color::from_rgb8(70, 70, 70), ..Border::default() },
            ..container::Style::default()
        }
    })
    .into()
}

/// ステータスバー用のキーアイコン描画処理
///
/// @param has_key キーの保有の有無
/// @return キーアイコン Element
fn key_status_icon(has_key: bool) -> Element<'static, Message> {
    let handle = if has_key {
        svg::Handle::from_memory(include_bytes!("../../assets/unlock.svg").as_slice())
    } else {
        svg::Handle::from_memory(include_bytes!("../../assets/lock.svg").as_slice())
    };
    let color =
        if has_key { Color::from_rgb8(200, 200, 200) } else { Color::from_rgb8(120, 120, 120) };

    svg(handle)
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0))
        .style(move |_theme: &Theme, _status| svg::Style { color: Some(color) })
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
        DialogState::KeyPrompt { path, value, mode } => {
            let ok_button = if value.is_empty() {
                button("OK")
            } else {
                button("OK").on_press(Message::KeyInputSubmitted)
            };
            let placeholder =
                if *mode == KeyInputMode::Passphrase { "passphrase" } else { "decryption_key" };
            column![
                text("キー入力").size(20).align_x(Horizontal::Center),
                row![
                    radio(
                        "暗号化キー",
                        KeyInputMode::EncryptionKey,
                        Some(*mode),
                        Message::KeyInputModeChanged
                    ),
                    radio(
                        "パスフレーズ",
                        KeyInputMode::Passphrase,
                        Some(*mode),
                        Message::KeyInputModeChanged
                    )
                ]
                .spacing(10),
                text_input(placeholder, value)
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
        #[allow(unused_variables)]
        DialogState::ConfirmSwitch { path } => column![
            text("確認").size(20).align_x(Horizontal::Center),
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

fn compact_filename(filename: &str, max_visible_chars: usize) -> String {
    let mut parts = filename.rsplitn(2, '.');

    let ext = parts.next().unwrap_or("");
    let name = parts.next().unwrap_or("");

    // 拡張子が存在しない場合（"."が無い）
    if name.is_empty() {
        if filename.chars().count() > max_visible_chars {
            return filename.chars().take(max_visible_chars).collect::<String>() + "…";
        } else {
            return filename.to_string();
        }
    }

    let name_chars: Vec<char> = name.chars().collect();

    if name_chars.len() > max_visible_chars {
        let truncated: String = name_chars.iter().take(max_visible_chars).collect();
        format!("{}….{}", truncated, ext)
    } else {
        filename.to_string()
    }
}
