use actix_web::web;
use mysql::TxOpts;
use serde::{Serialize, Deserialize};
use tap::Tap;
use tracing::{instrument, warn};
use crate::dal::oauth2::{generate_token, get_exchange_token, get_refresh_token, insert_bearer_token, insert_refresh_token, remove_exchange_token};
use crate::data::WebData;
use crate::error::{Error, WebResult};

#[derive(Debug, Deserialize)]
pub struct Request {
    client_id: String,
    client_secret: String,
    grant_type: String,
    code: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    token_type: &'static str,
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

#[instrument]
pub async fn exchange(data: WebData, payload: web::Form<Request>) -> WebResult<web::Json<Response>> {
    let cfg = &data.config;
    if payload.client_id.ne(&cfg.oauth2_client_id) {
        warn!("Client ID is not equal");
        return Err(Error::InvalidGrant);
    }

    if payload.client_secret.ne(&cfg.oauth2_client_secret) {
        warn!("Client secret is not equal (got: {}, wanted: {})", payload.client_secret, cfg.oauth2_client_secret);
        return Err(Error::InvalidGrant);
    }

    let mut tx = data.pool.start_transaction(TxOpts::default())?;

    let response = match payload.grant_type.as_str() {
        "authorization_code" => {
            let code = payload.code.as_ref().ok_or(Error::InvalidGrant).tap(|x| if x.is_err() {
                warn!("Auth code was not given");
            })?;

            let expiry = match get_exchange_token(&mut tx, &code)? {
                Some(x) => x,
                None => {
                    warn!("Auth code was not found");
                    return Err(Error::InvalidGrant)
                }
            };

            if time::OffsetDateTime::now_utc().unix_timestamp() > expiry {
                remove_exchange_token(&mut tx, &code)?;
                warn!("Auth code has expired");
                return Err(Error::InvalidGrant);
            }

            let access_token = generate_token();
            let access_token_expiry = time::Duration::days(1).whole_seconds();

            let refresh_token = generate_token();

            insert_bearer_token(&mut tx, &access_token, time::OffsetDateTime::now_utc().unix_timestamp() + access_token_expiry)?;
            insert_refresh_token(&mut tx, &refresh_token)?;

            Response {
                token_type: "Bearer",
                access_token,
                refresh_token: Some(refresh_token),
                expires_in: access_token_expiry
            }
        },
        "refresh_token" => {
            let token = payload.refresh_token.as_ref().ok_or(Error::InvalidGrant)?;
            match get_refresh_token(&mut tx, &token)? {
                Some(_) => {},
                None => {
                    warn!("Could not find refresh token");
                    return Err(Error::InvalidGrant)
                }
            }

            let access_token = generate_token();
            let access_token_expiry = time::Duration::days(1).whole_seconds();

            insert_bearer_token(&mut tx, &access_token, time::OffsetDateTime::now_utc().unix_timestamp() + access_token_expiry)?;

            Response {
                token_type: "Bearer",
                access_token,
                refresh_token: None,
                expires_in: access_token_expiry
            }
        },
        _ => return Err(Error::InvalidGrant)
    };

    tx.commit()?;
    Ok(web::Json(response))
}