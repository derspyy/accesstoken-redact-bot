// bot perms: manage messages

mod file;

use std::path::PathBuf;
use std::fs::{remove_dir_all, create_dir_all};

use serenity::{async_trait, utils};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

const DISCORD_CDN: &str = "https://cdn.discordapp.com";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return
        }
        let mut has_token = false;
        for attachment in &msg.attachments {
            if attachment.filename.starts_with("hs_err_pid") {
                let log_folder = PathBuf::from(format!("{}", msg.id.0));
                if let Err(why) = create_dir_all(&log_folder) {
                    println!("Couldn't create file directory: {why}")
                };
                has_token = true;
                let filename = log_folder.join(&attachment.filename);
                if let Err(x) = file::redact(&attachment.url, &filename) {
                    println!("{x}");
                    return
                }
            }
        }
        if has_token {
            if let Err(why) = msg.delete(&ctx.http).await {
                println!("Couldn't delete message: {why}")
            };
            let log_folder = PathBuf::from(format!("{}", msg.id.0));
            let files: Vec<PathBuf> = log_folder.read_dir().unwrap().map(|f| f.unwrap().path()).collect();
            let user_avatar = match msg.author.avatar {
                Some(x) => format!("{}/avatars/{}/{}.png", DISCORD_CDN, &msg.author.id, x),
                None => format!("{}/embed/avatars/{}.png", DISCORD_CDN, &msg.author.discriminator % 5),
            };
            let user_color = match msg.author.accent_colour {
                Some(x) => x,
                None => utils::Color::BLURPLE,
            };
            let file_refs: Vec<&PathBuf> = files.iter().collect();
            if let Err(why) = msg.channel_id.send_files(&ctx.http, file_refs, |m| {
                m.embed(|e| {
                    e.description(&msg.content)
                        .color(user_color)
                        .author(|a| {
                            a.icon_url(user_avatar)
                                .name(&msg.author.name)  
                        })
                })
            }).await {
                println!("Couldn't send message: {why}")
            };
            if let Err(why) = remove_dir_all(log_folder) {
                println!("Couldn't cleanup: {why}")
            };
        }
    }
    
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env!("CAFFEINE_DISCORD_BOT");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}