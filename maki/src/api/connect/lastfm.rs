use anyhow::anyhow;
use axum::{extract::Query, response::IntoResponse, Extension, Json};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{api::middleware::jwt::AuthUser, clients::lastfm::generate_api_sig, error::AppError};

#[derive(Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LastFmStepOneResponse {
    pub token: String,
    pub url: String,
}

pub async fn get_lastfm_token() -> Result<impl IntoResponse, AppError> {
    let method = "auth.gettoken";
    let api_key = std::env::var("FM_KEY").expect("FM_KEY must be set");
    let shared_secret = std::env::var("FM_SECRET").expect("FM_SECRET must be set");
    let query_params: &[(&str, &str)] = &[
        ("api_key", &api_key),
        ("method", method),
        ("format", "json"),
    ];
    let api_sig = generate_api_sig(query_params, &shared_secret);

    let client = reqwest::Client::new();

    match client
        .get("http://ws.audioscrobbler.com/2.0/")
        .query(&query_params)
        .query(&[("api_sig", &api_sig)])
        .send()
        .await
    {
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

    let query_params: &[(&str, &str)] = &[
        ("api_key", &api_key),
        ("method", method),
        ("token", &params.token),
    ];
    let api_sig = generate_api_sig(query_params, &shared_secret);

    let other_query_params: &[(&str, &str)] = &[("api_sig", &api_sig), ("format", "json")];

    let client = reqwest::Client::new();

    match client
        .get("http://ws.audioscrobbler.com/2.0/")
        .query(&query_params)
        .query(&other_query_params)
        .send()
        .await
    {
        Ok(response) => match response.json::<SessionResponse>().await {
            Ok(session_data) => {
                let _ = sqlx::query!(
                    r#"INSERT INTO user_lastfm ("userId", lastfm_username, lastfm_session_key) VALUES ($1, $2, $3)"#,
                    // mostly guaranteed to be an integer because of the auth middleware
                    payload.sub.parse::<i32>().unwrap(),
                    session_data.session.name,
                    session_data.session.key,
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

// #[test]
// fn test_generate_api_sig() {
//     let params = &[("api_key", "foo"), ("method", "bar"), ("token", "baz")];
//     let expected = "f1d2d2f924e986ac86fdf7b36c94bcdf";
//     assert_eq!(generate_api_sig(params), expected);
// }
