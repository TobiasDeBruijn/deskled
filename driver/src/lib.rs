use std::fmt;
use tracing::{info, trace};
use ws2818_rgb_led_spi_driver::adapter_gen::WS28xxAdapter;
use ws2818_rgb_led_spi_driver::adapter_spi::WS28xxSpiAdapter;
use ws2818_rgb_led_spi_driver::encoding::encode_rgb;
use error::Result;

mod error;
mod spidev;

pub use error::*;
pub use spidev::*;

pub struct Rgb((u8, u8, u8));

impl Rgb {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self((red, green, blue))
    }

    pub fn off() -> Self {
        Self((0, 0, 0))
    }
}

pub struct Driver {
    adapter: WS28xxSpiAdapter,
    length: u16
}

impl fmt::Debug for Driver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Driver {{ length: {} }}", self.length)
    }
}

impl Driver {
    pub fn new(spidev: &Spidev, length: u16) -> Result<Self> {
        trace!("Creating WS28xxSpiAdapter");
        let adapter = WS28xxSpiAdapter::new(&spidev.0).map_err(|e| Error::Ws28xx(e))?;
        Ok(Self {
            adapter,
            length
        })
    }

    pub fn set_rgb(&mut self, rgb: Rgb) -> Result<()> {
        trace!("Encoding RGB");
        let mut rgb_bits = Vec::with_capacity(self.length as usize * 48);
        let (r, g, b) = rgb.0;
        trace!("Setting R{r} G{g} B{b}");
        for _ in 0..self.length {
            // For some reason green and blue need to be swapped
            rgb_bits.extend_from_slice(&encode_rgb(r, b, g));
        }

        info!("Writing RGB");
        self.adapter.write_encoded_rgb(&rgb_bits).map_err(|e| Error::Ws28xx(e))?;
        Ok(())
    }
}