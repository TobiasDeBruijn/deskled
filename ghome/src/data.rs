use actix_web::web;
use mysql::Pool;
use crate::dal::device::Rgb;

pub(crate) type WebData = web::Data<AppData>;

#[derive(Debug, Clone)]
pub struct AppData {
    pub config: Config,
    pub pool: Pool,
    pub driver: tokio::sync::mpsc::Sender<Rgb>
}

#[derive(Debug, Clone)]
pub struct Config {
    pub oauth2_client_id: String,
    pub oauth2_client_secret: String,
    pub login_username: String,
    pub login_password: String,
    pub led_length: u16,
    pub mysql_host: String,
    pub mysql_password: String,
    pub mysql_username: String,
    pub mysql_database: String,
}
