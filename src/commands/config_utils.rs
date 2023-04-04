use mongodb::Database;
use once_cell::sync::Lazy;
use serenity::{
    builder::{CreateComponents, CreateEmbed},
    model::prelude::{component::ButtonStyle, EmojiId, GuildId, ReactionType},
};
use std::ops::Index;

use crate::{
    drop::uppercase_first,
    utils::{
        config_data::{get_config_data, get_value_from_all},
        reminders::current_time,
    },
};

const ON_EMOJI: &str = "<:on:1063161763786391572>";
const OFF_EMOJI: &str = "<:off:1063161785777135718>";

/*  */
pub struct Configs<'a> {
    pub reminders: ConfigType<'a>,
    pub analysis: ConfigType<'a>,
    //pub utils: ConfigType<'a>,
}

static CONFIG_TYPES: &[&str; 2] = &["reminders", "analysis" /* "utils" */];

impl<'a> Index<&str> for Configs<'a> {
    type Output = ConfigType<'a>;
    fn index(&self, field_name: &str) -> &ConfigType<'a> {
        match field_name {
            "reminders" => &self.reminders,
            "analysis" => &self.analysis,
            //"utils" => &self.utils,
            _ => &self.reminders,
        }
    }
}
/*  */

#[derive(Debug)]
pub struct ConfigType<'a> {
    pub name: &'a str,
    pub emoji: ReactionType,
    pub server_only: bool,
    pub options: Vec<ConfigOption<'a>>,
}
#[derive(Debug)]
pub struct ConfigOption<'a> {
    pub name: &'a str,
    pub fancy_name: &'a str,
    pub options: Vec<SubConfigOption<'a>>,
    pub description: Option<&'a str>,
}
#[derive(Debug)]
pub struct SubConfigOption<'a> {
    pub name: &'a str,
    pub emoji: &'a str,
    pub text: &'a str,
}

impl<'a> SubConfigOption<'a> {
    const TRUE: Self = SubConfigOption {
        name: "true",
        emoji: ON_EMOJI,
        text: "Enabled",
    };
    const FALSE: Self = SubConfigOption {
        name: "false",
        emoji: OFF_EMOJI,
        text: "Disabled",
    };
}

pub struct ConfigResponse {
    pub embed: CreateEmbed,
    pub components: CreateComponents,
}

pub static CONFIGS: Lazy<Configs> = Lazy::new(|| {
    Configs {
        reminders: ConfigType {
            name: "reminders",
            emoji: ReactionType::from(EmojiId(1061639553154285669)),
            server_only: false,
            options: vec![
                ConfigOption {
                    name: "drop",
                    fancy_name: "Drop",
                    options: vec![
                        SubConfigOption::TRUE,
                        SubConfigOption {
                            name: "dm",
                            emoji: ON_EMOJI,
                            text: "Enabled DM",
                        },
                        SubConfigOption::FALSE,
                    ],
                    description: None,
                },
                ConfigOption {
                    name: "grab",
                    fancy_name: "Grab",
                    options: vec![
                        SubConfigOption::TRUE,
                        SubConfigOption {
                            name: "dm",
                            emoji: ON_EMOJI,
                            text: "Enabled DM",
                        },
                        SubConfigOption::FALSE,
                    ],
                    description: None,
                },
            ],
        },
        analysis: ConfigType {
            name: "analysis",
            emoji: ReactionType::from('🔍'),
            server_only: false,
            options: vec![
                ConfigOption {
                    name: "enabled",
                    fancy_name: "Analysis",
                    options: vec![SubConfigOption::TRUE, SubConfigOption::FALSE],
                    description: None,
                },
                ConfigOption {
                    name: "expire-delete",
                    fancy_name: "Delete analysis after drop expires",
                    options: vec![SubConfigOption::TRUE, SubConfigOption::FALSE],
                    description: None,
                },
                ConfigOption {
                    name: "show-gen",
                    fancy_name: "Show Gen",
                    options: vec![SubConfigOption::TRUE, SubConfigOption::FALSE],
                    description: None,
                },
                ConfigOption {
                    name: "ping",
                    fancy_name: "Ping Me",
                    options: vec![SubConfigOption::TRUE, SubConfigOption::FALSE],
                    description: None,
                },
                ConfigOption {
                    name: "show-time",
                    fancy_name: "Show Generated in Miliseconds",
                    options: vec![SubConfigOption::TRUE, SubConfigOption::FALSE],
                    description: None,
                },
                /*ConfigOption {
                    name: "type",
                    fancy_name: "Message Type",
                    options: vec![SubConfigOption {
                        name: "1",
                        text: "Default",
                        emoji: "1️⃣"
                    }, SubConfigOption {
                        name: "2",
                        text: "Basic",
                        emoji: "2️⃣"
                    }],
                    description: None,
                },*/
            ],
        },
        /*utils: ConfigType {
            name: "utils",
            emoji:ReactionType::from('💡'),
            server_only: true,
            options: vec![ConfigOption {
                name: "delete-message",
                fancy_name: "Delete Message Feature",
                options: vec![SubConfigOption::TRUE, SubConfigOption::FALSE],
                description: Some("Enabling this option allows the users to delete their Sofi messages using Discord's new apps function. Nori needs the `ManageMessages` permission for this feature."),
            }],
        },*/
    }
});

pub async fn create_config_response<'a>(
    config_type: &ConfigType<'a>,
    userid: Option<String>,
    guildid: GuildId,
    db: &Database,
) -> ConfigResponse {
    let is_server = userid.is_none();
    let mut config_u_type = "user";
    if is_server {
        config_u_type = "server";
    }
    let config_data =
        get_config_data(db, config_type.name, userid, guildid.to_string(), "_all").await;
    let mut embed = CreateEmbed::default();
    let mut components = CreateComponents::default();
    components.create_action_row(|row| {
        row.create_select_menu(|menu| {
            menu.custom_id(format!(
                "configTypeSelector_{}_{}",
                config_u_type,
                current_time()
            ))
            .options(|menu_options| {
                for typ_str in CONFIG_TYPES {
                    let typ_obj = &CONFIGS[typ_str];
                    if !(typ_obj.server_only && !is_server) {
                        let name = uppercase_first(&typ_obj.name.to_string());
                        let is_default = typ_obj.name == config_type.name;
                        menu_options.create_option(|menu_option| {
                            menu_option
                                .emoji(typ_obj.emoji.clone())
                                .description(format!("Config of {}.", name))
                                .label(name)
                                .value(typ_obj.name)
                                .default_selection(is_default)
                        });
                    }
                }
                menu_options
            })
        })
    });
    let computed = compute_values(config_type.options.len());
    let mut i: usize = 1;
    for _ in 0..computed.0 {
        components.create_action_row(|row| {
            for _ in 0..computed.1 {
                match config_type.options.get(i - 1) {
                    Some(o) => {
                        row.create_button(|button| {
                            button
                                .label(i)
                                .style(ButtonStyle::Primary)
                                .custom_id(format!(
                                    "config_{}_{}_{}_{}",
                                    config_type.name,
                                    config_u_type,
                                    o.name,
                                    current_time()
                                ))
                        });
                    }
                    None => {}
                }
                i += 1;
            }
            row
        });
    }
    let mut options = vec![];
    for option in &config_type.options {
        let sub_option = option
            .options
            .iter()
            .find(|f| f.name == get_value_from_all(&config_data, option.name, "value").as_str())
            .unwrap();
        let mut server_default = "";
        if (!is_server)
            && get_value_from_all(&config_data, option.name, "server_default")
                .parse::<bool>()
                .unwrap()
        {
            server_default = " <Server Default>";
        }
        let mut description = "".to_string();
        if option.description.is_some() {
            description = format!("\n*{}*", option.description.unwrap());
        }
        options.push(format!(
            "`{}]` {} • `{}` • **{}**{}{}",
            options.len() + 1,
            sub_option.emoji,
            option.fancy_name,
            sub_option.text,
            server_default,
            description
        ));
    }
    embed
        .description(options.join("\n"))
        .color(15641224)
        .footer(|f| f.text("Use the Buttons below to toggle the attached option to it."))
        .title(format!(
            "{} {} Config",
            uppercase_first(&config_u_type.to_string()),
            uppercase_first(&config_type.name.to_string())
        ));
    ConfigResponse {
        embed: embed,
        components: components,
    }
}

fn compute_values(num: usize) -> (usize, usize) {
    let mut first = 1;
    let mut second = 2;

    while num > (first * second) {
        second += 1;
        if second > 5 {
            first += 1;
            second = 3;
        }
    }

    (first, second)
}
