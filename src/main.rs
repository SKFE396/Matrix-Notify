use tokio::{
    self, time::{sleep, Duration},
};
use matrix_sdk::{
    config::SyncSettings, room::Room, ruma::{
        api::client::{filter::FilterDefinition, sync::sync_events::v3::Filter},
        events::room::message::{
            MessageType, RoomMessageEventContent, TextMessageEventContent
        }, ServerName, UserId,
    }, AuthSession, Client
};
use std::{
    env::args, fs::{File, OpenOptions}, process::exit,
};
use serde::Deserialize;
use anyhow::Result;

#[derive(Deserialize, Debug)]
struct Config {
    server_name: String,
    user_id: String,
    password: String,
    room_name: String,
}

async fn send_loop(room: &Room, message: &str) -> bool {
    let message = RoomMessageEventContent::new(MessageType::Text(TextMessageEventContent::plain(message)));
    for i in 0..3 {
        match room.send(message.clone()).await {
            Ok(_) => return true,
            Err(e) => {
                println!("[mnot] [{}] failed: {:?}", i + 1, e);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    false
}

fn load_session(session_file: &str) -> Result<AuthSession> {
    let session = serde_json::from_reader(File::open(session_file)?)?;
    Ok(AuthSession::Matrix(session))
}

fn save_session(session_file: &str, session: AuthSession) -> Result<()> {
    match session {
        AuthSession::Matrix(session) => {
            serde_json::to_writer(OpenOptions::new().create(true).truncate(true).write(true).open(session_file)?, &session)?;
        },
        _ => panic!("unknown session type."),
    }
    Ok(())
}

async fn try_notify(config: &Config, session_file: &str, user_id: &UserId, message: &str) -> Result<bool> {
    let client = Client::builder().server_name(&ServerName::parse(&config.server_name)?).build().await?;

    // First check if we can restore the session from the session file.
    let mut need_auth = true;
    if let Ok(session) = load_session(&session_file) {
        if let Err(e) = client.restore_session(session).await {
            println!("[mnot] Error restoring session: {:?}", e);
        } else {
            need_auth = false;
        }
    } else {
        println!("[mnot] Cannot load the session file.");
    }
    if need_auth {
        client.matrix_auth().login_username(user_id, &config.password).send().await?;
    }
    // Save session
    if let Err(e) = save_session(&session_file, client.session().expect("expected a session")) {
        println!("[mnot] Failed to save the session: {:?}", e);
    }

    // Start the main logic
    let filter = FilterDefinition::ignore_all();
    let filter = Filter::FilterDefinition(filter);
    client.sync_once(SyncSettings::new().filter(filter)).await?;
    let mut target_room = None;
    for room in client.joined_rooms() {
        if room.name().as_ref() == Some(&config.room_name) {
            target_room = Some(room);
            break;
        }
    }

    // write!(OpenOptions::new().create(true).write(true).truncate(true).open("./out.log")?, "Rooms: {:#?}", rooms)?;
    if target_room.is_none() {
        println!("[mnot] No room with the specified name.");
        exit(1);
    }
    let room = target_room.unwrap();

    // Format the message
    let success = send_loop(&room, message).await;
    Ok(success)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut arguments = args();
    arguments.next().unwrap();
    let config_file = arguments.next().expect("expected config file as the first argument");
    let session_file = arguments.next().expect("expected session file as the second argument");
    let message = arguments.next().expect("expected the message as the third argument");

    let config: Config = serde_yaml::from_reader(File::open(&config_file)?)?;

    let user_id = UserId::parse(&config.user_id)?;

    for t in 0..20 {
        match try_notify(&config, &session_file, &user_id, &message).await {
            Ok(true) => {
                exit(0);
            },
            Ok(false) => {
                println!("[mnot] Trial {} failed. Retrying later.", t + 1);
            }
            Err(e) => {
                println!("[mnot] Trial {} failed with error {:?}. Retrying later.", t + 1, e);
            },
        };
        sleep(Duration::from_secs(60)).await;
    }
    println!("[mnot] Failed to send the notification. Giving up.");

    Ok(())
}
