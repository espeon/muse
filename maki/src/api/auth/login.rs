use axum::{
    extract::{Extension, Query},
    http::{header::SET_COOKIE, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use base64::Engine;
use reqwest::Url;
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
    /// `mobile` triggers a deep-link redirect after auth completes.
    /// `web` triggers a cookie + redirect to /controller/.
    /// `spa` redirects to the embedded web UI or an approved standalone origin.
    /// Any other / unset value returns the OIDC URL in JSON (the
    /// nozomi flow).
    pub platform: Option<String>,
    /// For a standalone SPA, the origin registered in WEB_ORIGINS.
    pub redirect_uri: Option<String>,
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

#[derive(Clone)]
pub struct SpaOrigins(Vec<String>);

impl SpaOrigins {
    pub fn parse(config: &str) -> anyhow::Result<Self> {
        config
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| {
                Self::origin(value).ok_or_else(|| anyhow::anyhow!("Invalid origin in WEB_ORIGINS"))
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .map(Self)
    }

    fn origin(value: &str) -> Option<String> {
        let url = Url::parse(value).ok()?;
        let loopback = url.domain() == Some("localhost")
            || url
                .host_str()
                .and_then(|host| {
                    host.trim_matches(&['[', ']'][..])
                        .parse::<std::net::IpAddr>()
                        .ok()
                })
                .is_some_and(|ip| ip.is_loopback());
        let allowed_scheme = url.scheme() == "https" || (url.scheme() == "http" && loopback);
        if !allowed_scheme
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return None;
        }
        Some(url.origin().ascii_serialization())
    }

    fn callback(&self, requested: Option<&str>) -> Result<String, (StatusCode, String)> {
        let Some(requested) = requested else {
            return Ok("/auth/callback".into());
        };
        Self::origin(requested)
            .filter(|origin| self.0.contains(origin))
            .map(|origin| format!("{}/auth/callback", origin))
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "SPA origin is not allowed".into()))
    }
}

pub async fn start_auth(
    Extension(authcfg): Extension<SharedAuthProvider>,
    Extension(spa_origins): Extension<SpaOrigins>,
    Query(params): Query<StartAuthQuery>,
) -> Result<Response, (StatusCode, String)> {
    let platform = params.platform.as_deref();
    tracing::info!("Starting auth flow, platform: {:?}", platform);

    let spa_target = if platform == Some("spa") {
        Some(spa_origins.callback(params.redirect_uri.as_deref())?)
    } else {
        None
    };

    let url = {
        let mut authcfg = authcfg.lock().await;
        match authcfg.generate_challenge() {
            Ok(url) => {
                tracing::info!("Generated auth URL: {}", url);
                if let Some(state) = extract_state(&url) {
                    match platform {
                        Some("mobile") => {
                            tracing::info!("Extracted state for mobile: {}", state);
                            authcfg.mark_mobile_session(&state);
                        }
                        Some("web") => {
                            tracing::info!("Extracted state for web: {}", state);
                            authcfg.mark_web_session(&state);
                        }
                        Some("spa") => {
                            tracing::info!("Extracted state for spa: {}", state);
                            if let Some(target) = &spa_target {
                                authcfg.mark_spa_session(&state, target);
                            }
                        }
                        _ => {}
                    }
                } else {
                    tracing::error!("Failed to extract state from auth URL");
                }
                url
            }
            Err(e) => {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
            }
        }
    };

    // For the web controller and browser SPA, return a 302 redirect to the OIDC
    // URL so the browser navigates directly. For mobile and the JSON callers
    // (nozomi), keep returning the URL in JSON.
    if platform == Some("web") || platform == Some("spa") {
        Ok(Redirect::temporary(&url).into_response())
    } else {
        Ok(Json(StartAuthResponse { url }).into_response())
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
    let (is_mobile, is_web, spa_target) = {
        let mut cfg = authcfg.lock().await;
        let m = cfg.take_mobile_session(&auth.state);
        let w = cfg.take_web_session(&auth.state);
        let s = cfg.take_spa_session(&auth.state);
        (m, w, s)
    };
    tracing::info!(
        "finish_auth: state={}, is_mobile={}, is_web={}, is_spa={}",
        auth.state,
        is_mobile,
        is_web,
        spa_target.is_some()
    );

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
    } else if is_web {
        // Set the session cookie and redirect to the controller page.
        // The cookie uses the standard authjs.session-token name so
        // the maki AuthUser extractor picks it up on the WebSocket
        // upgrade automatically. Plain cookie (no Secure flag) per
        // v1 decision; HTTPS deployments can swap in __Secure- later.
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let max_age = (sess.session_token.expiry - now).max(0);
        let cookie = format!(
            "authjs.session-token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            sess.session_token.token, max_age
        );
        tracing::info!("Setting authjs.session-token cookie, redirecting to /controller/");
        let response = Redirect::temporary("/controller/").into_response();
        let mut response = response;
        response.headers_mut().insert(
            SET_COOKIE,
            cookie
                .parse()
                .map_err(|e: axum::http::header::InvalidHeaderValue| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("bad cookie: {}", e),
                    )
                })?,
        );
        Ok(response)
    } else if let Some(target) = spa_target.as_deref() {
        // Redirect to the validated SPA target with the token pair in the URL
        // fragment (a fragment never reaches server logs / proxies). The SPA's
        // /auth/callback route reads location.hash and stores the tokens.
        let fragment = format!(
            "session_token={}&session_expiry={}&refresh_token={}&refresh_expiry={}",
            sess.session_token.token,
            sess.session_token.expiry,
            sess.refresh_token.token,
            sess.refresh_token.expiry,
        );
        let dst = format!("{}#{}", target.trim_end_matches('/'), fragment);
        tracing::info!("Redirecting to SPA callback target");
        Ok(Redirect::temporary(&dst).into_response())
    } else {
        tracing::info!("Returning JSON session for nozomi");
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
        // Bootstrap: the first user ever to sign in becomes admin. The advisory lock makes
        // concurrent first sign-ins deterministic instead of creating two admins.
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        sqlx::query!("SELECT pg_advisory_xact_lock($1)", 7_372_838_454_892_885_100i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let user_count: i64 = sqlx::query_scalar!("SELECT count(*) FROM users")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .unwrap_or(0);

        let user = sqlx::query_as!(
            UserInfoDbResponse,
            r#"
            INSERT INTO users (name, email, image, is_admin)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, email, image
            "#,
            Some(ext_user_info.username),
            Some(ext_user_info.email),
            ext_user_info.image,
            user_count == 0
        )
        .fetch_one(&mut *tx)
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
        .execute(&mut *tx)
        .await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to create account: {}", e),
            )
        })?;
        tx.commit()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::auth::providers::{
        create_shared_auth_provider, generic::GenericOidcPkceProvider,
    };
    use axum::{
        body::Body,
        http::{header::LOCATION, Request},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn spa_login_registers_only_embedded_and_configured_callbacks() {
        let provider = GenericOidcPkceProvider::new(
            reqwest::Client::new(),
            "muse-test".into(),
            "test-only".into(),
            "https://identity.example.test/authorize".into(),
            "https://identity.example.test".into(),
            "https://identity.example.test/token".into(),
            "https://maki.example.test/auth/login/callback".into(),
            None,
        )
        .await
        .unwrap();
        let auth = create_shared_auth_provider(provider);
        let app = crate::routes()
            .layer(Extension(auth.clone()))
            .layer(Extension(
                SpaOrigins::parse("http://localhost:5173").unwrap(),
            ));

        for (query, callback) in [
            ("platform=spa", "/auth/callback"),
            (
                "platform=spa&redirect_uri=http%3A%2F%2Flocalhost%3A5173",
                "http://localhost:5173/auth/callback",
            ),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/auth/login?{}", query))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
            let location = response.headers()[LOCATION].to_str().unwrap();
            let state = extract_state(location).unwrap();
            assert_eq!(
                auth.lock().await.take_spa_session(&state).as_deref(),
                Some(callback)
            );
        }
    }

    #[test]
    fn spa_callback_approval_requires_an_exact_configured_origin() {
        let origins =
            SpaOrigins::parse("https://music.example.test, http://localhost:5173").unwrap();
        assert_eq!(
            origins
                .callback(Some("https://music.example.test:443/"))
                .unwrap(),
            "https://music.example.test/auth/callback"
        );
        assert!(origins
            .callback(Some("https://music.example.test:444"))
            .is_err());
        assert!(origins
            .callback(Some("https://music.example.test/settings"))
            .is_err());
        assert!(origins
            .callback(Some("https://other.example.test"))
            .is_err());
        assert!(origins.callback(Some("")).is_err());
        assert!(SpaOrigins::parse("http://music.example.test").is_err());
    }
}
