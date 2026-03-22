//! Configuration types for `TripIt` API credentials.
//! OAuth 1.0 request signing for the `TripIt` API.

use base64::{Engine as _, engine::general_purpose::STANDARD as base64};
use hmac::{Hmac, Mac};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use reqwest::header::CONTENT_LENGTH;
use sha1::Sha1;
use thiserror::Error;
use url::form_urlencoded;
use uuid::Uuid;

type HmacSha1 = Hmac<Sha1>;

const REQUEST_TOKEN_URL: &str = "https://api.tripit.com/oauth/request_token";
const AUTHORIZE_URL: &str = "https://www.tripit.com/oauth/authorize";
const ACCESS_TOKEN_URL: &str = "https://api.tripit.com/oauth/access_token";

/// A temporary or permanent OAuth token pair.
#[derive(Debug, Clone)]
pub struct OAuthTokenPair {
    pub token: String,
    pub token_secret: String,
}

/// Errors that can occur during OAuth header generation.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("system clock is before the Unix epoch")]
    Clock(#[from] std::time::SystemTimeError),

    #[error("HMAC key setup failed: {0}")]
    Hmac(String),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("missing {0} in OAuth response")]
    MissingField(&'static str),
}

const OAUTH_ENCODE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn percent_encode(s: &str) -> String {
    percent_encoding::percent_encode(s.as_bytes(), OAUTH_ENCODE).to_string()
}

/// Consumer-only credentials (for auth setup).
#[derive(Debug, Clone)]
pub struct TripItConsumer {
    pub consumer_key: String,
    pub consumer_secret: String,
}

impl TripItConsumer {
    #[must_use]
    pub fn new(consumer_key: String, consumer_secret: String) -> Self {
        Self {
            consumer_key,
            consumer_secret,
        }
    }

    /// # Errors
    ///
    /// Returns `AuthError` if the system clock or HMAC setup fails.
    pub fn to_header(&self, method: &str, url: &str) -> Result<String, AuthError> {
        let nonce = Uuid::new_v4().to_string().replace('-', "");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            .to_string();

        self.to_header_with(method, url, &nonce, &timestamp)
    }

    /// # Errors
    ///
    /// Returns `AuthError` if HMAC key setup fails.
    pub fn to_header_with(
        &self,
        method: &str,
        url: &str,
        nonce: &str,
        timestamp: &str,
    ) -> Result<String, AuthError> {
        TripItAuth::new(
            self.consumer_key.clone(),
            self.consumer_secret.clone(),
            String::new(),
            String::new(),
        )
        .to_header_with(method, url, nonce, timestamp)
    }

    /// # Errors
    ///
    /// Returns `AuthError` on signing, HTTP, or response-parsing failures.
    pub async fn request_token(
        &self,
        client: &reqwest::Client,
    ) -> Result<OAuthTokenPair, AuthError> {
        let auth_header = self.to_header("POST", REQUEST_TOKEN_URL)?;
        let resp = client
            .post(REQUEST_TOKEN_URL)
            .header("Authorization", auth_header)
            .header(CONTENT_LENGTH, "0")
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(AuthError::Http(format!("{status}: {body}")));
        }
        parse_oauth_response(&body)
    }

    #[must_use]
    pub fn authorize_url(request_token: &str, callback_url: &str) -> String {
        format!(
            "{AUTHORIZE_URL}?oauth_token={}&oauth_callback={}",
            percent_encode(request_token),
            percent_encode(callback_url),
        )
    }

    /// # Errors
    ///
    /// Returns `AuthError` on signing, HTTP, or response-parsing failures.
    pub async fn access_token(
        &self,
        client: &reqwest::Client,
        request_token: &OAuthTokenPair,
    ) -> Result<OAuthTokenPair, AuthError> {
        let auth = TripItAuth::new(
            self.consumer_key.clone(),
            self.consumer_secret.clone(),
            request_token.token.clone(),
            request_token.token_secret.clone(),
        );
        let auth_header = auth.to_header("POST", ACCESS_TOKEN_URL)?;
        let resp = client
            .post(ACCESS_TOKEN_URL)
            .header("Authorization", auth_header)
            .header(CONTENT_LENGTH, "0")
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(AuthError::Http(format!("{status}: {body}")));
        }
        parse_oauth_response(&body)
    }
}

// Full credentials including access tokens (for API calls).
#[derive(Debug, Clone)]
pub struct TripItAuth {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

impl TripItAuth {
    #[must_use]
    pub fn new(
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_token_secret: String,
    ) -> Self {
        Self {
            consumer_key,
            consumer_secret,
            access_token,
            access_token_secret,
        }
    }

    /// Build an OAuth 1.0 `Authorization` header for a request.
    ///
    /// This supports both the full API flow (with access token) and the token
    /// exchange handshake (where `token`/`token_secret` may be empty).
    ///
    /// # Errors
    ///
    /// Returns `AuthError` if the system clock or HMAC setup fails.
    pub fn to_header(&self, method: &str, url: &str) -> Result<String, AuthError> {
        let nonce = Uuid::new_v4().to_string().replace('-', "");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            .to_string();

        self.to_header_with(method, url, &nonce, &timestamp)
    }

    /// # Errors
    ///
    /// Returns `AuthError` if HMAC key setup fails.
    pub fn to_header_with(
        &self,
        method: &str,
        url: &str,
        nonce: &str,
        timestamp: &str,
    ) -> Result<String, AuthError> {
        let mut params = vec![
            ("oauth_consumer_key", self.consumer_key.clone()),
            ("oauth_nonce", nonce.to_string()),
            ("oauth_signature_method", "HMAC-SHA1".to_string()),
            ("oauth_timestamp", timestamp.to_string()),
            ("oauth_version", "1.0".to_string()),
        ];

        if !self.access_token.is_empty() {
            params.push(("oauth_token", self.access_token.clone()));
        }

        params.sort_by(|a, b| a.0.cmp(b.0));

        let sorted_params = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let base = format!(
            "{}&{}&{}",
            method.to_uppercase(),
            percent_encode(url),
            percent_encode(&sorted_params),
        );

        let signing_key = format!(
            "{}&{}",
            percent_encode(&self.consumer_secret),
            percent_encode(&self.access_token_secret),
        );

        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
            .map_err(|e| AuthError::Hmac(e.to_string()))?;
        mac.update(base.as_bytes());
        let signature = base64.encode(mac.finalize().into_bytes());

        params.push(("oauth_signature", signature));

        let header = params
            .iter()
            .map(|(k, v)| format!(r#"{}="{}""#, k, percent_encode(v)))
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!("OAuth {header}"))
    }
}

fn parse_oauth_response(body: &str) -> Result<OAuthTokenPair, AuthError> {
    let params: std::collections::HashMap<String, String> = form_urlencoded::parse(body.as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let token = params
        .get("oauth_token")
        .filter(|v| !v.is_empty())
        .ok_or(AuthError::MissingField("oauth_token"))?
        .clone();
    let token_secret = params
        .get("oauth_token_secret")
        .filter(|v| !v.is_empty())
        .ok_or(AuthError::MissingField("oauth_token_secret"))?
        .clone();

    Ok(OAuthTokenPair {
        token,
        token_secret,
    })
}

#[cfg(test)]
mod tests {
    use super::{TripItAuth, TripItConsumer, percent_encode};
    use base64::{Engine as _, engine::general_purpose::STANDARD as base64};
    use std::collections::BTreeMap;
    use url::form_urlencoded;

    fn parse_oauth_header(header: &str) -> BTreeMap<String, String> {
        assert!(header.starts_with("OAuth "));
        header
            .trim_start_matches("OAuth ")
            .split(", ")
            .map(|part| {
                let mut split = part.splitn(2, '=');
                let key = split
                    .next()
                    .expect("header part should contain a key")
                    .to_string();
                let quoted = split.next().expect("header part should contain a value");
                let value = quoted
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                    .expect("header value should be quoted")
                    .to_string();
                (key, value)
            })
            .collect()
    }

    fn oauth_signature(header: &str) -> String {
        parse_oauth_header(header)
            .get("oauth_signature")
            .expect("oauth_signature should exist")
            .clone()
    }

    fn percent_decode(value: &str) -> String {
        form_urlencoded::parse(format!("v={value}").as_bytes())
            .next()
            .expect("should parse key/value")
            .1
            .to_string()
    }

    #[test]
    fn percent_encode_edge_cases() {
        assert_eq!(percent_encode(""), "");
        assert_eq!(percent_encode("hello world"), "hello%20world");
        assert_eq!(
            percent_encode("!*'();:@&=+$,/?#[]"),
            "%21%2A%27%28%29%3B%3A%40%26%3D%2B%24%2C%2F%3F%23%5B%5D"
        );
        assert_eq!(
            percent_encode("你好 🌍"),
            "%E4%BD%A0%E5%A5%BD%20%F0%9F%8C%8D"
        );
    }

    #[test]
    fn to_header_snapshot_with_fixed_nonce_and_timestamp() {
        let auth = TripItAuth::new(
            "consumer-key".to_string(),
            "consumer-secret".to_string(),
            "access-token".to_string(),
            "access-token-secret".to_string(),
        );

        let header = auth
            .to_header_with(
                "POST",
                "https://api.tripit.com/v1/list/trip",
                "fixednonce123",
                "1700000000",
            )
            .unwrap();

        assert_eq!(
            header,
            "OAuth oauth_consumer_key=\"consumer-key\", oauth_nonce=\"fixednonce123\", oauth_signature_method=\"HMAC-SHA1\", oauth_timestamp=\"1700000000\", oauth_token=\"access-token\", oauth_version=\"1.0\", oauth_signature=\"2St2pz%2BLGrdOtYOg71dh61grRZw%3D\""
        );
    }

    #[test]
    fn consumer_only_header_is_valid_and_has_no_oauth_token() {
        let consumer =
            TripItConsumer::new("consumer-key".to_string(), "consumer-secret".to_string());
        let header = consumer
            .to_header_with(
                "POST",
                "https://api.tripit.com/oauth/request_token",
                "nonce-consumer",
                "1700000001",
            )
            .unwrap();

        let fields = parse_oauth_header(&header);
        assert!(!fields.contains_key("oauth_token"));
        assert_eq!(
            fields.get("oauth_consumer_key"),
            Some(&"consumer-key".to_string())
        );
        assert_eq!(
            fields.get("oauth_nonce"),
            Some(&"nonce-consumer".to_string())
        );
        assert_eq!(
            fields.get("oauth_timestamp"),
            Some(&"1700000001".to_string())
        );

        let signature = fields
            .get("oauth_signature")
            .expect("signature should be present");
        assert!(
            base64.decode(percent_decode(signature)).is_ok(),
            "signature should be valid base64"
        );
    }

    #[test]
    fn full_auth_header_includes_oauth_token() {
        let auth = TripItAuth::new(
            "consumer-key".to_string(),
            "consumer-secret".to_string(),
            "access-token".to_string(),
            "access-token-secret".to_string(),
        );

        let header = auth
            .to_header_with(
                "GET",
                "https://api.tripit.com/v1/get/trip/id/123/format/json",
                "nonce-full",
                "1700000002",
            )
            .unwrap();

        let fields = parse_oauth_header(&header);
        assert_eq!(fields.get("oauth_token"), Some(&"access-token".to_string()));
        let signature = fields
            .get("oauth_signature")
            .expect("signature should be present");
        assert!(
            base64.decode(percent_decode(signature)).is_ok(),
            "signature should be valid base64"
        );
    }

    #[test]
    fn signature_changes_when_inputs_change() {
        let base_auth = TripItAuth::new(
            "consumer-key".to_string(),
            "consumer-secret".to_string(),
            "access-token".to_string(),
            "access-token-secret".to_string(),
        );

        let base_header = base_auth
            .to_header_with(
                "GET",
                "https://api.tripit.com/v1/list/trip",
                "same-nonce",
                "1700000000",
            )
            .unwrap();
        let base_sig = oauth_signature(&base_header);

        let changed_method_header = base_auth
            .to_header_with(
                "POST",
                "https://api.tripit.com/v1/list/trip",
                "same-nonce",
                "1700000000",
            )
            .unwrap();
        assert_ne!(base_sig, oauth_signature(&changed_method_header));

        let changed_url_header = base_auth
            .to_header_with(
                "GET",
                "https://api.tripit.com/v1/list/trip/past/true",
                "same-nonce",
                "1700000000",
            )
            .unwrap();
        assert_ne!(base_sig, oauth_signature(&changed_url_header));

        let changed_consumer_secret = TripItAuth::new(
            "consumer-key".to_string(),
            "different-consumer-secret".to_string(),
            "access-token".to_string(),
            "access-token-secret".to_string(),
        )
        .to_header_with(
            "GET",
            "https://api.tripit.com/v1/list/trip",
            "same-nonce",
            "1700000000",
        )
        .unwrap();
        assert_ne!(base_sig, oauth_signature(&changed_consumer_secret));

        let changed_access_secret = TripItAuth::new(
            "consumer-key".to_string(),
            "consumer-secret".to_string(),
            "access-token".to_string(),
            "different-access-secret".to_string(),
        )
        .to_header_with(
            "GET",
            "https://api.tripit.com/v1/list/trip",
            "same-nonce",
            "1700000000",
        )
        .unwrap();
        assert_ne!(base_sig, oauth_signature(&changed_access_secret));
    }

    #[test]
    fn oauth_parameter_ordering_is_alphabetical() {
        let auth = TripItAuth::new(
            "consumer-key".to_string(),
            "consumer-secret".to_string(),
            "access-token".to_string(),
            "access-token-secret".to_string(),
        );
        let header = auth
            .to_header_with(
                "GET",
                "https://api.tripit.com/v1/list/trip",
                "nonce-order",
                "1700000100",
            )
            .unwrap();

        let without_prefix = header.trim_start_matches("OAuth ");
        let keys: Vec<&str> = without_prefix
            .split(", ")
            .map(|part| {
                part.split('=')
                    .next()
                    .expect("all header parts should include key")
            })
            .collect();

        assert_eq!(
            keys,
            vec![
                "oauth_consumer_key",
                "oauth_nonce",
                "oauth_signature_method",
                "oauth_timestamp",
                "oauth_token",
                "oauth_version",
                "oauth_signature",
            ]
        );
    }
}
