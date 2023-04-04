use serenity::{model::prelude::Message, prelude::Context};

use crate::{
    utils::{handler::set_reminder, reminders::ReminderTime},
    PassData,
};

pub fn filter(message: &Message) -> bool {
    if message.author.id == 853629533855809596
        && (message.content.contains("battled") || message.content.contains("took the card"))
    {
        return true;
    }
    false
}

pub async fn run(message: &Message, context: &Context) {
    let user_id;
    if message.content.contains("battled") {
        user_id = message
            .content
            .split("battled")
            .next()
            .and_then(|s| s.split("<@").nth(1))
            .and_then(|s| s.split(">").next());
    } else {
        user_id = message
            .content
            .split("<@")
            .nth(1)
            .and_then(|s| s.split(">").next());
    }
    if user_id.is_some() {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        set_reminder(
            &pass_data.utils_sender,
            &pass_data.db,
            message,
            user_id.unwrap(),
            ReminderTime::GRAB,
        )
        .await;
    }
}
