use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use base64::Engine;
use sqlx::PgPool;
use time::Duration;

use crate::helpers::{
    jwt::{encode_jwe, PartialSessionToken},
    random_id,
};

use super::providers::{ExternalAccountInfo, ExternalUserInfo, SharedAuthProvider};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StartAuthResponse {
    pub url: String,
}

#[derive(serde::Deserialize, Default)]
pub struct StartAuthQuery {
    /// Set to "mobile" to trigger a deep-link redirect after auth completes.
    pub platform: Option<String>,
}

/// Extract the `state` query parameter from an OAuth2 authorization URL.
fn extract_state(url: &str) -> Option<String> {
    url.split('?').nth(1)?.split('&').find_map(|pair| {
        let mut kv = pair.splitn(2, '=');
        if kv.next()? == "state" {
            kv.next().map(|v| v.to_owned())
        } else {
            None
        }
    })
}

pub async fn start_auth(
    Extension(authcfg): Extension<SharedAuthProvider>,
    Query(params): Query<StartAuthQuery>,
) -> Result<Json<StartAuthResponse>, (StatusCode, String)> {
    tracing::info!("Starting auth flow, platform: {:?}", params.platform);
    let mut authcfg = authcfg.lock().await;
    match authcfg.generate_challenge() {
        Ok(url) => {
            tracing::info!("Generated auth URL: {}", url);
            if params.platform.as_deref() == Some("mobile") {
                if let Some(state) = extract_state(&url) {
                    tracing::info!("Extracted state for mobile: {}", state);
                    authcfg.mark_mobile_session(&state);
                } else {
                    tracing::error!("Failed to extract state from auth URL");
                }
            }
            Ok(Json(StartAuthResponse { url }))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(serde::Deserialize)]
pub struct FinishAuthQuery {
    code: String,
    state: String,
}

pub async fn finish_auth(
    Extension(authcfg): Extension<SharedAuthProvider>,
    Query(auth): Query<FinishAuthQuery>,
    Extension(pool): Extension<PgPool>,
) -> Result<Response, (StatusCode, String)> {
    let is_mobile = authcfg.lock().await.take_mobile_session(&auth.state);
    tracing::info!("finish_auth: state={}, is_mobile={}", auth.state, is_mobile);

    let mut authcfg = authcfg.lock().await;
    // verify challenge
    tracing::info!("Verifying challenge...");
    let token_res = match authcfg.verify_challenge(&auth.state, &auth.code).await {
        Ok(i) => {
            tracing::info!("Challenge verified successfully");
            i
        }
        Err(e) => {
            tracing::error!("Challenge verification failed: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    };

    // we are good, now fetch and save user info
    tracing::info!("Fetching user info...");
    let oidc_resp = match authcfg.get_user_info(&token_res).await {
        Ok(i) => {
            tracing::info!("User info fetched successfully");
            i
        }
        Err(e) => {
            tracing::error!("User info fetch failed: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    };

    let account_info = match authcfg.make_account_info(&token_res, &oidc_resp) {
        Ok(i) => i,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    let ext_user_info = match authcfg.make_user_info(oidc_resp) {
        Ok(i) => i,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    let user = find_or_create_user(account_info, ext_user_info, pool.clone()).await?;

    let sess = create_session(user, pool).await?;

    if is_mobile {
        // Deep-link back to the iOS app with the token pair
        let deep_link = format!(
            "muse://auth/callback?session_token={}&session_expiry={}&refresh_token={}&refresh_expiry={}",
            sess.session_token.token,
            sess.session_token.expiry,
            sess.refresh_token.token,
            sess.refresh_token.expiry,
        );
        tracing::info!("Redirecting to mobile deep link: {}", deep_link);
        Ok(Redirect::temporary(&deep_link).into_response())
    } else {
        tracing::info!("Returning JSON session for web");
        Ok(Json(sess).into_response())
    }
}

async fn find_or_create_user(
    account_info: ExternalAccountInfo,
    ext_user_info: ExternalUserInfo,
    pool: PgPool,
) -> Result<UserInfoDbResponse, (StatusCode, String)> {
    // check if the user already exists from the account info
    let user_id = match sqlx::query!(
        r#"
        SELECT id FROM accounts WHERE provider = $1 AND "providerAccountId" = $2
        "#,
        account_info.provider,
        ext_user_info.id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(i) => i,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())), // TODO: log error
    };

    if user_id.is_none() {
        // create user
        let user = sqlx::query_as!(
            UserInfoDbResponse,
            r#"
            INSERT INTO users (name, email, image)
            VALUES ($1, $2, $3)
            RETURNING id, name, email, image
            "#,
            Some(ext_user_info.username),
            Some(ext_user_info.email),
            ext_user_info.image
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to create user: {}", e),
            )
        })?;
        // create account, link user
        sqlx::query_as!(AccountDbResponse,
            r#"
            INSERT INTO accounts ("userId", type, provider, "providerAccountId", refresh_token, access_token, expires_at, scope, session_state, token_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT ("providerAccountId") DO UPDATE SET provider = $3, refresh_token = $5, access_token = $6, expires_at = $7, scope = $8, session_state = $9, token_type = $10
            "#,
                user.id as i32,
                account_info.provider_type,
                account_info.provider,
                account_info.provider_id,
                account_info.refresh_token,
                account_info.access_token,
                account_info.expires_at,
                account_info.scope,
                account_info.session_state,
                account_info.token_type,
        )
        .execute(&pool)
        .await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to create account: {}", e),
            )
        })?;

        Ok(user)
    } else {
        let user = sqlx::query_as!(
            UserInfoDbResponse,
            r#"
            SELECT id, name, email, image FROM users WHERE id = $1
            "#,
            user_id.unwrap().id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to fetch user: {}", e),
            )
        })?;
        Ok(user)
    }
}

// create a session token, returning a short-lived session JWT and long-lived refresh token pair
// Refresh tokens will be entered into the database
// Session tokens will not be returned to the client, as they can be verified in other ways.
// The short-lived-ness of the session token is to help mitigate issues relating to token leakage and misuse.
pub async fn create_session(
    user: UserInfoDbResponse,
    pool: PgPool,
) -> Result<CreateSessionResponse, (StatusCode, String)> {
    // duration for a refresh token is 4 weeks currently
    // TODO: make this configurable
    let duration = time::Duration::seconds(60 * 60 * 24 * 7 * 4);
    let refresh_token = random_id(32);
    let ref_expiry_time = time::OffsetDateTime::now_utc() + duration;
    // create and insert refresh token hash
    let refresh_token_hash = ring::digest::digest(&ring::digest::SHA256, refresh_token.as_bytes());
    let refresh_token_hash = base64::engine::general_purpose::URL_SAFE.encode(refresh_token_hash);
    sqlx::query!(
        r#"
        INSERT INTO sessions ("userId", expires, refresh_token)
        VALUES ($1, $2, $3)
        "#,
        user.id as i32,
        ref_expiry_time,
        refresh_token_hash
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create session: {}", e),
        )
    })?;

    let sess = new_jwt(user, Duration::seconds(60 * 60))?;

    Ok(CreateSessionResponse {
        session_token: TokenPair {
            token: sess,
            expiry: (time::OffsetDateTime::now_utc() + Duration::seconds(60 * 60)).unix_timestamp(),
        },
        refresh_token: TokenPair {
            token: refresh_token,
            expiry: ref_expiry_time.unix_timestamp(),
        },
    })
}

pub fn new_jwt(
    user: UserInfoDbResponse,
    expires: time::Duration,
) -> Result<String, (StatusCode, String)> {
    let sess = encode_jwe(
        PartialSessionToken {
            name: user.name,
            email: user.email.unwrap_or("no@email".to_string()),
            picture: user.image,
            sub: user.id.to_string(),
            acl: None,
        },
        "authjs.session-token",
        expires,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create session token: {}", e),
        )
    })?;
    Ok(sess)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateSessionResponse {
    pub session_token: TokenPair,
    pub refresh_token: TokenPair,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenPair {
    pub token: String,
    /// unix timestamp of the expiry time
    pub expiry: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserInfoDbResponse {
    pub id: i64,
    pub name: Option<String>,
    pub email: Option<String>,
    //email_verified: bool,
    pub image: Option<String>,
}
