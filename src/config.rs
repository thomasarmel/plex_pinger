use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) plex: Plex,
    pub(crate) discord_bot: DiscordBot,
    pub(crate) pinger_interval_seconds: u64,
    pub(crate) pinger_reminder_seconds: u64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Plex {
    pub(crate) domain: String,
    pub(crate) ssl: bool,
    pub(crate) port: u16,
    pub(crate) plex_token: String,
    pub(crate) certificate_uuid: String,
    pub(crate) libraries: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct DiscordBot {
    pub(crate) bot_token: String,
    pub(crate) msg_channel_id: u64,
    pub(crate) ping_user_id: Option<u64>,
}

impl Config {
    pub(crate) fn read_from_arg_file() -> Self {
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 2 {
            eprintln!("Usage: {} <config_file>", args[0]);
            std::process::exit(1);
        }
        let config_file = &args[1];
        let config_file = match std::fs::read_to_string(config_file) {
            Ok(config_file) => config_file,
            Err(e) => {
                eprintln!("Error reading config file: {}", e);
                std::process::exit(1);
            }
        };
        let config: Config = match serde_json::from_str(&config_file) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error parsing config file: {}", e);
                std::process::exit(1);
            }
        };
        config
    }
}
