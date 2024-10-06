use itertools::Itertools;
use md5::Digest;
use md5::Md5;
use reqwest::StatusCode;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::debug;
use tracing::error;

/// Generates an md5 hash of all the sorted query params in kv format, all squooshed together
/// Ex. api_keyfoomethodbartokenbazmysecret
pub fn generate_api_sig(params: &[(&str, &str)], secret: &str) -> String {
    let concat = params
        .iter()
        .sorted_by_key(|&(k, _)| k)
        .flat_map(|&(k, v)| [k, v])
        .chain(std::iter::once(secret))
        .collect::<String>();

    let digest = Md5::digest(concat.as_bytes());
    format!("{:x}", digest)
}

// struct FmNowPlayingResponse {
//     session_key: String,
//     username: String,
// }

pub async fn set_now_playing(
    id: i32,
    pool: &PgPool,
    track_name: &str,
    track_artist: &str,
    track_album: &str,
    track_duration: u32,
) -> anyhow::Result<()> {
    debug!("Setting now playing on Last.fm for user {}", id);
    let fm_creds = sqlx::query!(
        r#"
        SELECT lastfm_session_key FROM user_lastfm WHERE "userId" = $1
    "#,
        id
    )
    .fetch_one(pool)
    .await?;
    let session_key = fm_creds.lastfm_session_key;

    let body_params: &[(&str, &str)] = &[
        ("method", "track.updateNowPlaying"),
        ("sk", &session_key),
        (
            "api_key",
            &std::env::var("FM_KEY").expect("FM_KEY must be set"),
        ),
        ("artist", track_artist),
        ("track", track_name),
        ("album", track_album),
        ("duration", &track_duration.to_string()),
    ];
    let api_sig = generate_api_sig(
        body_params,
        &std::env::var("FM_SECRET").expect("FM_SECRET must be set"),
    );

    let other_body_params: &[(&str, &str)] = &[("api_sig", &api_sig), ("format", "json")];

    let client = reqwest::Client::new();

    // append all body params
    let form = body_params
        .iter()
        .chain(other_body_params.iter())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<Vec<_>>();

    match client
        .post("http://ws.audioscrobbler.com/2.0/")
        // post form body
        .form(&form)
        .send()
        .await
    {
        Ok(response) => match response.status() {
            StatusCode::OK => {
                debug!("Successfully set now playing on Last.fm");
                // print out response
                let response = response.text().await?;
                debug!("Last.fm response: {:?}", response);
            }
            _ => {
                let status = response.status();
                let text = response.text().await?;
                error!("Failed to set now playing on Last.fm: {}", text);
                return Err(anyhow::anyhow!(status));
            }
        },
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to set now playing on Last.fm: {}",
                e
            ));
        }
    }
    Ok(())
}

pub async fn scrobble(
    id: i32,
    pool: &PgPool,
    track_name: &str,
    track_artist: &str,
    track_album: &str,
    track_duration: u32,
) -> anyhow::Result<()> {
    debug!("Scrobbling song to Last.fm for user {}", id);
    let fm_creds = sqlx::query!(
        r#"
        SELECT lastfm_session_key FROM user_lastfm WHERE "userId" = $1
    "#,
        id
    )
    .fetch_one(pool)
    .await?;
    let session_key = fm_creds.lastfm_session_key;

    // get timestamp UTC
    let timestamp = OffsetDateTime::now_utc().unix_timestamp();

    let body_params: &[(&str, &str)] = &[
        ("method", "track.scrobble"),
        ("sk", &session_key),
        (
            "api_key",
            &std::env::var("FM_KEY").expect("FM_KEY must be set"),
        ),
        ("artist", track_artist),
        ("track", track_name),
        ("album", track_album),
        ("timestamp", &timestamp.to_string()),
        ("duration", &track_duration.to_string()),
    ];
    let api_sig = generate_api_sig(
        body_params,
        &std::env::var("FM_SECRET").expect("FM_SECRET must be set"),
    );

    let other_body_params: &[(&str, &str)] = &[("api_sig", &api_sig), ("format", "json")];

    let client = reqwest::Client::new();

    // append all body params
    let form = body_params
        .iter()
        .chain(other_body_params.iter())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<Vec<_>>();

    match client
        .post("http://ws.audioscrobbler.com/2.0/")
        // post form body
        .form(&form)
        .send()
        .await
    {
        Ok(response) => match response.status() {
            StatusCode::OK => {
                debug!("Successfully scrobbled on Last.fm");
                // print out response
                let response = response.text().await?;
                debug!("Last.fm response: {:?}", response);
            }
            _ => {
                let status = response.status();
                error!("Failed to scrobble on Last.fm: {}", status);
                return Err(anyhow::anyhow!("Failed to scrobble on Last.fm: {}", status));
            }
        },
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to scrobble on Last.fm: {}", e));
        }
    }
    Ok(())
}
