//! Token management for Pixiv OAuth: no-auth, access-token, or refresh-token.

use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwapOption;
use chrono::{DateTime, Utc};
use kv_pairs::kv_pairs;
use tokio::sync::Mutex as AsyncMutex;

use crate::PixivError;
use crate::models::{TokenRefreshResult, parse_into};
use crate::{debug, info};

/// Pixiv OAuth token endpoint.
pub const AUTH_TOKEN_URL: &str = "https://oauth.secure.pixiv.net/auth/token";
/// Default OAuth client ID (Pixiv iOS app).
///
/// 默认 OAuth 客户端 ID（Pixiv iOS 应用）。
pub const DEFAULT_CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
/// Default OAuth client secret (Pixiv iOS app).
///
/// 默认 OAuth 客户端密钥（Pixiv iOS 应用）。
pub const DEFAULT_CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";
/// Hash secret used for Pixiv auth.
///
/// 用于 Pixiv 认证的哈希密钥。
pub const HASH_SECRET: &str = "28c1fdd170a5204386cb1313c7077b34f83e4aaf4aa829ce78c231e05b0bae2c";
/// User-Agent sent when refreshing token.
///
/// 刷新 token 时发送的 User-Agent。
pub const AUTH_USER_AGENT: &str = "PixivAndroidApp/5.0.234 (Android 11; Pixel 5)";

/// Default access token lifetime in seconds.
///
/// 默认 access token 有效时间（秒）。
pub const DEFAULT_EXPIRES_IN: u64 = 3600;
/// Token refresh safe margin in seconds.
///
/// 刷新 token 安全边距（秒）。
pub const TOKEN_REFRESH_SAFE_MARGIN: u64 = 300;

/// Token manager: no auth, access token only, or refresh token with automatic refresh.
///
/// Token 管理器：无认证、仅 access token、或带自动刷新的 refresh token。
pub enum TokenManager {
    NoAuth,
    AccessToken {
        access_token: String,
    },
    RefreshToken {
        refresh_token: String,
        access_token_and_expires_at: ArcSwapOption<(String, DateTime<Utc>)>,
        update_lock: AsyncMutex<()>,
    },
}

impl TokenManager {
    /// Create a token manager with no authentication. All authenticated requests will fail.
    ///
    /// 创建无认证的 token 管理器；所有需认证的请求将失败。
    pub fn new_no_auth() -> Self {
        Self::NoAuth
    }

    /// Create a token manager from an existing access token. No refresh is performed.
    ///
    /// 使用已有的 access token 创建 token 管理器，不会自动刷新。
    pub fn new_from_access_token(access_token: String) -> Self {
        Self::AccessToken { access_token }
    }

    /// Create a token manager from a refresh token. Access token will be obtained/refreshed on demand.
    ///
    /// 使用 refresh token 创建 token 管理器，access token 将在需要时获取或刷新。
    pub fn new_from_refresh_token(refresh_token: String) -> Self {
        Self::RefreshToken {
            refresh_token,
            access_token_and_expires_at: ArcSwapOption::default(),
            update_lock: AsyncMutex::new(()),
        }
    }

    fn try_get_saved_token(
        access_token_and_expires_at: &ArcSwapOption<(String, DateTime<Utc>)>,
    ) -> Result<String, ()> {
        if let Some((access_token, expires_at)) = access_token_and_expires_at.load().as_deref() {
            if *expires_at > Utc::now() {
                return Ok(access_token.clone());
            }
        }
        Err(())
    }

    async fn try_refresh_token(refresh_token: &str) -> Result<(String, DateTime<Utc>), PixivError> {
        let client = reqwest::Client::new();
        let request = client
            .post(AUTH_TOKEN_URL)
            .form(
                &kv_pairs![
                    "client_id" =>  DEFAULT_CLIENT_ID,
                    "client_secret" => DEFAULT_CLIENT_SECRET,
                    "grant_type" => "refresh_token",
                    "include_policy" => "true",
                    "refresh_token" => refresh_token,
                ]
                .content,
            )
            .header("User-Agent", AUTH_USER_AGENT);
        let response = request.send().await?;
        let parsed: TokenRefreshResult = parse_into(response.text().await?)?;

        let access_token = parsed.access_token;
        let expires_at = Utc::now()
            + Duration::from_secs(
                match parsed.expires_in {
                    Some(sec) if sec > 0 => sec as u64,
                    _ => DEFAULT_EXPIRES_IN,
                } - TOKEN_REFRESH_SAFE_MARGIN,
            );

        Ok((access_token, expires_at))
    }

    /// Returns the current access token, refreshing from refresh token if necessary.
    ///
    /// 返回当前 access token，若为 refresh token 模式则在需要时自动刷新。
    pub async fn get_access_token(&self) -> Result<String, PixivError> {
        match self {
            Self::NoAuth => Err(PixivError::NoAuth),
            Self::AccessToken { access_token } => Ok(access_token.clone()),
            Self::RefreshToken {
                access_token_and_expires_at,
                update_lock,
                refresh_token,
            } => {
                // Try to get saved token
                if let Ok(access_token) = Self::try_get_saved_token(access_token_and_expires_at) {
                    return Ok(access_token);
                }

                debug!("Token not set or expired, trying to refresh");

                // Token not set or expired, try to update
                let mut _lock = update_lock.lock().await;

                // Has any other thread already updated the token?
                if let Ok(access_token) = Self::try_get_saved_token(access_token_and_expires_at) {
                    debug!("Token already updated by another thread");
                    return Ok(access_token);
                }

                // Refresh token
                info!("Refreshing token");
                let (access_token, expires_at) = Self::try_refresh_token(refresh_token).await?;
                info!("Token refreshed successfully, expires at {}", expires_at);
                access_token_and_expires_at
                    .store(Some(Arc::new((access_token.clone(), expires_at))));
                Ok(access_token)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_auth_returns_error() {
        let tm = TokenManager::new_no_auth();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tm.get_access_token());
        assert!(matches!(result, Err(PixivError::NoAuth)));
    }

    #[test]
    fn access_token_returns_token() {
        let tm = TokenManager::new_from_access_token("test_token".into());
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tm.get_access_token());
        assert_eq!(result.unwrap(), "test_token");
    }
}
