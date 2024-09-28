use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderValue},
};
use ring::hkdf;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub struct AuthUser {
    pub payload: SessionToken,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
pub struct SessionToken {
    pub name: Option<String>,
    pub email: String,
    pub picture: Option<String>,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
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

pub struct HkdfOutput<T>(pub T);

impl hkdf::KeyType for HkdfOutput<usize> {
    fn len(&self) -> usize {
        self.0
    }
}

fn derive_key(ikm: &[u8], salt: &[u8], info: &[u8], okm_len: usize) -> anyhow::Result<Vec<u8>> {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, salt);
    let prk = salt.extract(ikm);
    let binding = [info];
    let okm = prk
        .expand(&binding, HkdfOutput(okm_len))
        .map_err(|e| anyhow!("Failed to expand key: {:?}", e))?;

    let mut result = vec![0u8; okm_len];
    okm.fill(&mut result)
        .map_err(|e| anyhow!("Failed to fill key: {:?}", e))?;

    Ok(result)
}

fn decode_jwe(jwe: &str, cookie_name: &str) -> anyhow::Result<String> {
    let auth_secret = std::env::var("AUTH_SECRET")
        .map_err(|_| anyhow!("AUTH_SECRET is not defined in the environment"))?;

    let key = derive_key(
        auth_secret.as_bytes(),
        cookie_name.as_bytes(),
        format!("Auth.js Generated Encryption Key ({})", cookie_name).as_bytes(),
        64,
    )?;

    // Create decrypter
    let decrypter = josekit::jwe::Dir
        .decrypter_from_bytes(key)
        .map_err(|e| anyhow!("Failed to create decrypter: {:?}", e))?;

    // Parse the JWE token
    let payload = josekit::jwe::deserialize_compact(jwe, &decrypter).map_err(|e| {
        anyhow!(
            "Failed to deserialize JWE token: {:?}\n\n\n{}\n{}",
            e,
            jwe,
            cookie_name
        )
    })?;

    let data_bytes: Vec<u8> = payload.0;
    let data = String::from_utf8(data_bytes).map_err(|_| anyhow!("Invalid UTF-8 data"))?;
    debug!("JWE decrypted: {:?}", data);
    Ok(data)
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

        // deserialise decoded jwt
        let jwt: SessionToken = serde_json::from_str(&jwt).map_err(|e| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                format!("Failed to deserialise JWT: {:?}", e),
            )
        })?;

        Ok(AuthUser { payload: jwt })
    }
}
