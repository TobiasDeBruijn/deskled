use actix_web::{App, HttpServer, web};
use mysql::{OptsBuilder, Pool};
use tracing::{info, warn};
use crate::data::AppData;
use crate::error::WebResult;
use crate::routable::Routable;

mod authorization;
mod routes;
mod routable;
mod data;
mod dal;
mod error;

pub use data::Config;
use driver::Driver;

pub async fn start(config: Config, mut driver: Driver) -> WebResult<()> {
    let pool = setup_mysql(&config)?;
    let (tx, mut rx) = tokio::sync::mpsc::channel(250);
    let appdata = AppData {
        pool,
        config: config.clone(),
        driver: tx,
    };

    tokio::spawn(
        HttpServer::new(move || App::new()
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(actix_cors::Cors::permissive())
            .app_data(web::Data::new(appdata.clone()))
            .configure(routes::Router::configure)
        ).bind("[::]:8080").expect("Binding to port").run()
    );

    loop {
        let rgb = rx.recv().await.expect("Channel has been closed");
        info!("Setting RGB: {rgb:?}");
        match driver.set_rgb(driver::Rgb::new(rgb.r, rgb.g, rgb.b)) {
            Ok(_) => {},
            Err(e) => {
                warn!("Failed to set RGB: {e}");
            }
        }
    }
}

mod migrations {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

fn setup_mysql(config: &Config) -> WebResult<Pool> {
    let opts = OptsBuilder::new()

        .ip_or_hostname(Some(&config.mysql_host))
        .db_name(Some(&config.mysql_database))
        .user(Some(&config.mysql_username))
        .pass(Some(&config.mysql_password));
    let pool = Pool::new_manual(1, 10, opts)?;
    let mut conn = pool.get_conn()?;

    migrations::migrations::runner()
        .set_migration_table_name("__deskled_migrations")
        .run(&mut conn)?;

    Ok(pool)
}