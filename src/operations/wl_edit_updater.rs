use serenity::{model::prelude::MessageUpdateEvent, prelude::Context};

/**
 * Add leaderboard wishlist updaters!
 */
use crate::{
    utils::{
        cards_handler::{CardsHandleType, Character},
        handler::UtilType,
        series_handler::{Series, SeriesHandleType},
    },
    PassData,
};

pub fn filter(message: &MessageUpdateEvent) -> bool {
    if message.embeds.is_some()
        && (!message.embeds.as_ref().unwrap().is_empty())
        && (message.embeds.as_ref().unwrap()[0]
            .title
            .as_ref()
            .filter(|f| f.contains("(Sort By: Wishlist)"))
            .is_some()
            || message.embeds.as_ref().unwrap()[0]
                .title
                .as_ref()
                .filter(|f| f == &"WISHLIST LEADERBOARD - CHARACTERS")
                .is_some()
            || message.embeds.as_ref().unwrap()[0]
                .title
                .as_ref()
                .filter(|f| f == &"WISHLIST LEADERBOARD - SERIES")
                .is_some()
            || message.embeds.as_ref().unwrap()[0]
                .description
                .as_ref()
                .filter(|f| f.contains("Cards Collected:"))
                .is_some())
    {
        return true;
    }
    false
}

pub async fn run(message: &MessageUpdateEvent, context: &Context) {
    if message.embeds.as_ref().unwrap()[0]
        .title
        .as_ref()
        .filter(|f| f.contains("(Sort By: Wishlist)"))
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let d = message.embeds.as_ref().unwrap()[0]
            .description
            .as_ref()
            .unwrap()
            .split("\n");
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
                        series: x
                            .split_once("•  *")
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
    } else if message.embeds.as_ref().unwrap()[0]
        .title
        .as_ref()
        .filter(|f| f == &"WISHLIST LEADERBOARD - CHARACTERS")
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let d = message.embeds.as_ref().unwrap()[0]
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
    } else if message.embeds.as_ref().unwrap()[0]
        .title
        .as_ref()
        .filter(|f| f == &"WISHLIST LEADERBOARD - SERIES")
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let d = message.embeds.as_ref().unwrap()[0]
            .description
            .as_ref()
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();
        for x in d {
            pass_data
                .utils_sender
                .send(UtilType::SeriesHandler(SeriesHandleType::UpdateSeries(
                    Series {
                        name: x
                            .split_once("` • **")
                            .unwrap()
                            .1
                            .split_once("**")
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
                    },
                )))
                .await
                .unwrap();
        }
    } else if message.embeds.as_ref().unwrap()[0]
        .description
        .as_ref()
        .filter(|f| f.contains("Cards Collected:"))
        .is_some()
    {
        let pd = context.data.read().await;
        let pass_data = pd.get::<PassData>().unwrap();
        let embed = &message.embeds.as_ref().unwrap()[0];
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
    }
}
