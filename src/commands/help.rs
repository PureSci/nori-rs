use serenity::{
    builder::{CreateApplicationCommands},
    client::Context,
    model::{
        prelude::{ interaction::application_command::ApplicationCommandInteraction},
        application::interaction::InteractionResponseType,
    },
};


pub async fn run(interaction: &ApplicationCommandInteraction, ctx: &Context) {
    interaction.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| {
                    msg.embed(|e| {
                        e.title("")
                            .color(15641224)
                            .author(|a| {
                                a.name("Nori - Help")
                                   .icon_url(interaction.user.static_avatar_url().clone().unwrap())
                            }).description("Nori is designed to assist SOFI players for no cost forever and ever without any premium features.")
                            .fields(vec![
                                (
                                    "<:line:1071888617913470996> Features",
                                    "> `Wishlist Analysis` for Drops / Server Drops.
                                    > `Reminders` for drop / grab.",
                                    false
                                ),
                                (
                                    "<:line:1071888617913470996> Commands",
                                    "> `/config`: Config your preferences. 
                                    > `/serverconfig` : Config your default server preferences.",
                                    false
                                ),
                                (
                                    "<:line:1071888617913470996> Context Menus",
                                    "> `delete`: Allows users to delete their own SOFI messages without needing permission.
                                    > **Usage:** Right click on any `SOFI Message` > `Apps` > `Delete Message`
                                    > *delete must be enabled by a server admin under the utils serverconfig to be used*.",
                                    false
                                ),
                            ])
                    }).components(|c| 
                        c.create_action_row(|ar|
                            ar.create_button(|b|b.label("Support").style(serenity::model::application::component::ButtonStyle::Link).url("https://discord.gg/3m2gYq8mUQ"))
                        )
                    )
            })
    }).await.unwrap();
}

pub fn register(command: &mut CreateApplicationCommands) {
    command.create_application_command(|f| f.name("help").description("Shows features of the bot."));
}