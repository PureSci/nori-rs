mod commands;
mod operations;
mod utils;

use std::env;
use std::time::Duration;

pub use crate::operations::*;
use crate::utils::reminders::current_time;
use async_channel::Sender;
use mongodb::Database;
use mongodb::{options::ClientOptions, Client};
use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::MessageUpdateEvent;
use serenity::prelude::*;
use utils::handler::UtilType;
struct PassData {
    db: Database,
    utils_sender: Sender<UtilType>,
}
impl TypeMapKey for PassData {
    type Value = PassData;
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: serenity::model::prelude::Ready) {
        println!("{} is connected!", ready.user.name);
        Command::set_global_application_commands(&context.http, |command| {
            commands::help::register(command);
            commands::configs::register_config(command);
            commands::configs::register_serverconfig(command);
            command
        })
        .await
        .unwrap();
    }
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "help" => commands::help::run(&command, &context).await,
                "config" => commands::configs::run_config_command(&command, &context).await,
                "serverconfig" => {
                    commands::configs::run_serverconfig_command(&command, &context).await
                }
                _ => {}
            };
        } else if let Interaction::MessageComponent(mut component) = interaction {
            let id = &component.data.custom_id;
            if id.starts_with("config") {
                if current_time() - id.split("_").last().unwrap().parse::<u64>().unwrap() > 60 {
                    return;
                }
                commands::configs::handle_config_component(&mut component, &context).await;
            }
        }
    }

    async fn message(&self, context: Context, message: Message) {
        if message.author.bot && message.author.id != 853629533855809596 {
            return;
        }
        if drop::filter(&message) {
            drop::run(&message, &context, None).await;
        } else if eval::filter(&message) {
            eval::run(&message, &context).await;
        } else if minigame::filter(&message) {
            minigame::run(&message, &context).await;
        } else if captcha::filter(&message) {
            captcha::run(&message, &context).await;
        } else if series1::filter(&message) {
            series1::run(&message, &context).await;
        } else if series2::filter(&message) {
            series2::run(&message, &context).await;
        }
        /*this should always be at the end*/
        else if wishlist_updaters::filter(&message) == true {
            wishlist_updaters::run(&message, &context).await;
        }
    }
    async fn message_update(
        &self,
        context: Context,
        _: Option<Message>,
        _: Option<Message>,
        message: MessageUpdateEvent,
    ) {
        if wl_edit_updater::filter(&message) {
            wl_edit_updater::run(&message, &context).await;
        }
    }
}
#[tokio::main]
async fn main() {
    let db_url = "mongodb://localhost:27017/nori?retryWrites=true&w=majority".to_string();
    let cloud_db_url = env::var("DB_URL").unwrap();
    let db = create_database(db_url).await;
    let cloud_db = create_database(cloud_db_url).await;
    //let collection = db.collection::<Document>("jsons");
    let token = env::var("NORI_TOKEN").unwrap();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let (utils_sender, utils_receiver) = async_channel::unbounded();
    let mut client = serenity::Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    client.data.write().await.insert::<PassData>(PassData {
        db: db.clone(),
        utils_sender: utils_sender.clone(),
    });
    // starting here, without snding starting to it
    tokio::spawn(utils::handler::handler_loop(
        utils_receiver.clone(),
        db,
        cloud_db,
        client.cache_and_http.http.clone(),
        utils_sender,
    ));
    tokio::time::sleep(Duration::from_secs(4)).await;
    client.start().await.unwrap();
}

async fn create_database(url: String) -> Database {
    Client::with_options(ClientOptions::parse(url.clone()).await.unwrap())
        .unwrap()
        .database(
            url.split("/")
                .last()
                .unwrap_or_default()
                .split("?")
                .next()
                .unwrap_or_default(),
        )
}
