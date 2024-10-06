use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderValue},
};
use tracing::debug;

use crate::helpers::jwt::{decode_jwe, SessionToken};

pub struct AuthUser {
    pub payload: SessionToken,
}

/// Get a cookie value
/// name - An array of cookie names to search for. Returns the first value found.
fn get_cookie<'a>(names: &[&'a str], cookies: &'a HeaderValue) -> Option<(&'a str, &'a str)> {
    cookies.to_str().ok().and_then(|cookies_str| {
        cookies_str.split(';').find_map(|cookie| {
            let cookie = cookie.trim();
            names.iter().find_map(|&name| {
                if cookie.starts_with(&format!("{}=", name)) {
                    let data = cookie.split('=').nth(1)?;
                    Some((name, data))
                } else {
                    None
                }
            })
        })
    })
}

fn get_first_two<T>(mut vec: Vec<T>) -> Option<(T, T)> {
    if vec.len() >= 2 {
        let second = vec.pop().unwrap();
        let first = vec.pop().unwrap();
        Some((first, second))
    } else {
        None
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the Authorization header
        // E.g. Authorization: Bearer <JWE>
        let auth_header = parts.headers.get("Authorization");

        debug!("Auth header: {:?}", auth_header);

        // return a token pair - token name, token payload
        let token: (&str, &str) = if let Some(auth_header) = auth_header {
            if let Some(token) = auth_header
                .to_str()
                .ok()
                .and_then(|s| s.strip_prefix("Bearer "))
            {
                get_first_two(token.split(':').collect()).ok_or_else(|| {
                    (
                        axum::http::StatusCode::UNAUTHORIZED,
                        "Token could not be parsed from auth header. Invalid format?".to_string(),
                    )
                })?
            } else {
                return Err((
                    axum::http::StatusCode::UNAUTHORIZED,
                    "Token could not be parsed. Invalid header?".to_string(),
                ));
            }
        // Get the authjs session token from the cookie if there is one
        } else if let Some(cookie) = parts.headers.get("Cookie") {
            if let Some(token) = get_cookie(
                &["__Secure-authjs.session-token", "authjs.session-token"],
                cookie,
            ) {
                token
            } else {
                return Err((
                    axum::http::StatusCode::UNAUTHORIZED,
                    "Token could not be parsed. Invalid cookie name?".to_string(),
                ));
            }
        // Get the token from the query string (make this optional via config)
        } else if let Some(path) = parts.uri.query() {
            // get ?auth_token=xxx in a query string
            // WARNING: This is NOT secure. Do NOT use unless you know what you are doing.
            if let Some(token) = path.split("auth_token=").nth(1) {
                // we have xxx&othertoken
                if let Some(token) = token.split('&').next() {
                    get_first_two(token.split(':').collect()).ok_or_else(|| {
                        (
                            axum::http::StatusCode::UNAUTHORIZED,
                            "Token could not be parsed from query string. Invalid format?"
                                .to_string(),
                        )
                    })?
                } else {
                    return Err((
                        axum::http::StatusCode::UNAUTHORIZED,
                        "Token could not be parsed from query string. Invalid token format?"
                            .to_string(),
                    ));
                }
            } else {
                return Err((
                    axum::http::StatusCode::UNAUTHORIZED,
                    "Token could not be parsed from query string. Invalid query param?".to_string(),
                ));
            }
        } else {
            return Err((
                axum::http::StatusCode::UNAUTHORIZED,
                "No token found".to_string(),
            ));
        };

        debug!("Found JWE token-header pair: {:?}", token);

        // Decode the JWE
        let jwt = decode_jwe(token.1, token.0)
            .map_err(|e| (axum::http::StatusCode::UNAUTHORIZED, e.to_string()))?;

        Ok(AuthUser { payload: jwt })
    }
}
