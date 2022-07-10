use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod login_api;
mod login_static;
mod exchange;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/oauth2")
            .route("/login", web::get().to(login_static::login_static))
            .route("/login", web::post().to(login_api::login))
            .route("/exchange", web::post().to(exchange::exchange))
        );
    }
}