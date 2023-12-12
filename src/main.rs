// bot perms: manage messages

mod file;

use std::fs::{create_dir_all, remove_dir_all};
use std::path::PathBuf;

use serenity::async_trait;
use serenity::builder::{CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateMessage};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::{all, prelude::*};

struct Handler;

const DISCORD_CDN: &str = "https://cdn.discordapp.com";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let mut redacted_files = Vec::new();
        let mut has_token = false;
        for attachment in &msg.attachments {
            if attachment.filename.starts_with("hs_err_pid") {
                let log_folder = PathBuf::from(format!("{}", msg.id));
                if let Err(why) = create_dir_all(&log_folder) {
                    println!("Couldn't create file directory: {why}")
                };
                has_token = true;
                match file::redact(&attachment.url).await {
                    Ok(x) => redacted_files.push(x),
                    Err(x) => {
                        println!("{x}");
                        return;
                    }
                }
            }
        }
        if has_token {
            if let Err(why) = msg.delete(&ctx.http).await {
                println!("Couldn't delete message: {why}")
            };
            let log_folder = PathBuf::from(format!("{}", msg.id));
            let user_avatar = match msg.author.avatar {
                Some(x) => format!("{}/avatars/{}/{}.png", DISCORD_CDN, &msg.author.id, x),
                None => {
                    let index = match msg.author.discriminator {
                        Some(discriminator) => discriminator.get() as u64 % 5,
                        None => (msg.author.id.get() >> 22) % 6,
                    };
                    format!("{}/embed/avatars/{}.png", DISCORD_CDN, index)
                }
            };
            let user_color = match msg.author.accent_colour {
                Some(x) => x,
                None => all::Color::BLURPLE,
            };
            let paths = redacted_files
                .iter()
                .map(|x| CreateAttachment::bytes(x.as_bytes(), "log.txt"));
            let embed = CreateEmbed::new()
                .description(&msg.content)
                .color(user_color)
                .author(CreateEmbedAuthor::new(&msg.author.name).icon_url(user_avatar));
            if let Err(why) = msg
                .channel_id
                .send_files(&ctx.http, paths, CreateMessage::new().embed(embed))
                .await
            {
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
    let token = std::env::var("CAFFEINE_DISCORD_BOT")
        .expect("Please add your token to the 'CAFFEINE_DISCORD_BOT' environment variable");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
