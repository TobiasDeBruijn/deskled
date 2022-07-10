use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod oauth2;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config
            .configure(oauth2::Router::configure);
    }
}