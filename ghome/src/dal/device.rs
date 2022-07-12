use std::str::Chars;
use color_space::ToRgb;
use mysql::{Params, params, Row, Transaction};
use mysql::prelude::Queryable;
use crate::WebResult;

#[derive(Debug, Clone, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn get_brightness(&self) -> u8 {
        let rgb = color_space::Rgb::new(self.r as f64, self.g as f64, self.b as f64);
        let hsv = color_space::Hsv::from(rgb);
        (hsv.v * 100f64).floor() as u8
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        let rgb = color_space::Rgb::new(self.r as f64, self.g as f64, self.b as f64);
        let mut hsv = color_space::Hsv::from(rgb);

        hsv.v = brightness as f64 / 100f64;
        let rgb = hsv.to_rgb();
        self.r = rgb.r.floor() as u8;
        self.g = rgb.g.floor() as u8;
        self.b = rgb.b.floor() as u8;
    }

    pub fn off() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
        }
    }

    pub fn on() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
        }
    }

    pub fn is_off(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }

    pub fn from_spectrum_rgb(spectrum_rgb: i32) -> Self {
        let hex = format!("{:06X}", spectrum_rgb);
        let mut chars = hex.chars();

        let color = |chars: &mut Chars| {
            let hex = format!("{}{}", chars.nth(0).unwrap(), chars.nth(0).unwrap());
            u8::from_str_radix(&hex, 16).unwrap()
        };

        Self {
            r: color(&mut chars),
            g: color(&mut chars),
            b: color(&mut chars),
        }
    }

    pub fn into_spectrum_rgb(&self) -> i32 {
        let hex = format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b);
        i32::from_str_radix(&hex, 16).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::dal::device::Rgb;

    #[test]
    fn test_rgb_to_spectrum_rgb() {
        assert_eq!(16711935, Rgb { r: 255, g: 0, b: 255 }.into_spectrum_rgb());
    }

    #[test]
    fn test_spectrum_rgb_to_rgb() {
        assert_eq!(Rgb { r: 255, g: 0, b: 255 }, Rgb::from_spectrum_rgb(16711935))
    }
}

pub fn get_rgb(tx: &mut Transaction) -> WebResult<Option<Rgb>> {
    let row: Row = match tx.exec_first("SELECT r,g,b FROM device_color", Params::Empty)? {
        Some(x) => x,
        None => return Ok(None)
    };

    let r: u8 = row.get("r").unwrap();
    let g: u8 = row.get("g").unwrap();
    let b: u8 = row.get("b").unwrap();

    Ok(Some(Rgb {
        r,
        g,
        b
    }))
}

pub fn set_rgb(tx: &mut Transaction, rgb: Rgb) -> WebResult<()> {
    if get_rgb(tx)?.is_some() {
        tx.exec_drop("UPDATE device_color SET r = :r, g = :g, b = :b", params! {
            "r" => rgb.r,
            "g" => rgb.g,
            "b" => rgb.b
        })?;
    } else {
        tx.exec_drop("INSERT INTO device_color (r, g, b) VALUES (:r, :g, :b)", params! {
            "r" => rgb.r,
            "g" => rgb.g,
            "b" => rgb.b
        })?;
    }

    Ok(())
}

pub fn get_state(tx: &mut Transaction) -> WebResult<Option<bool>> {
    let row: Row = match tx.exec_first("SELECT off FROM device_state", Params::Empty)? {
        Some(x) => x,
        None => return Ok(None)
    };

    let off: bool = row.get("off").unwrap();
    Ok(Some(!off))
}

pub fn set_state(tx: &mut Transaction, on: bool) -> WebResult<()> {
    if get_state(tx)?.is_some() {
        tx.exec_drop("UPDATE device_state SET off = :off", params! {
            "off" => !on
        })?;
    } else {
        tx.exec_drop("INSERT INTO device_state (off) VALUES (:off)", params! {
            "off" => !on
        })?;
    }

    Ok(())
}