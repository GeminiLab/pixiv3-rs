//! Error type and shared types (port of pixivpy3.utils).

/// An error occurred in pixiv3-rs.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PixivError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Reqwest error.
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Token required without authentication method provided.
    #[error("token required without authentication method provided")]
    NoAuth,
    /// Bad access token.
    #[error("bad access token \"{access_token}\": {message}")]
    BadAccessToken {
        /// Access token provided.
        access_token: String,
        /// The message.
        message: String,
    },
    /// Response contains error.
    #[error("response contains error: {body}")]
    ErrResponse {
        /// The response body.
        body: String,
    },
    /// Unintelligible response.
    #[error("unintelligible response: {body}")]
    UnintelligibleResponse {
        /// The response body.
        body: String,
    },
    /// Rate limited.
    #[error("rate limited: {body}")]
    RateLimited {
        /// The response body.
        body: String,
    },
    /// Not found.
    #[error("not found: {body}")]
    NotFound {
        /// The response body.
        body: String,
    },
    /// Serde error.
    #[error("serde error: {error}, body: {body}")]
    Serde {
        /// The internal error.
        #[source]
        error: serde_json::Error,
        /// The response body.
        body: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let pixiv_err: PixivError = io_err.into();
        assert!(matches!(pixiv_err, PixivError::Io(_)));
        assert!(pixiv_err.to_string().contains("not found"));
    }

    #[test]
    fn display_no_auth() {
        let err = PixivError::NoAuth;
        assert!(err.to_string().contains("token required"));
    }

    #[test]
    fn display_err_response() {
        let err = PixivError::ErrResponse {
            body: "invalid request".to_string(),
        };
        assert!(err.to_string().contains("invalid request"));
        assert!(err.to_string().contains("error"));
    }

    #[test]
    fn display_rate_limited() {
        let err = PixivError::RateLimited {
            body: "too many requests".to_string(),
        };
        assert!(err.to_string().contains("too many requests"));
    }
}
