use std::time::{Duration, Instant};

use crate::utils::cards_handler::Character;
use crate::utils::config_data::{get_config_data, get_value_from_all};
use crate::utils::handler::{find_cards, ocr_drop, set_reminder};
use crate::utils::reminders::ReminderTime;
use crate::PassData;
use serenity::model::prelude::EmbedField;
use serenity::{model::prelude::Message, prelude::Context};

pub fn filter(message: &Message) -> bool {
    if message.author.id == 853629533855809596
        && (message.content.ends_with("is dropping the cards")
            || message.content.contains("Your extra drop is being used."))
    {
        return true;
    }
    false
}
/*

1] • :wishlist: 0    • ɢ1717 • Iwao Uesugi • Kimi Ga Aruji...
2] • :wishlist: 0    • ɢ2000 • Takumi Kijima • Ao-chan Cant...
3] • :wishlist: 53   • ɢ1171 • Crona • Soul Eater
@Pure  Took 1140ms to produce.
 */
pub fn blank(mut data: String, length: usize) -> String {
    while !(data.len() >= length) {
        data.push(' ');
        if data.len() > 10 {
            break;
        }
    }
    data
}
/*for _ in 0..data.len() {
    r_str.push(ch.next().unwrap_or(' '));
}
r_str*/

pub fn uppercase_first(s: &String) -> String {
    let mut o_s = String::new();
    for word in s.split(" ") {
        let mut c = word.chars();
        o_s += &(match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        } + " ");
    }
    o_s.trim().to_string()
}

pub fn c(c: &Option<u32>) -> String {
    match c {
        None => "?".to_string(),
        Some(d) => d.to_string(),
    }
}

pub fn g(g: &Option<u16>) -> String {
    match g {
        None => "".to_string(),
        Some(d) => format!("• `ɢ{}` ", blank(d.to_string(), 4)),
    }
}

pub static WLEMOJI: &str = "<:wishlist:1061698988358779020>";
fn get_message(
    typ: &u8,
    cards: &Vec<Character>,
    ping: &bool,
    show_time: &bool,
    userid: &str,
    time: &u128,
) -> String {
    let mut ping_s: String = String::new();
    if *ping {
        ping_s = format!("<@{}> ", userid);
    }
    let mut time_s: String = String::new();
    if *show_time {
        time_s = format!("Took {}ms to produce.", time);
    }
    match typ {
        _ => format!(
            "`1]` • {WLEMOJI} `{}` {}• **{}** • {}\n`2]` • {WLEMOJI} `{}` {}• **{}** • {}\n`3]` • {WLEMOJI} `{}` {}• **{}** • {}\n{ping_s}{time_s}",
            blank(c(&cards[0].wl), 4),
            g(&cards[0].gen),
            uppercase_first(&cards[0].name),
            uppercase_first(&cards[0].series),
            blank(c(&cards[1].wl), 4),
            g(&cards[1].gen),
            uppercase_first(&cards[1].name),
            uppercase_first(&cards[1].series),
            blank(c(&cards[2].wl), 4),
            g(&cards[2].gen),
            uppercase_first(&cards[2].name),
            uppercase_first(&cards[2].series),
        ),
    }
}
#[allow(unused_must_use)]
pub async fn run(message: &Message, context: &Context, mut link: Option<String>) {
    let start = Instant::now();
    let pd = context.data.read().await;
    let pass_data = pd.get::<PassData>().unwrap();
    let userid = message
        .content
        .split_once("<@")
        .unwrap_or(("353623899184824330>", ""))
        .1
        .split_once(">")
        .unwrap_or(("353623899184824330", ""))
        .0;
    let analysis_config = get_config_data(
        &pass_data.db,
        "analysis",
        Some(userid.to_string()),
        message.guild_id.unwrap().to_string(),
        "_all",
    )
    .await;
    if get_value_from_all(&analysis_config, "enabled", "value") != "true" {
        return;
    }
    let show_gen = get_value_from_all(&analysis_config, "show-gen", "value");
    // If it is an Image drop:
    let found_cards;
    if message.attachments.len() > 0 || link != None {
        link = match link {
            None => Some(message.attachments.first().unwrap().url.clone()),
            Some(d) => Some(d),
        };
        let ocr_output = ocr_drop(
            &pass_data.utils_sender,
            link.unwrap(),
            show_gen.parse::<bool>().unwrap(),
        )
        .await;
        found_cards = find_cards(&pass_data.utils_sender, ocr_output.to_vec()).await;
    } else {
        let fields: &Vec<EmbedField> = message.embeds[0].fields.as_ref();
        let cards = (0..3)
            .map(|i| {
                let field = &fields[i];
                let name_series = field
                    .value
                    .split("\n\n")
                    .map(|x| {
                        let r = x
                            .replace("```", "")
                            .replace("\n", "")
                            .trim()
                            .to_ascii_lowercase();
                        if r.ends_with("-") {
                            let new_str = &r[..r.len() - 1];
                            format!("{}...", new_str)
                        } else {
                            r.to_string()
                        }
                    })
                    .collect::<Vec<String>>();
                Character {
                    name: name_series.get(0).unwrap().to_owned(),
                    series: name_series.get(1).unwrap().to_owned(),
                    gen: Some(
                        field.name.split("Gen ").collect::<Vec<&str>>()[1]
                            .trim()
                            .parse::<u16>()
                            .unwrap_or(0),
                    ),
                    wl: None,
                }
            })
            .collect::<Vec<Character>>();
        found_cards = find_cards(&pass_data.utils_sender, cards).await;
    }
    if !message.content.contains("Your extra drop is being used.") {
        set_reminder(
            &pass_data.utils_sender,
            &pass_data.db,
            &message,
            userid,
            ReminderTime::DROP,
        )
        .await;
    }
    let msg = message
        .reply(
            &context.http,
            get_message(
                &get_value_from_all(&analysis_config, "type", "value")
                    .parse::<u8>()
                    .unwrap(),
                &found_cards,
                &get_value_from_all(&analysis_config, "ping", "value")
                    .parse::<bool>()
                    .unwrap(),
                &get_value_from_all(&analysis_config, "show-time", "value")
                    .parse::<bool>()
                    .unwrap(),
                userid,
                &start.elapsed().as_millis(),
            ),
        )
        .await
        .unwrap();
    if get_value_from_all(&analysis_config, "expire-delete", "value") == "true" {
        tokio::time::sleep(Duration::from_secs(60)).await;
        msg.delete(&context.http);
    }
}
