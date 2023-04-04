use std::sync::Arc;

use async_channel::{unbounded, Receiver, Sender};
use image::{DynamicImage, ImageFormat};
use mongodb::Database;
use serenity::{
    http::Http,
    model::prelude::{Message, UserId},
};

use super::{
    cards_handler::{cards_handler_loop, CardsHandleType, Character},
    config_data::{get_config_data, get_value},
    database_handler::{database_handler_loop, DatabaseHandleType},
    ocr_captcha::captcha_ocr_loop,
    ocr_drop::drop_ocr_loop,
    ocr_series::series_ocr_loop,
    reminders::{get_name, reminder_loop, ReminderData, ReminderTime, ReminderType},
    series_handler::{series_handler_loop, Series, SeriesHandleType},
};

pub enum UtilType {
    CardsHandler(CardsHandleType),
    OcrDrop(DynamicImage, bool, Sender<[Character; 3]>),
    SetReminder(ReminderType, ReminderData),
    SeriesHandler(SeriesHandleType),
    OcrCaptcha(DynamicImage, Sender<[Character; 1]>),
    OcrSeries(DynamicImage, Sender<[Character; 2]>),
    DatabaseHandler(DatabaseHandleType),
}
pub async fn handler_loop(
    utils_receiver: Receiver<UtilType>,
    db: Database,
    cloud_db: Database,
    http: Arc<Http>,
    utils_sender: Sender<UtilType>,
) {
    let (drop_ocr_sender, drop_ocr_receiver) = unbounded();
    let (find_card_sender, find_card_receiver) = unbounded();
    let (reminder_sender, reminder_receiver) = unbounded();
    let (find_series_sender, find_series_receiver) = unbounded();
    let (captcha_ocr_sender, captcha_ocr_receiver) = unbounded();
    let (series_ocr_sender, series_ocr_receiver) = unbounded();
    let (database_handler_sender, database_handler_receiver) = unbounded();
    tokio::spawn(drop_ocr_loop(drop_ocr_receiver));
    tokio::spawn(cards_handler_loop(
        find_card_receiver,
        db.clone(),
        utils_sender.clone(),
        http.clone(),
    ));
    tokio::spawn(reminder_loop(reminder_receiver, http));
    tokio::spawn(series_handler_loop(
        find_series_receiver,
        db.clone(),
        utils_sender,
    ));
    tokio::spawn(captcha_ocr_loop(captcha_ocr_receiver));
    tokio::spawn(series_ocr_loop(series_ocr_receiver));
    tokio::spawn(database_handler_loop(
        database_handler_receiver,
        db,
        cloud_db,
    ));
    loop {
        let received = utils_receiver.recv().await.unwrap();
        match received {
            UtilType::CardsHandler(cards_handle_type) => {
                find_card_sender.send(cards_handle_type).await.unwrap();
            }
            UtilType::OcrDrop(im, show_gen, return_sender) => {
                drop_ocr_sender
                    .send((im, show_gen, return_sender))
                    .await
                    .unwrap();
            }
            UtilType::SetReminder(typ, data) => {
                reminder_sender.send((typ, data)).await.unwrap();
            }
            UtilType::SeriesHandler(series_handler_type) => {
                find_series_sender.send(series_handler_type).await.unwrap();
            }
            UtilType::OcrCaptcha(im, return_sender) => {
                captcha_ocr_sender.send((im, return_sender)).await.unwrap();
            }
            UtilType::OcrSeries(im, return_sender) => {
                series_ocr_sender.send((im, return_sender)).await.unwrap();
            }
            UtilType::DatabaseHandler(database_handle_type) => {
                database_handler_sender
                    .send(database_handle_type)
                    .await
                    .unwrap();
            }
        }
    }
}

pub async fn ocr_drop(
    utils_sender: &Sender<UtilType>,
    link: String,
    show_gen: bool,
) -> [Character; 3] {
    let (return_sender, return_receiver) = unbounded();
    //let a = Instant::now();
    let bytes = reqwest::get(link).await.unwrap().bytes().await.unwrap();
    let im = image::load_from_memory_with_format(&bytes, ImageFormat::WebP).unwrap();
    drop(bytes);
    //im = im.adjust_contrast(-3.0);
    utils_sender
        .send(UtilType::OcrDrop(im, show_gen, return_sender))
        .await
        .unwrap();
    return_receiver.recv().await.unwrap()
}

pub async fn ocr_captcha(utils_sender: &Sender<UtilType>, link: String) -> [Character; 1] {
    let (return_sender, return_receiver) = unbounded();
    let bytes = reqwest::get(link).await.unwrap().bytes().await.unwrap();
    let im = image::load_from_memory_with_format(&bytes, ImageFormat::Png).unwrap();
    drop(bytes);
    utils_sender
        .send(UtilType::OcrCaptcha(im, return_sender))
        .await
        .unwrap();
    return_receiver.recv().await.unwrap()
}

pub async fn ocr_series(utils_sender: &Sender<UtilType>, link: String) -> [Character; 2] {
    let (return_sender, return_receiver) = unbounded();
    let bytes = reqwest::get(link).await.unwrap().bytes().await.unwrap();
    let im = image::load_from_memory_with_format(&bytes, ImageFormat::Png).unwrap();
    drop(bytes);
    utils_sender
        .send(UtilType::OcrSeries(im, return_sender))
        .await
        .unwrap();
    return_receiver.recv().await.unwrap()
}

pub async fn find_cards(
    utils_sender: &Sender<UtilType>,
    characters: Vec<Character>,
) -> Vec<Character> {
    let (return_sender, return_receiver) = unbounded();
    utils_sender
        .send(UtilType::CardsHandler(CardsHandleType::FindCard(
            characters,
            return_sender,
        )))
        .await
        .unwrap();
    return_receiver.recv().await.unwrap()
}

pub async fn find_series(utils_sender: &Sender<UtilType>, series: [Series; 3]) -> [Series; 3] {
    let (return_sender, return_receiver) = unbounded();
    utils_sender
        .send(UtilType::SeriesHandler(SeriesHandleType::FindSeries(
            series,
            return_sender,
        )))
        .await
        .unwrap();
    return_receiver.recv().await.unwrap()
}

pub async fn set_reminder(
    utils_sender: &Sender<UtilType>,
    db: &Database,
    message: &Message,
    userid: &str,
    typ: ReminderTime,
) -> () {
    let config_data = get_config_data(
        db,
        "reminders",
        Some(userid.to_string()),
        message.guild_id.unwrap().to_string(),
        &get_name(typ),
    )
    .await;
    let reminder_config_data = get_value(&config_data, "value");
    let is_dm: bool;
    match reminder_config_data.as_str() {
        "true" => is_dm = false,
        "dm" => is_dm = true,
        _ => return,
    }
    utils_sender
        .send(UtilType::SetReminder(
            ReminderType(UserId(userid.parse::<u64>().unwrap()), typ),
            ReminderData(message.channel_id, is_dm),
        ))
        .await
        .unwrap();
}
