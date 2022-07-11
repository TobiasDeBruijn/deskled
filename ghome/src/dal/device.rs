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