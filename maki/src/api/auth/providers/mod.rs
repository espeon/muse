use std::sync::Arc;

use async_trait::async_trait;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    EmptyExtraTokenFields, Scope, StandardTokenResponse,
};
use tokio::sync::Mutex;

pub mod generic;

pub type SharedAuthProvider = Arc<Mutex<dyn AuthProvider>>;

// Information about the account from the external provider
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalAccountInfo {
    // the external provider's unique identifier for the account
    pub provider_id: String,
    // to be filled in per provider
    pub provider: String,
    pub provider_type: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    // unix timestamp
    pub expires_at: Option<i64>,
    // should be all lowercase
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub session_state: Option<String>,
}

// Information about the user from the external provider
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalUserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenIdConfiguration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,

    // Optional fields
    #[serde(default)]
    pub userinfo_endpoint: Option<String>,

    #[serde(default)]
    pub scopes_supported: Option<Vec<String>>,

    #[serde(default)]
    pub grant_types_supported: Option<Vec<String>>,

    #[serde(default)]
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,

    #[serde(default)]
    pub claims_supported: Option<Vec<String>>,

    #[serde(default)]
    pub end_session_endpoint: Option<String>,

    #[serde(default)]
    pub introspection_endpoint: Option<String>,

    #[serde(default)]
    pub revocation_endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OIDCProfileResponse {
    // 'openid' scope claims
    // Subject identifier, uniquely identifying the End-User
    pub sub: String,

    // 'profile' scope claims
    // User's full name in displayable form including all name parts, possibly including titles and suffixes, ordered according to the End-User's locale and preferences.
    pub name: Option<String>,
    // Given name(s) or first name(s) of the End-User
    pub given_name: Option<String>,
    // Surname(s) or last name(s) of the End-User
    pub family_name: Option<String>,
    // Middle name(s) of the End-User
    pub middle_name: Option<String>,
    // Casual name of the End-User that may or may not be the same as the given_name. For instance, a nickname value of Mike might be returned alongside a given_name value of Michael.
    pub nickname: Option<String>,
    // URL of the End-User's profile picture
    pub picture: Option<String>,
    // Time the End-User's information was last updated
    pub updated_at: Option<u64>,

    // Additional claims
    pub preferred_username: Option<String>,
    pub username: Option<String>,

    // 'email' scope claims
    // URL of the End-User's profile picture
    pub email: Option<String>,
    // True if the End-User's e-mail address has been verified; otherwise false
    pub email_verified: Option<bool>,

    // Additional standard OIDC claims from ID Token
    // Issuer Identifier for the Issuer of the response
    pub iss: Option<String>,
    // Audience(s) that this ID Token is intended for
    pub aud: Option<String>,
    // Expiration time on or after which the ID Token MUST NOT be accepted for processing
    pub exp: Option<u64>,
    // Issued At time
    pub iat: Option<u64>,
    // Hash of the ID Token
    pub at_hash: Option<String>,
}
#[async_trait]
/// Trait for authentication providers
/// Expected to have a 'new' method outside the trait that instantiates the provider.
/// See the generic provider for an example.
pub trait AuthProvider: Send + Sync {
    // Get the underlying OAuth2 client
    fn get_client(&self) -> &BasicClient;

    async fn get_openid_configuration(&self) -> anyhow::Result<OpenIdConfiguration>;

    // Generate a challenge and authorization URL
    fn generate_challenge(&mut self) -> anyhow::Result<String>;

    // Verify the challenge and exchange code for token
    async fn verify_challenge(
        &mut self,
        csrf_token: &str,
        auth_code: &str,
    ) -> anyhow::Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>;

    async fn get_user_info(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> anyhow::Result<OIDCProfileResponse>;

    // Get required scopes (can be overridden by specific providers)
    fn get_scopes(&self) -> Vec<Scope> {
        vec![
            // email and profile are not necessarily "required" but we need them for the info we want
            Scope::new("openid".to_string()),
            Scope::new("email".to_string()),
            Scope::new("profile".to_string()),
        ]
    }

    fn make_user_info(&self, profile: OIDCProfileResponse) -> anyhow::Result<ExternalUserInfo> {
        Ok(ExternalUserInfo {
            id: profile.sub.clone(),
            username: profile.preferred_username.unwrap_or(profile.sub),
            email: profile.email.unwrap_or_default(),
            email_verified: profile.email_verified.unwrap_or(false),
            image: profile.picture,
        })
    }
    fn make_account_info(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        profile: &OIDCProfileResponse,
    ) -> anyhow::Result<ExternalAccountInfo>;
}

pub fn create_shared_auth_provider<T>(provider: T) -> SharedAuthProvider
where
    T: AuthProvider + Send + 'static,
{
    Arc::new(Mutex::new(provider))
}
