use actix_web::web;
use mysql::Pool;

pub(crate) type WebData = web::Data<AppData>;

#[derive(Debug, Clone)]
pub struct AppData {
    pub config: Config,
    pub pool: Pool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub oauth2_client_id: String,
    pub oauth2_client_secret: String,
    pub oauth2_redirect_uri: String,
    pub login_username: String,
    pub login_password: String,

    pub mysql_host: String,
    pub mysql_password: String,
    pub mysql_username: String,
    pub mysql_database: String,
}
