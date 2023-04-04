use serenity::{model::prelude::Message, prelude::Context};

use super::drop::WLEMOJI;
use crate::{
    drop::{blank, c, g, uppercase_first},
    utils::handler::{find_cards, ocr_captcha},
    PassData,
};

pub fn filter(message: &Message) -> bool {
    if message.author.id == 853629533855809596
        && (!message.embeds.is_empty())
        && message.embeds[0]
            .title
            .as_ref()
            .filter(|a| a == &&"Captcha Drop".to_string())
            .is_some()
    {
        return true;
    }
    false
}
#[allow(unused_must_use)]
pub async fn run(message: &Message, context: &Context) {
    let pd = context.data.read().await;
    let pass_data = pd.get::<PassData>().unwrap();
    let ocr_output = ocr_captcha(
        &pass_data.utils_sender,
        message.embeds[0].image.as_ref().unwrap().url.clone(),
    )
    .await;
    let wl = c(&find_cards(&pass_data.utils_sender, ocr_output.to_vec()).await[0].wl);
    //UNDONE
    message
        .reply(
            &context.http,
            format!(
                "`1]` • {WLEMOJI} `{}` {}• **{}** • {}",
                blank(wl, 4),
                g(&ocr_output[0].gen),
                uppercase_first(&ocr_output[0].name),
                uppercase_first(&ocr_output[0].series)
            ),
        )
        .await;
}
