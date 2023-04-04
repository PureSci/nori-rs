use async_channel::Receiver;
use serenity::{
    http::Http,
    model::prelude::{ChannelId, UserId},
};
use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use std::{fs::File, io::BufReader};
use tokio::{
    task::JoinHandle,
    time::{sleep, Duration},
};

pub static REMINDER_EMOJI: &str = "<:reminder_set:1061639553154285669>";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReminderTime(pub u32);
#[allow(dead_code)]
impl ReminderTime {
    pub const DROP: Self = ReminderTime(480);
    pub const GRAB: Self = ReminderTime(240);
    pub const RAID: Self = ReminderTime(43200);
    pub const DEV: Self = ReminderTime(10);
}

pub fn get_name(t: ReminderTime) -> String {
    match t.0 {
        480 => "drop".to_string(),
        240 => "grab".to_string(),
        43200 => "raid".to_string(),
        _ => "drop".to_string(),
    }
}

pub struct ReminderData(pub ChannelId, pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReminderType(pub UserId, pub ReminderTime);
pub async fn reminder_loop(receiver: Receiver<(ReminderType, ReminderData)>, http: Arc<Http>) {
    let mut reminders: HashMap<ReminderType, (JoinHandle<()>, u64, ReminderData)> = HashMap::new();
    match File::open("reminders.pure") {
        Ok(f) => {
            let mut buf_reader = BufReader::new(f);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents).unwrap();
            let mut saved_vec = contents.split("&").collect::<Vec<&str>>();
            saved_vec.pop();
            for reminder in saved_vec {
                let data = reminder.split(",").collect::<Vec<&str>>();
                let start_time = data.get(2).unwrap().parse::<u64>().unwrap();
                let reminder_time_u = data.get(1).unwrap().parse::<u32>().unwrap();
                match (reminder_time_u as u64).checked_sub(current_time() - start_time) {
                    None => {}
                    Some(_) => {
                        let typ = ReminderType(
                            UserId(data.get(0).unwrap().parse::<u64>().unwrap()),
                            ReminderTime(reminder_time_u),
                        );
                        let channel_id = ChannelId(data.get(3).unwrap().parse::<u64>().unwrap());
                        let is_dm = data.get(4).unwrap().parse::<bool>().unwrap();
                        let function = tokio::spawn(start_reminder(
                            typ,
                            data.get(2).unwrap().parse::<u64>().unwrap(),
                            channel_id,
                            is_dm,
                            http.clone(),
                        ));
                        reminders
                            .insert(typ, (function, start_time, ReminderData(channel_id, is_dm)));
                    }
                }
            }
            /*
            0 // userid
            1 // reminderTime (drop,grab)
                    2    // startTime
                    3 // channelId
                    4  // is Dm */
        }
        Err(_) => {}
    }
    ChannelId(env::var("STATUS_CHANNEL").unwrap().parse::<u64>().unwrap())
        .send_message(&http, |m| {
            m.content(format!(
                "**Nori woke up!** <a:angury:1090035662037725295>\n\n> Recovered **{}** reminders.",
                reminders.len()
            ));
            m
        })
        .await
        .unwrap();
    loop {
        let (kind, data) = receiver.recv().await.unwrap();
        if kind.1 .0 != 0 {
            if reminders.contains_key(&kind) == true {
                reminders.remove(&kind).unwrap().0.abort();
            }
            let start_time = current_time();
            let function = tokio::spawn(start_reminder(
                kind,
                start_time,
                data.0,
                data.1,
                http.clone(),
            ));
            reminders.insert(kind, (function, start_time, data));
        } else {
            ChannelId(env::var("STATUS_CHANNEL").unwrap().parse::<u64>().unwrap())
                .send_message(&http, |m| {
                    m.content(format!("**Nori is preparing to sleep** <:sleep:1090035329555234917>\n\n> Attempting to save **{}** reminders...", reminders.len()));
                    m
                })
                .await
                .unwrap();
            let mut f = File::create("reminders.pure").unwrap();
            let mut b_str = String::new();
            for reminder in &reminders {
                b_str += format!(
                    "{},{},{},{},{}&",
                    reminder.0 .0,    // userid
                    reminder.0 .1 .0, // reminderTime (drop,grab)
                    reminder.1 .1,    // startTime
                    reminder.1 .2 .0, // channelId
                    reminder.1 .2 .1  // is Dm
                )
                .as_str();
            }
            f.write_all(b_str.as_bytes()).unwrap();
            println!("Reminders Saved to reminders.pure");
            std::process::exit(0);
        }
    }
}
#[allow(unused_must_use)]
pub async fn start_reminder(
    data: ReminderType,
    start_time: u64,
    channelid: ChannelId,
    is_dm: bool,
    http: Arc<Http>,
) {
    //let difference = current_time() - start_time;
    match (data.1 .0 as u64).checked_sub(current_time() - start_time) {
        None => {}
        Some(sleep_time) => {
            sleep(Duration::from_secs(sleep_time as u64)).await;
            if is_dm == true {
                data.0
                    .to_user(&http)
                    .await
                    .unwrap()
                    .direct_message(&http, |m| {
                        m.content(format!(
                            "{REMINDER_EMOJI} You can now **{}**! <#{}>",
                            get_name(data.1),
                            channelid
                        ))
                    })
                    .await;
            } else {
                channelid
                    .send_message(&http, |m| {
                        m.content(format!(
                            "{REMINDER_EMOJI} <@{}> you can now **{}**!",
                            data.0.to_string(),
                            get_name(data.1)
                        ))
                    })
                    .await
                    .unwrap();
            }
        }
    }
    //get_config_data(db, typ, user_id, guild_id, data, start)
    //userid.to_user(http).await.unwrap();
    //user.direct_message(http, f);
}

pub fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
