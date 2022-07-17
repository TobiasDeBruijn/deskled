use mysql::{params, Row, Transaction};
use mysql::prelude::Queryable;
use rand::Rng;
use crate::error::WebResult;
use tracing::instrument;

pub fn generate_token() -> String {
    rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(20).map(char::from).collect::<String>()
}

#[instrument]
pub fn insert_exchange_token(tx: &mut Transaction, token: &str, expiry: i64) -> WebResult<()> {
    tx.exec_drop("INSERT INTO oauth2_exchange_tokens (token, expiry) VALUES (:token, :expiry)", params! {
        "token" => token,
        "expiry" => expiry
    })?;
    Ok(())
}

#[instrument]
pub fn get_exchange_token(tx: &mut Transaction, token: &str) -> WebResult<Option<i64>> {
    let row: Row = match tx.exec_first("SELECT expiry FROM oauth2_exchange_tokens WHERE token = :token", params! {
        "token" => token
    })? {
        Some(x) => x,
        None => return Ok(None)
    };

    Ok(Some(row.get("expiry").unwrap()))
}

#[instrument]
pub fn remove_exchange_token(tx: &mut Transaction, token: &str) -> WebResult<()> {
    tx.exec_drop("DELETE FROM oauth2_exchange_tokens WHERE token = :token", params! {
        "token" => token
    })?;
    Ok(())
}

#[instrument]
pub fn insert_bearer_token(tx: &mut Transaction, token: &str, expiry: i64) -> WebResult<()> {
    tx.exec_drop("INSERT INTO oauth2_bearer_tokens (token, expiry) VALUES (:token, :expiry)", params! {
        "token" => token,
        "expiry" => expiry
    })?;
    Ok(())
}

#[instrument]
pub fn get_bearer_token(tx: &mut Transaction, token: &str) -> WebResult<Option<i64>> {
    let row: Row = match tx.exec_first("SELECT expiry FROM oauth2_bearer_tokens WHERE token = :token", params! {
        "token" => token
    })? {
        Some(x) => x,
        None => return Ok(None)
    };

    Ok(Some(row.get("expiry").unwrap()))
}

#[instrument]
pub fn remove_bearer_token(tx: &mut Transaction, token: &str) -> WebResult<()> {
    tx.exec_drop("DELETE FROM oauth2_bearer_tokens WHERE token = :token", params! {
        "token" => token
    })?;
    Ok(())
}

#[instrument]
pub fn insert_refresh_token(tx: &mut Transaction, token: &str) -> WebResult<()> {
    tx.exec_drop("INSERT INTO oauth2_refresh_tokens (token) VALUES (:token)", params! {
        "token" => token,
    })?;
    Ok(())
}

#[instrument]
pub fn get_refresh_token(tx: &mut Transaction, token: &str) -> WebResult<Option<()>> {
    let _: Row = match tx.exec_first("SELECT 1 FROM oauth2_refresh_tokens WHERE token = :token", params! {
        "token" => token
    })? {
        Some(x) => x,
        None => return Ok(None)
    };

    Ok(Some(()))
}

#[instrument]
pub fn remove_refresh_token(tx: &mut Transaction, token: &str) -> WebResult<()> {
    tx.exec_drop("DELETE FROM oauth2_refresh_tokens WHERE token = :token", params! {
        "token" => token
    })?;
    Ok(())
}
