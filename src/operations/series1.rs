use serenity::{model::prelude::Message, prelude::Context};

use super::drop::WLEMOJI;
use crate::{
    drop::blank,
    utils::{handler::find_series, series_handler::Series},
    PassData,
};

pub fn filter(message: &Message) -> bool {
    if message.author.id == 853629533855809596
        && (!message.embeds.is_empty())
        && message.embeds[0]
            .description
            .as_ref()
            .filter(|a| a.starts_with("**I will drop cards from the most voted series"))
            .is_some()
    {
        return true;
    }
    false
}

fn c(c: &Option<u32>) -> String {
    match c {
        None => "0".to_string(),
        Some(d) => d.to_string(),
    }
}

#[allow(unused_must_use)]
pub async fn run(message: &Message, context: &Context) {
    let pd = context.data.read().await;
    let pass_data = pd.get::<PassData>().unwrap();
    let mut series = message.embeds[0]
        .description
        .as_ref()
        .unwrap()
        .split("\n")
        .collect::<Vec<&str>>();
    //series.pop();
    series.remove(0);
    series.pop();
    let v_series = series
        .iter()
        .map(|x| Series {
            name: x.split("]").collect::<Vec<&str>>()[1]
                .split("**")
                .collect::<Vec<&str>>()[0]
                .trim()
                .to_string(),
            wl: None,
        })
        .collect::<Vec<Series>>();
    let s_series: [Series; 3] = std::array::from_fn(|i| v_series[i].clone());
    let found_series = find_series(&pass_data.utils_sender, s_series).await;
    message
        .reply(
            &context.http,
            format!(
                "`1]` • {WLEMOJI} `{}` • {}
`2]` • {WLEMOJI} `{}` • {}
`3]` • {WLEMOJI} `{}` • {}",
                blank(c(&found_series[0].wl), 4),
                found_series[0].name,
                blank(c(&found_series[1].wl), 4),
                found_series[1].name,
                blank(c(&found_series[2].wl), 4),
                found_series[2].name,
            ),
        )
        .await;
}
