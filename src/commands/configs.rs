use std::time::Duration;

use crate::commands::config_utils::create_config_response;
use crate::commands::config_utils::CONFIGS;
use crate::utils::config_data::get_config_data;
use crate::utils::config_data::get_value;
use crate::utils::database_handler::DatabaseHandleType;
use crate::utils::handler::UtilType;
use serenity::builder::CreateApplicationCommands;
use serenity::client::Context;
use serenity::model::prelude::component::ComponentType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::model::prelude::interaction::InteractionResponseType;

use crate::PassData;
pub async fn run_config_command(interaction: &ApplicationCommandInteraction, context: &Context) {
    let pd = context.data.read().await;
    let pass_data = pd.get::<PassData>().unwrap();
    let response = create_config_response(
        &CONFIGS["reminders"],
        Some(interaction.user.id.to_string()),
        interaction.guild_id.unwrap(),
        &pass_data.db,
    )
    .await;
    interaction
        .create_interaction_response(&context.http, |res| {
            res.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.set_embed(response.embed)
                        .set_components(response.components)
                })
        })
        .await
        .unwrap();
}

pub async fn run_serverconfig_command(
    interaction: &ApplicationCommandInteraction,
    context: &Context,
) {
    let pd = context.data.read().await;
    let pass_data = pd.get::<PassData>().unwrap();
    let response = create_config_response(
        &CONFIGS["reminders"],
        None,
        interaction.guild_id.unwrap(),
        &pass_data.db,
    )
    .await;
    interaction
        .create_interaction_response(&context.http, |res| {
            res.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.set_embed(response.embed)
                        .set_components(response.components)
                })
        })
        .await
        .unwrap();
}

pub async fn handle_config_component(
    component: &mut MessageComponentInteraction,
    context: &Context,
) {
    let pd = context.data.read().await;
    let pass_data = pd.get::<PassData>().unwrap();
    match component.data.component_type {
        ComponentType::SelectMenu => {
            let userid = match component
                .data
                .custom_id
                .split_once("_")
                .unwrap()
                .1
                .split_once("_")
                .unwrap()
                .0
            {
                "user" => Some(component.user.id.to_string()),
                _ => None,
            };
            let response = create_config_response(
                &CONFIGS[component.data.values.first().unwrap()],
                userid,
                component.guild_id.unwrap(),
                &pass_data.db,
            )
            .await;
            component
                .message
                .edit(&context.http, |message| {
                    message
                        .set_embed(response.embed)
                        .set_components(response.components)
                })
                .await
                .unwrap();
            component.defer(&context.http).await.unwrap();
            // dbg!(component.data.values.first().unwrap()); "reminders"
        }
        ComponentType::Button => {
            let data = component.data.custom_id.split("_").collect::<Vec<&str>>();
            let mut userid = None;
            let mut id = component.guild_id.unwrap().to_string();
            if data[2] == "user" {
                let uid = component.user.id.to_string();
                userid = Some(uid.clone());
                id = uid;
            }
            let config_dat = &get_config_data(
                &pass_data.db,
                data[1],
                userid.clone(),
                component.guild_id.unwrap().to_string(),
                data[3],
            )
            .await;
            let mut current_data = None;
            // if it isnt server default
            if (!get_value(config_dat, "server_default")
                .parse::<bool>()
                .unwrap())
                || userid.is_none()
            {
                current_data = Some(get_value(config_dat, "value"));
            }
            pass_data
                .utils_sender
                .send(UtilType::DatabaseHandler(
                    DatabaseHandleType::SetConfigData(
                        data[1].to_string(),
                        data[2].to_string(),
                        data[3].to_string(),
                        id,
                        current_data,
                    ),
                ))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
            let response = create_config_response(
                &CONFIGS[data[1]],
                userid,
                component.guild_id.unwrap(),
                &pass_data.db,
            )
            .await;
            component
                .message
                .edit(&context.http, |message| {
                    message
                        .set_embed(response.embed)
                        .set_components(response.components)
                })
                .await
                .unwrap();
            component.defer(&context.http).await.unwrap();
            // dbg!(component.data.custom_id.clone()); config_analysis_user_enabled_1680468322
        }
        _ => {}
    }
}

pub fn register_serverconfig(command: &mut CreateApplicationCommands) {
    command.create_application_command(|f| {
        f.name("serverconfig")
            .description("Opens the serverconfig menu.")
    });
}
pub fn register_config(command: &mut CreateApplicationCommands) {
    command.create_application_command(|f| f.name("config").description("Opens the config menu."));
}
