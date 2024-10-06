use async_trait::async_trait;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, RedirectUrl, StandardTokenResponse, TokenResponse, TokenUrl,
};
use time::Duration;

use crate::api::auth::providers::OIDCProfileResponse;

use super::{AuthProvider, ExternalAccountInfo, OpenIdConfiguration};

pub struct GenericOidcPkceProvider {
    pub client: BasicClient,
    pub reqwest: reqwest::Client,
    pub pkce_store: crate::api::auth::pkce_store::PkceStore,
    pub auth_issuer: String,
}

impl GenericOidcPkceProvider {
    pub async fn new(
        reqwest: reqwest::Client,
        client_id: String,
        client_secret: String,
        auth_authorize: String,
        auth_issuer: String,
        auth_token_url: String,
        redirect_uri: String,
    ) -> anyhow::Result<Self> {
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_authorize)?,
            Some(TokenUrl::new(auth_token_url)?),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        Ok(Self {
            reqwest,
            client,
            pkce_store: crate::api::auth::pkce_store::PkceStore::new(),
            auth_issuer,
        })
    }
}

#[async_trait]
impl AuthProvider for GenericOidcPkceProvider {
    async fn get_openid_configuration(&self) -> anyhow::Result<OpenIdConfiguration> {
        Ok(self
            .reqwest
            .get(format!(
                "{}/.well-known/openid-configuration",
                self.auth_issuer
            ))
            .send()
            .await?
            .json::<OpenIdConfiguration>()
            .await?)
    }
    fn get_client(&self) -> &BasicClient {
        &self.client
    }

    fn generate_challenge(&mut self) -> anyhow::Result<String> {
        // Generate a PKCE challenge.
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL.
        let (auth_url, csrf_token) = self
            .get_client()
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            .add_scopes(self.get_scopes())
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();

        // Store the PKCE code verifier. Expires in 10 minutes, maybe should be shorter?
        self.pkce_store.insert(
            csrf_token.secret().to_string(),
            pkce_verifier,
            Duration::minutes(10),
        );

        Ok(auth_url.to_string())
    }

    async fn verify_challenge(
        &mut self,
        csrf_token: &str,
        auth_code: &str,
    ) -> anyhow::Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        if let Some(pkce_verifier) = self.pkce_store.get(csrf_token) {
            // trade verifier for access token
            Ok(self
                .get_client()
                .exchange_code(AuthorizationCode::new(auth_code.to_string()))
                // Set the PKCE code verifier.
                .set_pkce_verifier(pkce_verifier)
                .request_async(async_http_client)
                .await?)
        } else {
            anyhow::bail!("CSRF token not found in store");
        }
    }

    async fn get_user_info(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> anyhow::Result<OIDCProfileResponse> {
        let token = token.access_token().secret();
        Ok(self
            .reqwest
            .get(format!("{}oidc/v1/userinfo", self.auth_issuer))
            .bearer_auth(token)
            .send()
            .await?
            .json::<OIDCProfileResponse>()
            .await?)
    }

    fn make_account_info(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        profile: &OIDCProfileResponse,
    ) -> anyhow::Result<ExternalAccountInfo> {
        Ok(ExternalAccountInfo {
            provider_id: profile.sub.clone(),
            provider: "Generic".to_string(),
            provider_type: "oidc".to_string(),
            access_token: Some(token.access_token().secret().to_string()),
            refresh_token: token.refresh_token().map(|t| t.secret().to_string()),
            // add expires in seconds to unix timestamp
            expires_at: Some(
                token
                    .expires_in()
                    .map(|t| {
                        (time::OffsetDateTime::now_utc().unix_timestamp() as f64 + t.as_secs_f64())
                            as i64
                    })
                    .unwrap_or(0),
            ),
            token_type: Some(format!("{:?}", token.token_type())),
            scope: token.scopes().map(|s| {
                s.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            }),
            session_state: None,
        })
    }
}
