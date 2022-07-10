CREATE TABLE oauth2_exchange_tokens (
    token VARCHAR(20) NOT NULL PRIMARY KEY,
    expiry BIGINT NOT NULL
);

CREATE TABLE oauth2_bearer_tokens (
    token VARCHAR(20) NOT NULL PRIMARY KEY,
    expiry BIGINT NOT NULL
);

CREATE TABLE oauth2_refresh_tokens (
    token VARCHAR(20) NOT NULL PRIMARY KEY
);