//! Error type and shared types (port of pixivpy3.utils).

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PixivError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("token required without authentication method provided")]
    NoAuth,
    #[error("bad access token \"{access_token}\": {message}")]
    BadAccessToken {
        access_token: String,
        message: String,
    },
    #[error("response contains error: {body}")]
    ErrResponse { body: String },
    #[error("unintelligible response: {body}")]
    UnintelligibleResponse { body: String },
    #[error("rate limited: {body}")]
    RateLimited { body: String },
    #[error("not found: {body}")]
    NotFound { body: String },
    #[error("serde error: {error}, body: {body}")]
    Serde {
        #[source]
        error: serde_json::Error,
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
