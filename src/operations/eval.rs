use crate::captcha;
use crate::drop;
use crate::grab_reminder;
use crate::series1;
use crate::series2;
use crate::utils::handler::set_reminder;
use crate::utils::reminders::ReminderTime;
use crate::PassData;
use serenity::{model::prelude::Message, prelude::Context};

pub fn filter(message: &Message) -> bool {
    if message.author.id == 353623899184824330 && message.content.starts_with("ne ") {
        return true;
    }
    false
}

pub async fn run(message: &Message, context: &Context) {
    let content = message.content.clone();
    let c = content.split_once(" ").unwrap().1;
    let splitted = match c.split_once(" ") {
        None => (c, ""),
        Some(d) => d,
    };
    let typ = splitted.0;
    let mut input = splitted.1;
    if typ == "drop" {
        if input.len() == 0 {
            input = "https://cdn.discordapp.com/attachments/1071872668938293340/1087771735341940748/31058602-b753-467c-a2b3-889acfdcb28f.webp";
        }
        drop::run(&message, &context, Some(input.to_string())).await;
    } else if typ == "captcha" {
        if input.len() == 0 {
            input = "https://cdn.discordapp.com/attachments/1060674994859933846/1089522639413981264/card.png";
        }
        let m = message
            .channel_id
            .send_message(&context.http, |m| {
                m.add_embed(|embed| {
                    embed.title("Captcha Drop");
                    embed.image(input);
                    embed
                });
                m
            })
            .await
            .unwrap();
        captcha::run(&m, &context).await;
    } else if typ == "series1" {
        if input.len() == 0 {
            input = "**I will drop cards from the most voted series :smugsofi:
            1] Ring My Bell
            2] The Owl House
            3] Persona 4**";
        }
        let m = message
            .channel_id
            .send_message(&context.http, |m| {
                m.add_embed(|embed| {
                    embed.description(input);
                    embed
                });
                m
            })
            .await
            .unwrap();
        series1::run(&m, &context).await;
    } else if typ == "series2" {
        if input.len() == 0 {
            input = "https://cdn.discordapp.com/attachments/1060674994859933846/1076450380633878528/drop.png";
        }
        let m = message
            .channel_id
            .send_message(&context.http, |m| {
                m.content("**Series drop**");
                m.add_file(input);
                m
            })
            .await
            .unwrap();
        series2::run(&m, &context).await;
    } else if typ == "stop" {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        set_reminder(
            &pass_data.utils_sender,
            &pass_data.db,
            &message,
            "1",
            ReminderTime(0),
        )
        .await;
    } else if typ == "grabrem" {
        input = "<@353623899184824330> took the card Hikari Kohinata | mvgxn7 | :woodw: Wood";
        let mut m = message.clone();
        m.content = input.to_string();
        grab_reminder::run(&m, context).await;
    } else if typ == "say" {
        message.delete(&context.http).await.ok();
        match &message.referenced_message {
            None => {
                message
                    .channel_id
                    .send_message(&context.http, |f| f.content(input))
                    .await
                    .ok();
            }
            Some(d) => {
                d.reply(&context.http, input).await.ok();
            }
        }
    }
}
