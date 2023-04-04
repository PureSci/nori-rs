use serenity::{model::prelude::Message, prelude::Context};

use super::drop::WLEMOJI;
use crate::{
    drop::c,
    utils::{cards_handler::Character, handler::find_cards},
    PassData,
};

pub fn filter(message: &Message) -> bool {
    if message.author.id == 853629533855809596
        && (!message.embeds.is_empty())
        && message.embeds[0]
            .title
            .as_ref()
            .filter(|a| a == &&"MINIGAME".to_string())
            .is_some()
    {
        return true;
    }
    false
}
#[allow(unused_must_use)]
pub async fn run(message: &Message, context: &Context) {
    let series = message.embeds[0]
        .description
        .as_ref()
        .map(|desc| {
            desc.split("*(").collect::<Vec<&str>>()[1]
                .split(")*")
                .collect::<Vec<&str>>()[0]
        })
        .unwrap();
    let name = message.embeds[0]
        .description
        .as_ref()
        .map(|desc| desc.split("**").collect::<Vec<&str>>()[1])
        .map(|desc| desc.split("**").collect::<Vec<&str>>()[0])
        .unwrap();
    let pd = context.data.read().await;
    let pass_data = pd.get::<PassData>().unwrap();
    let wl = c(&find_cards(
        &pass_data.utils_sender,
        vec![Character {
            name: name.to_string(),
            series: series.to_string(),
            gen: None,
            wl: None,
        }],
    )
    .await[0]
        .wl);
    message
        .reply(
            &context.http,
            format!("`1]` • {WLEMOJI} {wl} • **{name}** • {series}"),
        )
        .await;
}
