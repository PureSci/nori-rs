use crate::utils::cards_handler::{CardsHandleType, Character};
use crate::utils::handler::UtilType;
use crate::utils::series_handler::{Series, SeriesHandleType};
use crate::PassData;
use serenity::{model::prelude::Message, prelude::Context};

pub fn filter(message: &Message) -> bool {
    if message.author.id == 853629533855809596 {
        return true;
    }
    false
}

pub async fn run(message: &Message, context: &Context) {
    if message.embeds.is_empty() {
        return;
    }
    if message.embeds[0]
        .title
        .as_ref()
        .filter(|f| f == &&"LOOKUP")
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let filtered = message.embeds[0]
            .description
            .as_ref()
            .unwrap()
            .split("\n")
            .filter(|x| {
                ["**`Wishlisted", "**Series", "**Name"]
                    .iter()
                    .any(|y| x.starts_with(y))
            })
            .collect::<Vec<&str>>();
        pass_data
            .utils_sender
            .send(UtilType::CardsHandler(CardsHandleType::UpdateCard(
                Character {
                    name: filtered[1]
                        .split_once(": ** ")
                        .unwrap()
                        .1
                        .trim()
                        .to_string(),
                    series: filtered[0]
                        .split_once(": ** ")
                        .unwrap()
                        .1
                        .trim()
                        .to_string(),
                    wl: Some(
                        filtered[2]
                            .split_once("➜** `")
                            .unwrap()
                            .1
                            .split_once("`")
                            .unwrap()
                            .0
                            .trim()
                            .parse::<u32>()
                            .unwrap(),
                    ),
                    gen: None,
                },
            )))
            .await
            .unwrap();
    } else if message.embeds[0]
        .description
        .as_ref()
        .filter(|f| f.contains("Cards Collected:"))
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let embed = &message.embeds[0];
        let series = embed
            .description
            .as_ref()
            .unwrap()
            .split_once("__**")
            .unwrap()
            .1
            .split_once("**__")
            .unwrap()
            .0
            .trim()
            .to_string();
        let d = embed.fields[0].value.split("\n").collect::<Vec<&str>>();
        for x in d {
            pass_data
                .utils_sender
                .send(UtilType::CardsHandler(CardsHandleType::UpdateCard(
                    Character {
                        name: x
                            .split_once("**")
                            .unwrap()
                            .1
                            .split_once("**")
                            .unwrap()
                            .0
                            .trim()
                            .to_string(),
                        series: series.clone(),
                        wl: Some(
                            x.split_once("❤️ `")
                                .unwrap()
                                .1
                                .split_once("`")
                                .unwrap()
                                .0
                                .trim()
                                .parse::<u32>()
                                .unwrap(),
                        ),
                        gen: None,
                    },
                )))
                .await
                .unwrap();
        }
        pass_data
            .utils_sender
            .send(UtilType::SeriesHandler(SeriesHandleType::UpdateSeries(
                Series {
                    name: series,
                    wl: Some(
                        embed
                            .description
                            .as_ref()
                            .unwrap()
                            .split_once("*Total Wishlist:* **")
                            .unwrap()
                            .1
                            .split_once("**")
                            .unwrap()
                            .0
                            .trim()
                            .parse::<u32>()
                            .unwrap(),
                    ),
                },
            )))
            .await
            .unwrap();
    } else if message.embeds[0]
        .title
        .as_ref()
        .filter(|f| f.contains("__Characters Lookup__"))
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let d = message.embeds[0].description.as_ref().unwrap().split("\n");
        for x in d {
            pass_data
                .utils_sender
                .send(UtilType::CardsHandler(CardsHandleType::UpdateCard(
                    Character {
                        name: x
                            .split_once("` •  **")
                            .unwrap()
                            .1
                            .split_once("**")
                            .unwrap()
                            .0
                            .trim()
                            .to_string(),
                        series: x
                            .split_once("• *")
                            .unwrap()
                            .1
                            .split_once("*")
                            .unwrap()
                            .0
                            .trim()
                            .to_string(),
                        wl: Some(
                            x.split_once("❤️ ")
                                .unwrap()
                                .1
                                .split_once("`")
                                .unwrap()
                                .0
                                .trim()
                                .parse::<u32>()
                                .unwrap(),
                        ),
                        gen: None,
                    },
                )))
                .await
                .unwrap();
        }
    } else if message.embeds[0]
        .title
        .as_ref()
        .filter(|f| f.contains("SERIES"))
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let d = message.embeds[0].description.as_ref().unwrap().split("\n");
        for x in d {
            if x.find("❤️ `").is_none() {
                return;
            }
            pass_data
                .utils_sender
                .send(UtilType::SeriesHandler(SeriesHandleType::UpdateSeries(
                    Series {
                        name: x
                            .split_once("**")
                            .unwrap()
                            .1
                            .split_once("**")
                            .unwrap()
                            .0
                            .trim()
                            .to_string(),
                        wl: Some(
                            x.split_once("❤️ `")
                                .unwrap()
                                .1
                                .split_once("`")
                                .unwrap()
                                .0
                                .trim()
                                .parse::<u32>()
                                .unwrap(),
                        ),
                    },
                )))
                .await
                .unwrap();
        }
    } else if message.embeds[0]
        .title
        .as_ref()
        .filter(|f| f == &"WISHLIST LEADERBOARD - CHARACTERS")
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let d = message.embeds[0]
            .description
            .as_ref()
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();
        for x in d {
            pass_data
                .utils_sender
                .send(UtilType::CardsHandler(CardsHandleType::UpdateCard(
                    Character {
                        name: x
                            .split_once("` • **")
                            .unwrap()
                            .1
                            .split_once("** • *")
                            .unwrap()
                            .0
                            .trim()
                            .to_string(),
                        series: x
                            .split_once("** • *")
                            .unwrap()
                            .1
                            .split_once("*")
                            .unwrap()
                            .0
                            .trim()
                            .to_string(),
                        wl: Some(
                            x.split_once("> `")
                                .unwrap()
                                .1
                                .split_once("`")
                                .unwrap()
                                .0
                                .trim()
                                .parse::<u32>()
                                .unwrap(),
                        ),
                        gen: None,
                    },
                )))
                .await
                .unwrap();
        }
    }
}
