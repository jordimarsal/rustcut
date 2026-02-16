use clap::Parser;
#[cfg(not(test))]
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
        // Do NOT load `.env` during tests to keep test environment deterministic.
        #[cfg(not(test))]
        {
            dotenv().ok();
        }

        // Remove common test-harness / tooling arguments (cargo test) before parsing so
        // running `cargo test -- --nocapture` doesn't confuse clap.
        let mut args: Vec<String> = std::env::args().collect();
        let test_flags = [
            "--quiet",
            "--nocapture",
            "--exact",
            "--ignored",
            "--test-threads",
            "--show-output",
        ];
        args.retain(|a| {
            if test_flags.iter().any(|f| a == f) {
                return false;
            }
            if a.starts_with("--color") || a.starts_with("--list") {
                return false;
            }
            true
        });

        Self::parse_from(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn from_env_and_args_respects_env_vars_and_defaults() {
        // ensure defaults when no env vars
        env::remove_var("BASE_URL");
        env::remove_var("SERVER_PORT");
        env::remove_var("PROTOCOL");

        let cfg = AppConfig::from_env_and_args();
        assert_eq!(cfg.base_url, "localhost");
        assert_eq!(cfg.server_port, "8080");

        // override via env
        env::set_var("BASE_URL", "example.com");
        env::set_var("SERVER_PORT", "9000");
        env::set_var("PROTOCOL", "http");

        let cfg2 = AppConfig::from_env_and_args();
        assert_eq!(cfg2.base_url, "example.com");
        assert_eq!(cfg2.server_port, "9000");
        assert_eq!(cfg2.protocol, "http");

        // cleanup
        env::remove_var("BASE_URL");
        env::remove_var("SERVER_PORT");
        env::remove_var("PROTOCOL");
    }

    #[test]
    fn from_env_and_args_ignores_test_harness_flags() {
        // Simply calling the function should not panic even if the test harness injected flags
        // such as `--nocapture` (the running test binary may include them).
        let cfg = AppConfig::from_env_and_args();
        assert!(!cfg.base_url.is_empty());
    }
}
