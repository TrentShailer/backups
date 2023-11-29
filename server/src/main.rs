mod config;

use config::load_config;
use log::{error, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(_) => warn!("No .env file found"), // Log to info as variables could already be in env
    };

    let config = match load_config() {
        Ok(config) => config,
        Err(error) => {
            error!("{}", error);
            anyhow::bail!("Error when loading config");
        }
    };

    let address = format!("0.0.0.0:{}", config.port);

    info!("Address: {0}", address);

    Ok(())
}
