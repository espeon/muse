//! Routes oauth2's token requests through the same `reqwest::Client` the rest of the
//! server uses.
//!
//! oauth2 is built with `default-features = false` on purpose: its bundled `reqwest`
//! feature pins reqwest 0.12, which would compile a second copy of the whole HTTP stack
//! for a single token exchange. Supplying the client here keeps one version in the tree.

use std::convert::TryInto;
use std::future::Future;
use std::pin::Pin;

use axum::http::Response;
use oauth2::{AsyncHttpClient, HttpClientError, HttpRequest, HttpResponse};

/// Wrapper that lets oauth2 drive a `reqwest::Client`.
pub struct OAuthHttpClient(pub reqwest::Client);

impl<'c> AsyncHttpClient<'c> for OAuthHttpClient {
    type Error = HttpClientError<reqwest::Error>;
    type Future =
        Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + Sync + 'c>>;

    fn call(&'c self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            let response = self
                .0
                .execute(request.try_into().map_err(Box::new)?)
                .await
                .map_err(Box::new)?;

            let mut builder = Response::builder()
                .status(response.status())
                .version(response.version());

            for (name, value) in response.headers() {
                builder = builder.header(name, value);
            }

            builder
                .body(response.bytes().await.map_err(Box::new)?.to_vec())
                .map_err(HttpClientError::Http)
        })
    }
}
