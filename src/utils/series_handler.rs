use async_channel::{Receiver, Sender};
use mongodb::{bson::doc, Database};
use rayon::prelude::*;

use serde::Deserialize;

use super::database_handler::DatabaseHandleType;
use super::handler::UtilType;

#[derive(Debug, Deserialize, Clone)]
pub struct Series {
    pub name: String,
    pub wl: Option<u32>,
}

pub enum SeriesHandleType {
    FindSeries([Series; 3], Sender<[Series; 3]>),
    UpdateSeries(Series),
}

pub async fn series_handler_loop(
    //receiver: Receiver<([Series; 3], Sender<[Series; 3]>)>,
    receiver: Receiver<SeriesHandleType>,
    db: Database,
    utils_sender: Sender<UtilType>,
) {
    let collection = db.collection::<Series>("analysis_series");
    let mut cursor = collection.find(None, None).await.unwrap();
    let mut series = vec![];
    while cursor.advance().await.unwrap() {
        series.push(cursor.deserialize_current().unwrap());
    }
    loop {
        //let (cards, return_sender) = receiver.recv().await.unwrap();
        match receiver.recv().await.unwrap() {
            SeriesHandleType::FindSeries(cards, return_sender) => {
                let found = (0..3)
                    .into_par_iter()
                    .map(|i| {
                        let card = &cards[i].name;
                        //card.retain(|c| !c.is_whitespace());
                        let is_dot_card = card.ends_with("...");
                        match series.par_iter().find_any(|serie| {
                            let is_dot_serie = serie.name.ends_with("...");
                            if is_dot_serie && is_dot_card {
                                if serie.name.len() > card.len() {
                                    return serie.name.starts_with(card);
                                } else {
                                    return card.starts_with(&serie.name);
                                }
                            } else if is_dot_serie && !is_dot_card {
                                return card.starts_with(&serie.name);
                            } else if (!is_dot_serie) && is_dot_card {
                                return serie.name.starts_with(card);
                            } else {
                                return &serie.name == card;
                            }
                        }) {
                            None => cards[i].clone(),
                            Some(found) => found.to_owned(),
                        }
                    })
                    .collect::<Vec<Series>>();
                let return_data: [Series; 3] = std::array::from_fn(|i| found[i].clone());
                return_sender.send(return_data).await.unwrap();
            }
            SeriesHandleType::UpdateSeries(cardo) => {
                let card = &cardo.name;
                let is_dot_card = card.ends_with("...");
                match series.par_iter_mut().find_any(|serie| {
                    let is_dot_serie = serie.name.ends_with("...");
                    if is_dot_serie && is_dot_card {
                        if serie.name.len() > card.len() {
                            return serie.name.starts_with(card);
                        } else {
                            return card.starts_with(&serie.name);
                        }
                    } else if is_dot_serie && !is_dot_card {
                        return card.starts_with(&serie.name);
                    } else if (!is_dot_serie) && is_dot_card {
                        return serie.name.starts_with(card);
                    } else {
                        return &serie.name == card;
                    }
                }) {
                    None => {
                        if !is_dot_card {
                            series.push(cardo.clone());
                            utils_sender
                                .send(UtilType::DatabaseHandler(DatabaseHandleType::UpdateSeries(
                                    cardo,
                                )))
                                .await
                                .unwrap();
                        }
                    }
                    Some(found_series) => {
                        found_series.wl = cardo.wl;
                        utils_sender
                            .send(UtilType::DatabaseHandler(DatabaseHandleType::UpdateSeries(
                                found_series.clone(),
                            )))
                            .await
                            .unwrap();
                    }
                };
            }
        }
    }
}
