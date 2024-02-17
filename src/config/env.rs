use clap::Parser;
use dotenv::dotenv;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct AppConfig {
    #[arg(short, long, env("BASE_URL"), default_value = "localhost")]
    pub base_url: String,

    #[arg(short, long, env("SERVER_PORT"), default_value = "8080")]
    pub server_port: String,

    #[arg(short, long, env("PROTOCOL"), default_value = "https")]
    pub protocol: String,
}

impl AppConfig {
    pub fn from_env_and_args() -> Self {
        dotenv().ok();
        Self::parse()
    }
}
