use std::process::exit;
use tracing::{debug, error, info};
use driver::{Driver, Rgb};
use crate::cli::Cli;

mod cli;

fn main() {
    setup_tracing();
    info!("Welcome! v{}", env!("CARGO_PKG_VERSION"));

    debug!("Parsing CLI");
    let cli = Cli::new();

    debug!("Aquiring SPI device");
    let spidev = match if let Some(spidev) = cli.dev {
        driver::Spidev::new_with_name(spidev)
    } else {
        driver::Spidev::new()
    } {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to aquire SPI device. Is SPI enabled?: {e}");
            exit(1);
        }
    };

    debug!("Creating Driver");
    let mut driver = match Driver::new(&spidev, cli.length) {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to create driver: {e}");
            exit(1);
        }
    };

    let rgb = Rgb::new(
        cli.red.unwrap_or(0),
        cli.green.unwrap_or(0),
        cli.blue.unwrap_or(0)
    );

    debug!("Setting RGB");
    match driver.set_rgb(rgb) {
        Ok(_) => {},
        Err(e) => {
            error!("Failed to set RGB: {e}");
            exit(1);
        }
    }
}


pub fn setup_tracing() {
    let sub = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(sub).expect("Setting tracing subscriber");
}