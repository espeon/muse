use anyhow::anyhow;
use axum::{extract::Query, response::IntoResponse, Extension, Json};

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};

use crate::{api::middleware::jwt::AuthUser, error::AppError};

#[derive(Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LastFmStepOneResponse {
    pub token: String,
    pub url: String,
}

fn generate_api_sig(api_key: &str, api_endpoint: &str, shared_secret: &str) -> String {
    let mut hasher = Md5::new();
    let concat = format!(
        "api_key{}method{}token{}",
        api_key, api_endpoint, shared_secret
    );
    hasher.update(concat.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn get_lastfm_token() -> Result<impl IntoResponse, AppError> {
    let method = "auth.gettoken";
    let api_key = std::env::var("FM_KEY").expect("FM_KEY must be set");
    let shared_secret = std::env::var("FM_SECRET").expect("FM_SECRET must be set");
    let api_sig = generate_api_sig(&api_key, method, &shared_secret);

    let url = format!(
        "http://ws.audioscrobbler.com/2.0/?method={}&api_key={}&api_sig={}&format=json",
        method, api_key, api_sig
    );

    let client = reqwest::Client::new();

    match client.get(&url).send().await {
        Ok(response) => match response.json::<TokenResponse>().await {
            Ok(token_data) => Ok(Json(LastFmStepOneResponse {
                token: token_data.token.to_owned(),
                url: format!(
                    "http://www.last.fm/api/auth/?api_key={}&token={}",
                    api_key, token_data.token
                ),
            })
            .into_response()),
            Err(e) => {
                Err(anyhow!("Failed to parse response: ".to_string() + &e.to_string()).into())
            }
        },
        Err(e) => Err(anyhow!("Failed to get token: ".to_string() + &e.to_string()).into()),
    }
}

// token query param
#[derive(Deserialize)]
pub struct SessionResponse {
    pub session: SessionKey,
}

#[derive(Deserialize)]
pub struct SessionKey {
    pub key: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct LastFmStepTwoQueryParams {
    pub token: String,
}

#[derive(Serialize)]
pub struct LastFmStepTwoResponse {
    pub session_key: String,
    pub username: String,
}

pub async fn post_lastfm_session(
    Query(params): Query<LastFmStepTwoQueryParams>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let method = "auth.getSession";
    let api_key = std::env::var("FM_KEY").expect("FM_KEY must be set");
    let shared_secret = std::env::var("FM_SECRET").expect("FM_SECRET must be set");

    let api_sig = generate_api_sig(&api_key, method, &shared_secret);

    let url = format!(
        "http://ws.audioscrobbler.com/2.0/?method={}&api_key={}&token={}&api_sig={}&format=json",
        method, api_key, params.token, api_sig
    );

    let client = reqwest::Client::new();

    match client.get(&url).send().await {
        Ok(response) => match response.json::<SessionResponse>().await {
            Ok(session_data) => {
                let q = sqlx::query!(
                    r#"INSERT INTO user_lastfm ("userId", lastfm_username, lastfm_session_key) VALUES ($1, $2, $3)"#,
                    payload.sub.parse::<i32>().unwrap(),
                    session_data.session.key,
                    session_data.session.name
                ).execute(&pool).await;

                Ok(Json(LastFmStepTwoResponse {
                    session_key: session_data.session.key,
                    username: session_data.session.name,
                })
                .into_response())
            }
            Err(e) => Err(anyhow!(
                "Failed to parse session response: ".to_string() + &e.to_string()
            )
            .into()),
        },
        Err(e) => Err(anyhow!("Failed to get session: ".to_string() + &e.to_string()).into()),
    }
}
