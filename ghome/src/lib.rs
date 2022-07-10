use actix_web::{App, HttpServer, web};
use mysql::{OptsBuilder, Pool};
use crate::data::AppData;
use crate::error::WebResult;
use crate::routable::Routable;

mod routes;
mod routable;
mod data;
mod dal;
mod error;

pub use data::Config;

pub async fn start(config: Config) -> WebResult<()> {
    let pool = setup_mysql(&config)?;
    let appdata = AppData {
        pool,
        config
    };

    HttpServer::new(move || App::new()
        .wrap(tracing_actix_web::TracingLogger::default())
        .wrap(actix_cors::Cors::permissive())
        .app_data(web::Data::new(appdata.clone()))
        .configure(routes::Router::configure)
    ).bind("[::]:8080")?.run().await?;

    Ok(())
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
    let pool = Pool::new(opts)?;
    let mut conn = pool.get_conn()?;

    migrations::migrations::runner()
        .set_migration_table_name("__deskled_migrations")
        .run(&mut conn)?;

    Ok(pool)
}