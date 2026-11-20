use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    response::Response,
    Extension, Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use crate::helpers::jwt::{encode_jwe, PartialSessionToken};

async fn test_session(pool: &PgPool, admin: bool) -> String {
    let user_id: i32 = sqlx::query_scalar("INSERT INTO users (is_admin) VALUES ($1) RETURNING id")
        .bind(admin)
        .fetch_one(pool)
        .await
        .unwrap();
    let cookie = "authjs.session-token";
    let token = encode_jwe(
        PartialSessionToken {
            sub: user_id.to_string(),
            name: None,
            picture: None,
            email: "user@example.test".into(),
            acl: None,
        },
        cookie,
        time::Duration::minutes(5),
    )
    .expect("set AUTH_SECRET when running database-backed API tests");
    format!("Bearer {}:{}", cookie, token)
}

async fn request(app: &Router, auth: &str, method: &str, path: &str, body: Body) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("authorization", auth)
                .header("content-type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn read_json(response: Response) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[sqlx::test]
#[ignore = "requires a disposable PostgreSQL database and AUTH_SECRET"]
async fn name_display_is_saved_for_the_signed_in_user(pool: PgPool) {
    let auth = test_session(&pool, false).await;
    let other = test_session(&pool, false).await;
    let app = crate::routes().layer(Extension(pool));
    let response = request(
        &app,
        &auth,
        "POST",
        "/api/v1/me/display",
        Body::from(r#"{"name_display":"romanised"}"#),
    )
    .await;
    assert_eq!(read_json(response).await["name_display"], "romanised");

    let invalid = request(
        &app,
        &auth,
        "POST",
        "/api/v1/me/display",
        Body::from(r#"{"name_display":"unsupported"}"#),
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

    for (auth, expected) in [(&auth, "romanised"), (&other, "original")] {
        let response = request(&app, auth, "GET", "/api/v1/me", Body::empty()).await;
        assert_eq!(read_json(response).await["name_display"], expected);
    }
}

#[sqlx::test]
#[ignore = "requires a disposable PostgreSQL database and AUTH_SECRET"]
async fn listed_identity_review_can_be_dismissed_by_its_id(pool: PgPool) {
    let auth = test_session(&pool, true).await;
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO identity_review (kind, target, detail) VALUES ('artist_alias', 12, 'ambiguous') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let app = crate::routes().layer(Extension(pool));
    let path = "/api/v1/admin/identity-reviews";

    let list = request(&app, &auth, "GET", path, Body::empty()).await;
    assert_eq!(read_json(list).await[0]["id"], id);

    let dismissed = request(
        &app,
        &auth,
        "POST",
        &format!("{}/{}/resolve", path, id),
        Body::empty(),
    )
    .await;
    assert_eq!(dismissed.status(), StatusCode::ACCEPTED);

    let list = request(&app, &auth, "GET", path, Body::empty()).await;
    assert_eq!(read_json(list).await, json!([]));
}

#[sqlx::test]
#[ignore = "requires a disposable PostgreSQL database and AUTH_SECRET"]
async fn album_match_post_rejects_a_candidate_missing_from_that_album(pool: PgPool) {
    let auth = test_session(&pool, true).await;
    let artist: i32 = sqlx::query_scalar(
        "INSERT INTO artist (name, slug, created_at) VALUES ('Artist', 'artist', now()) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let albums: Vec<i32> = sqlx::query_scalar(
        "INSERT INTO album (name, slug, artist, created_at) VALUES
         ('First', 'first', $1, now()), ('Second', 'second', $1, now()) RETURNING id",
    )
    .bind(artist)
    .fetch_all(&pool)
    .await
    .unwrap();
    let release_mbid = "00000000-0000-0000-0000-000000000001";
    sqlx::query(
        "INSERT INTO album_match_candidate (album, release_mbid, score, payload)
         VALUES ($1, $2, 0.8, '{}')",
    )
    .bind(albums[0])
    .bind(release_mbid)
    .execute(&pool)
    .await
    .unwrap();
    let app = crate::routes().layer(Extension(pool));

    let list = request(
        &app,
        &auth,
        "GET",
        &format!("/api/v1/admin/albums/{}/matches", albums[0]),
        Body::empty(),
    )
    .await;
    assert_eq!(read_json(list).await[0]["release_mbid"], release_mbid);

    let response = request(
        &app,
        &auth,
        "POST",
        &format!("/api/v1/admin/albums/{}/match", albums[1]),
        Body::from(json!({ "release_mbid": release_mbid }).to_string()),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"no such candidate");
}

#[tokio::test]
async fn api_misses_return_404_instead_of_the_web_app() {
    let response = request(
        &crate::routes(),
        "",
        "POST",
        "/api/v1/unknown",
        Body::empty(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
