use base64::Engine;
use josekit::jwe::{self, JweHeader};
use ring::hkdf;
use serde::{Deserialize, Serialize};

use anyhow::anyhow;
use time::OffsetDateTime;

#[derive(Deserialize, Serialize, Debug)]
pub struct PartialSessionToken {
    pub name: Option<String>,
    pub email: String,
    pub picture: Option<String>,
    /// Subject - the user's unique identifier
    pub sub: String,
    pub acl: Option<String>,
}

impl PartialSessionToken {
    /// Converts a partial session token into a session token. Consumes the partial session token.
    pub fn into_session_token(self, iat: i64, exp: i64, jti: String) -> SessionToken {
        SessionToken {
            name: self.name,
            email: self.email,
            picture: self.picture,
            sub: self.sub,
            acl: self.acl,
            iat,
            exp,
            jti,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionToken {
    pub name: Option<String>,
    pub email: String,
    pub picture: Option<String>,
    pub acl: Option<String>,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
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

pub fn encode_jwe(
    partial_payload: PartialSessionToken,
    cookie_name: &str,
    expiration: time::Duration,
) -> anyhow::Result<String> {
    let auth_secret = std::env::var("AUTH_SECRET")
        .map_err(|_| anyhow!("AUTH_SECRET is not defined in the environment"))?;

    // Create a JWE header
    let mut header = JweHeader::new();
    header.set_content_encryption("A256CBC-HS512");
    header.set_token_type("JWT");

    // generate a random key
    let key = derive_key(
        auth_secret.as_bytes(),
        cookie_name.as_bytes(),
        format!("Auth.js Generated Encryption Key ({})", cookie_name).as_bytes(),
        64,
    )?;

    // for now, the jti will be b64 of the sha256 of the key
    // TODO: change this to be actually random, and store it in the database
    let jti = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(ring::digest::digest(&ring::digest::SHA256, key.as_slice()));

    // Create the payload
    let iat = OffsetDateTime::now_utc();
    let exp = iat + expiration;
    let payload =
        partial_payload.into_session_token(iat.unix_timestamp(), exp.unix_timestamp(), jti);

    // Serialize the payload to JSON
    let payload_json = serde_json::to_string(&payload)?;

    // Encrypt the payload
    let encrypter = jwe::Dir.encrypter_from_bytes(key)?;
    let jwe = jwe::serialize_compact(payload_json.as_bytes(), &header, &encrypter)?;

    Ok(jwe)
}

pub fn verify_jwt(jwt: &SessionToken) -> anyhow::Result<()> {
    // get current unix timestamp
    let now = OffsetDateTime::now_utc().unix_timestamp();
    // Verify the iat and exp
    if jwt.iat > now || jwt.exp < now {
        return Err(anyhow!(
            "Token is invalid because it has an invalid time. Likely expired."
        ));
    }

    Ok(())
}
pub fn decode_jwe(jwe: &str, cookie_name: &str) -> anyhow::Result<SessionToken> {
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

    // Deserialize the payload
    let payload: SessionToken = serde_json::from_str(&data)?;

    // Verify JWT
    verify_jwt(&payload)?;

    Ok(payload)
}
