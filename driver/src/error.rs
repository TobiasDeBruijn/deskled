use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error in underlying driver: {0}")]
    Ws28xx(String),
    #[error("No SPI device could be found")]
    NoSpiDev,
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}