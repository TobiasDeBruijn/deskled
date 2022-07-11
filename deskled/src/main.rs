use std::process::exit;
use tracing::error;
use driver::{Driver, Spidev};
use crate::config::Config;

mod config;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // CLion trips on the tokio::main macro, so we just defer to another function
    do_main().await;
}

async fn do_main() {
    setup_tracing();
    let config = match Config::get().await {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to open config: {e}");
            exit(1);
        }
    };

    let spidev = match Spidev::new() {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to open Spidev: {e}");
            exit(1);
        }
    };
    let driver = match Driver::new(&spidev, config.led.length) {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to create driver: {e}");
            exit(1);
        }
    };

    match ghome::start(ghome::Config {
        mysql_host: config.mysql.host,
        mysql_username: config.mysql.username,
        mysql_password: config.mysql.password,
        mysql_database: config.mysql.database,
        led_length: config.led.length,
        oauth2_client_id: config.oauth2.client_id,
        oauth2_client_secret: config.oauth2.client_secret,
        login_username: config.login.username,
        login_password: config.login.password
    }, driver).await {
        Ok(_) => {},
        Err(e) => {
            error!("Failed to start webserver: {e}");
            exit(1);
        }
    }
}

pub fn setup_tracing() {
    let sub = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(sub).expect("Setting tracing subscriber");
}