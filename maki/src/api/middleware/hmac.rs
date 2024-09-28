use std::time::{SystemTime, UNIX_EPOCH};

use crate::helpers::{verify_hmac, HmacMessage};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use base64::Engine;
use tracing::trace;

#[allow(dead_code)]
pub struct HmacAuth {
    pub message: HmacMessage,
}

#[async_trait]
impl<S> FromRequestParts<S> for HmacAuth
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret_key = env!("AUTH_SECRET");
        // All parts are in a query string 'tk'
        // ?tk=st=1370627194~exp=1370627409~acl=*~hmac=6a10b3f602ffde88c02cd1b89665bfdfdc0fc84c3cf7049752931ad7
        let params = parts.uri.query().unwrap();
        let token = if let Some(token) = params.split("tk=").nth(1) {
            if let Some(token) = token.split('&').next() {
                token
            } else {
                trace!("invalid token (token:{})", token);
                return Err(axum::http::StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        };
        let mut split = token.split("~hmac=");
        // get the message from the token (everything before the hmac)
        let message = if let Some(message) = split.nth(0) {
            message
        } else {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        };
        // get the hmac from the token (nth consumes the message)
        let hmac = if let Some(hmac) = split.nth(0) {
            hmac.replace("hmac=", "")
        } else {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        };
        // make sure the hmac is valid
        let msg = HmacMessage::decode(message).unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // if timestamp is in between st and exp
        // Give 10s leeway for clock skew
        if !(msg.st - 10 < timestamp && msg.exp > timestamp) {
            trace!(
                "invalid timestamp for hmac (token:{:?}, timestamp:{})",
                msg,
                timestamp
            );
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }

        // check the acl (make sure it starts with the acl at least)
        // TODO: make this easier to configure and check
        // get the url base and path without the query string
        if msg.acl != "*" {
            let authority = parts.uri.authority().unwrap();
            let path = parts.uri.path().to_string();
            if !(format!("{}{}", authority, path).starts_with(&msg.acl)) {
                // acl is invalid!
                trace!("invalid acl for hmac (token:{})", token);
                return Err(axum::http::StatusCode::UNAUTHORIZED);
            }
        }

        // decode the hmac to bytes
        let hmac_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&hmac)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

        trace!("verifying hmac:  message:{}, signature:{}", message, hmac);

        if verify_hmac(secret_key.as_bytes(), message.as_bytes(), &hmac_bytes) {
            // Authentication successful
            // In a real app, you might decode a user ID from the message
            Ok(HmacAuth { message: msg })
        } else {
            trace!("invalid hmac, could not verify! (token:{})", token);
            Err(axum::http::StatusCode::UNAUTHORIZED)
        }
    }
}
