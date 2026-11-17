//! Host extraction for building absolute URLs in API responses.
//!
//! axum 0.7 removed its built-in `Host` extractor and axum-extra does not ship a
//! replacement, so this is a port of axum 0.7's implementation. The precedence
//! order matters and must be kept: `Forwarded` and `X-Forwarded-Host` take
//! priority so that requests arriving through a reverse proxy report the public
//! host rather than the proxy's upstream address.

use axum::{
    extract::FromRequestParts,
    http::{
        header::{HeaderMap, FORWARDED},
        request::Parts,
        StatusCode,
    },
};

const X_FORWARDED_HOST_HEADER_KEY: &str = "X-Forwarded-Host";

/// Extractor that resolves the hostname of the request.
///
/// Hostname is resolved through the following, in order:
/// - `Forwarded` header
/// - `X-Forwarded-Host` header
/// - `Host` header
/// - request target / URI
///
/// Clients can set these headers to arbitrary values, so callers must validate
/// the result before trusting it.
pub struct Host(pub String);

impl<S> FromRequestParts<S> for Host
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(host) = parse_forwarded(&parts.headers) {
            return Ok(Host(host.to_owned()));
        }

        if let Some(host) = parts
            .headers
            .get(X_FORWARDED_HOST_HEADER_KEY)
            .and_then(|host| host.to_str().ok())
        {
            return Ok(Host(host.to_owned()));
        }

        if let Some(host) = parts
            .headers
            .get(axum::http::header::HOST)
            .and_then(|host| host.to_str().ok())
        {
            return Ok(Host(host.to_owned()));
        }

        if let Some(host) = parts.uri.host() {
            return Ok(Host(host.to_owned()));
        }

        Err((StatusCode::BAD_REQUEST, "No host found in request"))
    }
}

/// Extracts the `host` field out of the first element of the `Forwarded` header.
fn parse_forwarded(headers: &HeaderMap) -> Option<&str> {
    // If there are multiple `Forwarded` headers, `get` returns the first one.
    let forwarded_values = headers.get(FORWARDED)?.to_str().ok()?;

    // Only the first set of values applies to the client that made the request.
    let first_value = forwarded_values.split(',').next()?;

    first_value.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("host")
            .then(|| value.trim().trim_matches('"'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderName, HeaderValue};

    fn header_map(values: &[(HeaderName, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (key, value) in values {
            headers.append(key.clone(), HeaderValue::from_str(value).unwrap());
        }
        headers
    }

    #[test]
    fn forwarded_host_is_parsed() {
        let headers = header_map(&[(FORWARDED, "host=192.0.2.60;proto=http;by=203.0.113.43")]);
        assert_eq!(parse_forwarded(&headers), Some("192.0.2.60"));
    }

    #[test]
    fn forwarded_key_is_case_insensitive() {
        let headers = header_map(&[(FORWARDED, "Host=192.0.2.60")]);
        assert_eq!(parse_forwarded(&headers), Some("192.0.2.60"));
    }

    #[test]
    fn forwarded_ipv6_host_is_unquoted() {
        let headers = header_map(&[(FORWARDED, "host=\"[2001:db8:cafe::17]:4711\"")]);
        assert_eq!(parse_forwarded(&headers), Some("[2001:db8:cafe::17]:4711"));
    }

    #[test]
    fn forwarded_uses_first_value() {
        let headers = header_map(&[(FORWARDED, "host=192.0.2.60, host=127.0.0.1")]);
        assert_eq!(parse_forwarded(&headers), Some("192.0.2.60"));
    }

    #[test]
    fn forwarded_without_host_is_ignored() {
        let headers = header_map(&[(FORWARDED, "proto=http;by=203.0.113.43")]);
        assert_eq!(parse_forwarded(&headers), None);
    }
}
