use dotenv::dotenv;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct AppConfig {
    #[arg(short, long, default_value = "localhost")]
    pub base_url: String,

    #[arg(short, long, default_value = "8083")]
    pub server_port: u16,
}

impl AppConfig {

    pub fn from_env_and_args() -> Self {
        dotenv().ok();
        Self::parse()
    }
}
