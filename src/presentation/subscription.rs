//! UI サブスクリプション定義

use iced::event;
use iced::window;
use iced::Subscription;

use crate::presentation::message::Message;
use crate::presentation::state::{AppModel, AppStatus};

/// サブスクリプション生成処理
///
/// @param model 画面状態 Model
/// @return 画面状態対応 Subscription
pub fn subscription(model: &AppModel) -> Subscription<Message> {
    let mut subscriptions = vec![event::listen_with(|event, _status, _window| match event {
        iced::Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
        _ => None,
    })];

    if matches!(model.ui.status, AppStatus::Running | AppStatus::Pause) {
        subscriptions
            .push(iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick));
    }

    Subscription::batch(subscriptions)
}
