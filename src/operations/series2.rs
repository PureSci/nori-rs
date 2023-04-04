use serenity::{model::prelude::Message, prelude::Context};

use super::drop::WLEMOJI;
use crate::{
    drop::{blank, c, g, uppercase_first},
    utils::handler::{find_cards, ocr_series},
    PassData,
};

pub fn filter(message: &Message) -> bool {
    if message.author.id == 853629533855809596 && message.content == "**Series drop**" {
        return true;
    }
    false
}
#[allow(unused_must_use)]
pub async fn run(message: &Message, context: &Context) {
    let pd = context.data.read().await;
    let pass_data = pd.get::<PassData>().unwrap();
    let ocr_output = ocr_series(
        &pass_data.utils_sender,
        message.attachments.first().unwrap().url.clone(),
    )
    .await;
    let found_cards = find_cards(&pass_data.utils_sender, ocr_output.to_vec()).await;
    //UNDONE
    message
        .reply(
            &context.http,
            format!(
                "`1]` • {WLEMOJI} `{}` {}• **{}** • {}\n`2]` • {WLEMOJI} `{}` {}• **{}** • {}",
                blank(c(&found_cards[0].wl), 4),
                g(&ocr_output[0].gen),
                uppercase_first(&ocr_output[0].name),
                uppercase_first(&ocr_output[0].series),
                blank(c(&found_cards[1].wl), 4),
                g(&ocr_output[1].gen),
                uppercase_first(&ocr_output[1].name),
                uppercase_first(&ocr_output[1].series)
            ),
        )
        .await;
}
