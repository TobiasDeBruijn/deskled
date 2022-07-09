use crate::error::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::trace;

pub struct Spidev(pub(crate) String);

impl Spidev {
    pub fn new() -> Result<Self> {
        trace!("Reading directory '/dev', scanning for spi devices");
        let rd = fs::read_dir("/dev/")?;
        for entry in rd {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                continue;
            }

            let path = entry.path();
            let path_string = path.to_string_lossy();

            if path_string.contains("spi") {
                trace!("Found spi device {:?}", path_string);
                return Ok(Self(path_string.to_string()))
            }
        }

        trace!("Found no spi device");
        Err(Error::NoSpiDev)
    }

    pub fn new_with_name<S: AsRef<str>>(name: S) -> Result<Self> {
        trace!("Checking if provided spidev '{}' exists", name.as_ref());
        let path = Path::new(name.as_ref());
        if !path.exists() {
            return Err(Error::NoSpiDev);
        }

        Ok(Self(name.as_ref().to_string()))
    }
}
