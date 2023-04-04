use mongodb::{
    bson::{doc, Document},
    options::UpdateOptions,
    Database,
};

use crate::commands::config_utils::CONFIGS;

use super::{cards_handler::Character, series_handler::Series};
use async_channel::Receiver;

pub enum DatabaseHandleType {
    UpdateCard(Character),
    UpdateSeries(Series),
    SetConfigData(String, String, String, String, Option<String>),
    /*
     * String    String String   String String
     * analysis  user   enabled  12303  true
     * category  sv/us  value    id     current value
     */
}

pub async fn database_handler_loop(
    receiver: Receiver<DatabaseHandleType>,
    db: Database,
    cloud_db: Database,
) {
    loop {
        match receiver.recv().await.unwrap() {
            DatabaseHandleType::UpdateCard(card) => {
                update_card(&db, &card).await;
                update_card(&cloud_db, &card).await;
            }
            DatabaseHandleType::UpdateSeries(series) => {
                update_series(&db, &series).await;
                update_series(&cloud_db, &series).await;
            }
            DatabaseHandleType::SetConfigData(category, typ, value, id, current_value) => {
                set_config_data(&db, &category, &typ, &value, &id, &current_value).await;
                set_config_data(&cloud_db, &category, &typ, &value, &id, &current_value).await;
            }
        }
    }
}

async fn set_config_data(
    db: &Database,
    category: &String,
    typ: &String,
    value: &String,
    id: &String,
    current_value: &Option<String>,
) {
    let options = &CONFIGS[category]
        .options
        .iter()
        .find(|f| f.name == value)
        .unwrap()
        .options;
    let mut set_or_unset = "$set";
    let set_value = match current_value {
        None => options.first().unwrap().name,
        Some(d) => match options.get(options.iter().position(|f| f.name == d).unwrap() + 1) {
            Some(a) => a.name,
            None => {
                if typ == "server" {
                    options.first().unwrap().name
                } else {
                    set_or_unset = "$unset";
                    ""
                }
            }
        },
    };
    db.collection::<Document>(format!("{typ}_config_{category}").as_str())
        .update_one(
            doc! {
                "_id": id,
            },
            doc! {
                set_or_unset: {
                    value: set_value,
            }
            },
            UpdateOptions::builder().upsert(true).build(),
        )
        .await
        .unwrap();
}

async fn update_card(db: &Database, card: &Character) {
    db.collection::<Document>("analysis_characters")
        .update_one(
            doc! {
                "name": &card.name,
                "series": &card.series
            },
            doc! {
                "$set": {
                    "wl": &card.wl.unwrap()
            }
            },
            UpdateOptions::builder().upsert(true).build(),
        )
        .await
        .unwrap();
}

async fn update_series(db: &Database, series: &Series) {
    db.collection::<Document>("analysis_series")
        .update_one(
            doc! {
                "name": &series.name,
            },
            doc! {
                "$set": {
                    "wl": &series.wl.unwrap()
            }
            },
            UpdateOptions::builder().upsert(true).build(),
        )
        .await
        .unwrap();
}
