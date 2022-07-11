use actix_web::web;
use mysql::TxOpts;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use crate::dal::oauth2::{generate_token, insert_exchange_token};
use crate::data::WebData;
use crate::error::{Error, WebResult};

#[derive(Debug, Deserialize)]
pub struct Query {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    state: String,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    redirect_uri: String,
}

#[instrument]
pub async fn login(data: WebData, query: web::Query<Query>, payload: web::Json<Request>) -> WebResult<web::Json<Response>> {
    let cfg = &data.config;
    if query.client_id.ne(&cfg.oauth2_client_id) {
        return Err(Error::Unauthorized);
    }

    if query.response_type.ne("code") {
        return Err(Error::Unauthorized);
    }

    if payload.username.ne(&cfg.login_username) {
        return Err(Error::Unauthorized);
    }

    if payload.password.ne(&cfg.login_password) {
        return Err(Error::Unauthorized);
    }

    let token = generate_token();
    let expiry = (time::OffsetDateTime::now_utc() + time::Duration::minutes(10)).unix_timestamp();

    let mut tx = data.pool.start_transaction(TxOpts::default())?;
    insert_exchange_token(&mut tx, &token, expiry)?;
    tx.commit()?;

    let redirect_uri = format!("{}?code={token}&state={}", query.redirect_uri, query.state);
    Ok(web::Json(Response {
        redirect_uri
    }))
}
