use std::sync::Arc;

use async_channel::{Receiver, Sender};
use mongodb::{bson::doc, Database};
use rayon::prelude::*;

use serde::Deserialize;
use serenity::http::Http;

use super::database_handler::DatabaseHandleType;
use super::handler::UtilType;
#[derive(Debug, Deserialize, Clone)]
pub struct Character {
    pub name: String,
    pub series: String,
    pub gen: Option<u16>,
    pub wl: Option<u32>,
}

pub enum CardsHandleType {
    FindCard(Vec<Character>, Sender<Vec<Character>>),
    UpdateCard(Character),
}

pub async fn cards_handler_loop(
    receiver: Receiver<CardsHandleType>,
    db: Database,
    utils_sender: Sender<UtilType>,
    _http: Arc<Http>,
) {
    let collection = db.collection::<Character>("analysis_characters");
    let mut cursor = collection.find(None, None).await.unwrap();
    let mut characters = vec![];
    while cursor.advance().await.unwrap() {
        let mut c = cursor.deserialize_current().unwrap();
        c.series.retain(|f| f != '!');
        c.series = c.series.replace(":", "");
        characters.push(c);
    }
    loop {
        match receiver.recv().await.unwrap() {
            CardsHandleType::FindCard(cards, return_sender) => {
                let found = (0..cards.len())
                    .into_par_iter()
                    .map(|i| {
                        let card = &cards[i];
                        let mut card_name = card.name.clone();
                        if card_name.ends_with("....") {
                            card_name = card_name.replace("....", "...");
                        }
                        let mut card_series = card.series.clone();
                        if card_series.ends_with("....") {
                            card_series = card_series.replace("....", "...");
                        }
                        if card_name.ends_with("..")
                            && card_name.chars().nth(card_name.len() - 3).unwrap() != '.'
                        {
                            card_name += ".";
                        }
                        if card_series.ends_with("..")
                            && card_series.chars().nth(card_series.len() - 3).unwrap() != '.'
                        {
                            card_series += ".";
                        }
                        let is_dot_name = card_name.ends_with("...");
                        let is_dot_series = card_series.ends_with("...");
                        card_name = process_string(card_name);
                        card_series = process_string(card_series);
                        match characters.par_iter().find_any(|character| {
                            if sub_find(card_name.clone(), character.name.clone(), &is_dot_name)
                                && sub_find(
                                    card_series.clone(),
                                    character.series.clone(),
                                    &is_dot_series,
                                )
                            {
                                true
                            } else {
                                false
                            }
                        }) {
                            None => {
                                dbg!(card.clone())
                            }
                            Some(found_card) => {
                                let mut c = card.clone();
                                c.wl = found_card.wl;
                                c
                            }
                        }
                    })
                    .collect::<Vec<Character>>();
                return_sender.send(found).await.unwrap();
            }
            CardsHandleType::UpdateCard(card) => {
                let mut mutab_card = card.clone();
                let is_dot_name = mutab_card.name.ends_with("...");
                let is_dot_series = mutab_card.series.ends_with("...");
                mutab_card.name = process_string(mutab_card.name);
                mutab_card.series = process_string(mutab_card.series);
                match characters.par_iter_mut().find_any(|character| {
                    if sub_find(
                        mutab_card.name.clone(),
                        character.name.clone(),
                        &is_dot_name,
                    ) && sub_find(
                        mutab_card.series.clone(),
                        character.series.clone(),
                        &is_dot_series,
                    ) {
                        true
                    } else {
                        false
                    }
                }) {
                    None => {
                        if (!is_dot_name) && (!is_dot_series) {
                            characters.push(mutab_card.clone());
                            utils_sender
                                .send(UtilType::DatabaseHandler(DatabaseHandleType::UpdateCard(
                                    mutab_card,
                                )))
                                .await
                                .unwrap();
                        }
                    }
                    Some(found_card) => {
                        found_card.wl = mutab_card.wl;
                        utils_sender
                            .send(UtilType::DatabaseHandler(DatabaseHandleType::UpdateCard(
                                found_card.clone(),
                            )))
                            .await
                            .unwrap();
                    }
                };
            }
        }
    }
}

fn process_string(mut s: String) -> String {
    s = s.to_lowercase();
    s.retain(|c| !c.is_whitespace() && c != '!');
    s.replace("...", "").replace("'", "").replace("’", "")
    //.replace(".-", "")
}
static BALANCERS: &[&[char]] = &[
    &['|'],
    &['’'],
    &['o', '0'],
    &['l', 'i'],
    &['1', ']'],
    &['y', 'v'],
    &['$', 's'],
    &['i', '!'],
    &['s', '5'],
    &['©', 'o'],
    &['1', 'i'],
    &['a', 'é'],
];

fn sub_find(mut card: String, mut character: String, is_dot: &bool) -> bool {
    if character.contains("★") {
        character = character.replace("★", "");
    }
    let mut diff_chars = vec![];
    let is_dot_char = character.contains("...");
    character = character.replace("...", "");
    card.clone()
        .chars()
        .zip(character.clone().chars())
        .enumerate()
        .for_each(|(i, (c1, c2))| {
            if c1 != c2 {
                if diff_chars.len() > 1 {
                    return;
                }
                diff_chars.push(c1);
                diff_chars.push(c2);
                character.remove(i);
                card.remove(i);
            }
        });
    if diff_chars.len() / 2 > 1 {
        return false;
    }
    if diff_chars.len() / 2 == 1 {
        let balanced = BALANCERS.iter().any(|balancer| {
            balancer
                .iter()
                .all(|balancer_char| diff_chars.contains(balancer_char))
        });
        if balanced != true {
            return false;
        }
    }
    let (longer, shorter) = longest_string(card.clone(), character.clone());
    if *is_dot && is_dot_char {
        if longer.starts_with(&shorter) {
            return true;
        }
        return false;
    } else if *is_dot && !is_dot_char {
        if longer == shorter {
            return false; // can cause problems, delete if needed
        }
        if character.starts_with(&card) {
            return true;
        }
        return false;
    } else if (!*is_dot) && is_dot_char {
        if card.starts_with(&character) {
            return true;
        }
        return false;
    } else {
        if card == character {
            return true;
        }
        return false;
    }
}
fn longest_string(str1: String, str2: String) -> (String, String) {
    if str1.len() > str2.len() {
        (str1, str2)
    } else {
        (str2, str1)
    }
}
