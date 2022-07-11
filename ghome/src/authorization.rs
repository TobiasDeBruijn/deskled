use std::future::Future;
use std::pin::Pin;
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use mysql::TxOpts;
use tap::TapFallible;
use tracing::warn;
use crate::dal::oauth2::{get_bearer_token, remove_bearer_token};
use crate::data::WebData;
use crate::error::Error;

pub struct Auth;

impl FromRequest for Auth {
    type Error = crate::error::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let data: &WebData = req.app_data().unwrap();
            let authorization = req.headers()
                .get("authorization")
                .ok_or(Error::Unauthorized)
                .tap_err(|_| warn!("Missing authorization header"))?
                .to_str()
                .map_err(|_| Error::Unauthorized)
                .tap_err(|_| warn!("Invalid str in authorization header"))?;
            if !authorization.starts_with("Bearer ") {
                warn!("Authorization does not start with Bearer");
                return Err(Error::Unauthorized);
            }

            let parts = authorization.split("Bearer ").collect::<Vec<_>>();
            let token = parts.last().ok_or(Error::Unauthorized).tap_err(|_| warn!("Missing token in header"))?;

            let mut tx = data.pool.start_transaction(TxOpts::default())?;
            let expiry = get_bearer_token(&mut tx, token)?.ok_or(Error::Unauthorized).tap_err(|_| warn!("Unknown bearer token"))?;
            if time::OffsetDateTime::now_utc().unix_timestamp() > expiry {
                remove_bearer_token(&mut tx, token)?;
                warn!("Token has expired");
                return Err(Error::Unauthorized);
            }

            tx.commit()?;
            Ok(Self)
        })
    }
}
