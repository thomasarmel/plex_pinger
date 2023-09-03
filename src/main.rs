mod config;
mod plex_checker;

use crate::config::Config;
use serenity::model::id::{ChannelId, UserId};
use serenity::model::mention::Mention;

extern crate tokio;

#[tokio::main]
async fn main() {
    const DISCORD_PLEX_DOWN_MESSAGE: &'static str = "Plex is down!";

    let config = Config::read_from_arg_file();
    let plex_checker = plex_checker::PlexChecker::new(&config);

    let discord_channel_id = ChannelId(config.discord_bot.msg_channel_id);
    let http_discord_client =
        serenity::http::HttpBuilder::new(config.discord_bot.bot_token).build();

    if http_discord_client.get_current_user().await.is_err() {
        eprintln!("Error: Discord bot token is invalid");
        std::process::exit(3);
    }

    let mut last_plex_state_up = true;
    let mut last_reminder_time: Option<std::time::Instant> = None;

    loop {
        if !plex_checker.check_plex_up().await {
            if last_plex_state_up
                || last_reminder_time.is_none()
                    || last_reminder_time.unwrap().elapsed().as_secs()
                        > config.pinger_reminder_seconds
            {
                println!("Plex is down, sending Discord message");
                if discord_channel_id
                    .send_message(&http_discord_client, |m| {
                        match config.discord_bot.ping_user_id {
                            Some(user_id) => {
                                let ping_user = UserId::from(user_id);
                                m.content(format!(
                                    "{}, {}",
                                    Mention::from(ping_user),
                                    DISCORD_PLEX_DOWN_MESSAGE
                                ))
                            }
                            None => m.content(DISCORD_PLEX_DOWN_MESSAGE),
                        }
                    })
                    .await
                    .is_err()
                {
                    eprintln!("Error: Discord message failed to send");
                }
                last_reminder_time = Some(std::time::Instant::now());
            }
            last_plex_state_up = false;
        } else {
            last_plex_state_up = true;
            last_reminder_time = None;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(
            config.pinger_interval_seconds,
        ))
        .await;
    }
}
