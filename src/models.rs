//! Typed API response models (Rust port of pixivpy3.models).
//!
//! All types use Serde for JSON (de)serialization. Pixiv API returns snake_case;
//! only `WebviewNovel` uses camelCase (from HTML embedding).

use chrono::{DateTime, FixedOffset};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::PixivError;
use crate::{error, warn};

// ----------------------------------------------------------------------------
// User & profile
// ----------------------------------------------------------------------------

/// Profile image URL (medium size).
///
/// 头像图片 URL（中等尺寸）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileImageUrls {
    pub medium: String,
}

/// Basic user info as returned in lists and details.
///
/// 列表与详情中返回的基本用户信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: u64,
    pub name: String,
    pub account: String,
    pub profile_image_urls: ProfileImageUrls,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub is_followed: Option<bool>,
    #[serde(default)]
    pub is_access_blocking_user: Option<bool>,
    #[serde(default)]
    pub is_accept_request: Option<bool>,
}

/// User info as shown in comments.
///
/// 评论中的用户信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentUser {
    pub id: u64,
    pub name: String,
    pub account: String,
    pub profile_image_urls: ProfileImageUrls,
}

/// User profile (detailed).
///
/// 用户资料（详细）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub webpage: Option<String>,
    pub gender: String,
    pub birth: String,
    pub birth_day: String,
    pub birth_year: i64,
    pub region: String,
    pub address_id: i64,
    pub country_code: String,
    pub job: String,
    pub job_id: i64,
    pub total_follow_users: i64,
    pub total_mypixiv_users: i64,
    pub total_illusts: i64,
    pub total_manga: i64,
    pub total_novels: i64,
    pub total_illust_bookmarks_public: i64,
    pub total_illust_series: i64,
    pub total_novel_series: i64,
    pub background_image_url: String,
    pub twitter_account: String,
    pub twitter_url: Option<String>,
    pub pawoo_url: Option<String>,
    pub is_premium: bool,
    pub is_using_custom_profile_image: bool,
}

/// Profile publicity settings.
///
/// 资料公开范围设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePublicity {
    pub gender: String,
    pub region: String,
    pub birth_day: String,
    pub birth_year: String,
    pub job: String,
    pub pawoo: bool,
}

/// User workspace info (tools, desk, etc.).
///
/// 用户工作区信息（工具、桌面等）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub pc: String,
    pub monitor: String,
    pub tool: String,
    pub scanner: String,
    pub tablet: String,
    pub mouse: String,
    pub printer: String,
    pub desktop: String,
    pub music: String,
    pub desk: String,
    pub chair: String,
    pub comment: String,
    pub workspace_image_url: Option<String>,
}

/// Detailed user info (user + profile + workspace). Port of user_detail response.
///
/// 用户详情（用户 + 资料 + 工作区）。对应 user_detail 接口响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoDetailed {
    pub user: UserInfo,
    pub profile: Profile,
    pub profile_publicity: ProfilePublicity,
    pub workspace: Workspace,
}

// ----------------------------------------------------------------------------
// Illust / image
// ----------------------------------------------------------------------------

/// Image URLs for an illust (square, medium, large).
///
/// 插画图片 URL（方形、中等、大图）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrls {
    pub square_medium: String,
    pub medium: String,
    pub large: String,
}

/// Tag on an illustration.
///
/// 插画标签。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustrationTag {
    pub name: String,
    pub translated_name: Option<String>,
}

/// Series info (id and title).
///
/// 系列信息（id 与标题）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub id: u64,
    pub title: String,
}

/// Pixiv returns `{}` instead of `null` for empty objects.
///
/// Pixiv 以 `{}` 表示空对象而非 `null`。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyObject {}

/// Series or empty object (Pixiv uses `{}` for "no series").
///
/// 系列或空对象（Pixiv 用 `{}` 表示无系列）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SeriesOrEmpty {
    Series(Series),
    Empty(EmptyObject),
}

/// Single-page illust meta (original image URL).
///
/// 单页插画 meta（原图 URL）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSinglePage {
    pub original_image_url: Option<String>,
}

/// One page of a multi-page illust (image URLs).
///
/// 多页插画中的一页（图片 URL）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPage {
    pub image_urls: ImageUrls,
}

/// Illustration info (list or detail).
///
/// 插画信息（列表或详情）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustrationInfo {
    pub id: u64,
    pub title: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub image_urls: ImageUrls,
    pub caption: String,
    pub restrict: i32,
    pub user: UserInfo,
    pub tags: Vec<IllustrationTag>,
    pub tools: Vec<String>,
    pub create_date: DateTime<FixedOffset>,
    pub page_count: i32,
    pub width: i32,
    pub height: i32,
    pub sanity_level: i32,
    pub x_restrict: i32,
    pub series: Option<Series>,
    pub meta_single_page: MetaSinglePage,
    pub meta_pages: Vec<MetaPage>,
    pub total_view: i64,
    pub total_bookmarks: i64,
    pub is_bookmarked: bool,
    pub visible: bool,
    pub is_muted: bool,
    pub illust_ai_type: i32,
    pub illust_book_style: i32,
    #[serde(default)]
    pub total_comments: Option<i32>,
    #[serde(default)]
    pub restriction_attributes: Vec<String>,
}

/// Illust detail response (wraps single illust).
///
/// 插画详情响应（单条插画）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustDetail {
    pub illust: IllustrationInfo,
}

// ----------------------------------------------------------------------------
// Novel
// ----------------------------------------------------------------------------

/// Tag on a novel.
///
/// 小说标签。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelTag {
    pub name: String,
    pub translated_name: Option<String>,
    pub added_by_uploaded_user: bool,
}

/// Novel info (list or detail).
///
/// 小说信息（列表或详情）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelInfo {
    pub id: u64,
    pub title: String,
    pub caption: String,
    pub restrict: i32,
    pub x_restrict: i32,
    pub is_original: bool,
    pub image_urls: ImageUrls,
    pub create_date: String,
    pub tags: Vec<NovelTag>,
    pub page_count: i32,
    pub text_length: i64,
    pub user: UserInfo,
    pub series: SeriesOrEmpty,
    pub is_bookmarked: bool,
    pub total_bookmarks: i64,
    pub total_view: i64,
    pub visible: bool,
    pub total_comments: i32,
    pub is_muted: bool,
    pub is_mypixiv_only: bool,
    pub is_x_restricted: bool,
    pub novel_ai_type: i32,
    #[serde(default)]
    pub comment_access_control: Option<i32>,
}

/// Recursive: comment or empty object (Pixiv uses `{}` for no parent).
///
/// 评论或空对象（Pixiv 用 `{}` 表示无父评论）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommentOrEmpty {
    Comment(Box<Comment>),
    Empty(EmptyObject),
}

/// A single comment (illust or novel).
///
/// 单条评论（插画或小说）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub comment: String,
    pub date: String,
    pub user: Option<CommentUser>,
    pub parent_comment: CommentOrEmpty,
}

/// Novel comments list with pagination.
///
/// 小说评论列表（分页）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelComments {
    pub total_comments: i32,
    pub comments: Vec<Comment>,
    pub next_url: Option<String>,
    pub comment_access_control: i32,
}

/// Novel stats (like, bookmark, view counts).
///
/// 小说统计（点赞、收藏、浏览数）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelRating {
    pub like: i64,
    pub bookmark: i64,
    pub view: i64,
}

/// Novel navigation entry in a series (order, title, cover).
///
/// 系列内小说导航项（顺序、标题、封面）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelNavigationInfo {
    pub id: u64,
    pub viewable: bool,
    pub content_order: String,
    pub title: String,
    pub cover_url: String,
    pub viewable_message: Option<String>,
}

/// Series navigation。
///
/// 系列导航信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesNavigation {
    pub prev_novel: Option<NovelNavigationInfo>,
    pub next_novel: Option<NovelNavigationInfo>,
}

/// Series navigation or empty (Pixiv uses `{}` for none).
///
/// 系列导航或空对象（Pixiv 用 `{}` 表示无）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum SeriesNavigationOrEmpty {
    Info(SeriesNavigation),
    Empty(EmptyObject),
}

/// Novel data from webview HTML embedding; uses camelCase.
///
/// 来自 webview HTML 嵌入的小说数据；字段为 camelCase。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewNovel {
    pub id: String,
    pub title: String,
    pub series_id: Option<String>,
    pub series_title: Option<String>,
    pub series_is_watched: Option<bool>,
    pub user_id: String,
    pub cover_url: String,
    pub tags: Vec<String>,
    pub caption: String,
    pub cdate: String,
    pub rating: NovelRating,
    pub text: String,
    pub marker: Option<String>,
    pub illusts: Vec<String>,
    pub images: Vec<String>,
    pub series_navigation: Option<SeriesNavigationOrEmpty>,
    pub glossary_items: Vec<String>,
    pub replaceable_item_ids: Vec<String>,
    pub ai_type: i32,
    pub is_original: bool,
}

// ----------------------------------------------------------------------------
// Response wrappers (illust/user/novel lists)
// ----------------------------------------------------------------------------

/// User bookmarked novels (paged).
///
/// 用户收藏小说列表（分页）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBookmarksNovel {
    pub novels: Vec<NovelInfo>,
    pub next_url: Option<String>,
}

/// User novels list (paged).
///
/// 用户小说列表（分页）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNovels {
    pub user: UserInfo,
    pub novels: Vec<NovelInfo>,
    pub next_url: Option<String>,
}

/// Novel search result (paged).
///
/// 小说搜索结果（分页）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchNovel {
    pub novels: Vec<NovelInfo>,
    pub next_url: Option<String>,
    pub search_span_limit: i32,
    pub show_ai: bool,
}

/// Illust search result (paged).
///
/// 插画搜索结果（分页）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIllustrations {
    pub illusts: Vec<IllustrationInfo>,
    pub next_url: Option<String>,
    pub search_span_limit: i32,
    pub show_ai: bool,
}

/// User bookmarked illusts (paged).
///
/// 用户收藏插画列表（分页）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBookmarksIllustrations {
    pub illusts: Vec<IllustrationInfo>,
    pub next_url: Option<String>,
}

/// User preview (user + sample illusts/novels) in following/follower lists.
///
/// 关注/粉丝列表中的用户预览（用户 + 示例作品）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreview {
    pub user: UserInfo,
    pub illusts: Vec<IllustrationInfo>,
    pub novels: Vec<NovelInfo>,
    pub is_muted: bool,
}

/// User following list (paged).
///
/// 用户关注列表（分页）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFollowing {
    pub user_previews: Vec<UserPreview>,
    pub next_url: Option<String>,
}

/// User illusts list (paged).
///
/// 用户插画列表（分页）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIllustrations {
    pub user: UserInfo,
    pub illusts: Vec<IllustrationInfo>,
    pub next_url: Option<String>,
}

/// OAuth token refresh response (access_token, expires_in, etc.).
///
/// OAuth 刷新 token 的响应（access_token、expires_in 等）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}

// ----------------------------------------------------------------------------
// Utils
// ----------------------------------------------------------------------------

/// Parsed JSON response from API (same as Python `ParsedJson` / `JsonDict`).
/// In Rust we use `serde_json::Value` for attribute-like access via indexing.
///
/// 接口返回的 JSON 解析结果；可用索引方式访问字段。
pub type ParsedJson = serde_json::Value;

// ----------------------------------------------------------------------------
// Parsing
// ----------------------------------------------------------------------------

/// Returns true if the response body is a JSON object with an `"error"` key (API error response).
///
/// 若响应体为带 `"error"` 键的 JSON 对象（API 错误响应）则返回 true。
pub fn is_error_response(res_body: &str) -> bool {
    if let Ok(parsed) = serde_json::from_str::<ParsedJson>(res_body) {
        return parsed.get("error").is_some();
    }
    false
}

/// Deserialize response body string into type `T`. Returns `PixivError::Serde` on parse failure.
///
/// 将响应体字符串反序列化为类型 `T`；解析失败时返回 `PixivError::Serde`。
pub fn parse_into<T: DeserializeOwned, S: AsRef<str> + Into<String>>(
    res_body: S,
) -> Result<T, PixivError> {
    match serde_json::from_str(res_body.as_ref()) {
        Ok(parsed) => Ok(parsed),
        Err(error) => Err(PixivError::Serde {
            error,
            body: res_body.into(),
        }),
    }
}

/// Read response body and deserialize into `T`. Handles rate limit (429), not found (404), and API error payloads.
///
/// 读取响应体并反序列化为 `T`；会处理 429、404 及 API 错误体。
pub async fn parse_response_into<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, PixivError> {
    let status = response.status();
    let body = response.text().await?;

    match status {
        StatusCode::TOO_MANY_REQUESTS => {
            error!("API rate limited: {body}");
            Err(PixivError::RateLimited { body })
        }
        StatusCode::NOT_FOUND => {
            error!("API resource not found: {body}");
            Err(PixivError::NotFound { body })
        }
        _ => {
            if !status.is_success() {
                warn!("API request returned non-success status: {status}, parse body anyway");
            }

            parse_into(body).map_err(|e| {
                // If it failed to parse, check if it's an error response
                if let PixivError::Serde { error, body } = e {
                    if is_error_response(&body) {
                        PixivError::ErrResponse { body }
                    } else {
                        PixivError::Serde { error, body }
                    }
                } else {
                    e
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_user_info() {
        let json = r#"{
            "id": 11,
            "name": "pixiv事務局",
            "account": "pixiv",
            "profile_image_urls": { "medium": "https://example.com/img.jpg" },
            "comment": "hello",
            "is_followed": false
        }"#;
        let user: UserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 11);
        assert_eq!(user.name, "pixiv事務局");
        assert_eq!(user.account, "pixiv");
        assert_eq!(user.comment.as_deref(), Some("hello"));
        assert_eq!(user.is_followed, Some(false));
    }

    #[test]
    fn deserialize_illust_detail() {
        let json = r#"{
            "illust": {
                "id": 12345,
                "title": "test illust",
                "type": "illust",
                "image_urls": {
                    "square_medium": "https://example.com/sq.jpg",
                    "medium": "https://example.com/m.jpg",
                    "large": "https://example.com/l.jpg"
                },
                "caption": "caption",
                "restrict": 0,
                "user": {
                    "id": 1,
                    "name": "user",
                    "account": "acc",
                    "profile_image_urls": { "medium": "https://example.com/p.jpg" }
                },
                "tags": [],
                "tools": [],
                "create_date": "2024-01-01T12:00:00+09:00",
                "page_count": 1,
                "width": 800,
                "height": 600,
                "sanity_level": 2,
                "x_restrict": 0,
                "meta_single_page": {},
                "meta_pages": [],
                "total_view": 100,
                "total_bookmarks": 10,
                "is_bookmarked": false,
                "visible": true,
                "is_muted": false,
                "illust_ai_type": 0,
                "illust_book_style": 0
            }
        }"#;
        let detail: IllustDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.illust.id, 12345);
        assert_eq!(detail.illust.title, "test illust");
        assert_eq!(detail.illust.page_count, 1);
    }

    #[test]
    fn deserialize_empty_series_as_empty_object() {
        let json = r#"{}"#;
        let result: SeriesOrEmpty = serde_json::from_str(json).unwrap();
        assert!(matches!(result, SeriesOrEmpty::Empty(_)));
    }

    #[test]
    fn deserialize_series_as_series_variant() {
        let json = r#"{"id": 1, "title": "My Series"}"#;
        let result: SeriesOrEmpty = serde_json::from_str(json).unwrap();
        match &result {
            SeriesOrEmpty::Series(s) => {
                assert_eq!(s.id, 1);
                assert_eq!(s.title, "My Series");
            }
            SeriesOrEmpty::Empty(_) => panic!("expected Series variant"),
        }
    }

    #[test]
    fn is_error_response_detects_error() {
        let body = r#"{"error": {"message": "invalid token"}}"#;
        assert!(is_error_response(body));
    }

    #[test]
    fn is_error_response_no_error() {
        let body = r#"{"id": 1, "title": "test"}"#;
        assert!(!is_error_response(body));
    }

    #[test]
    fn parse_into_success() {
        let body = r#"{"id": 1, "title": "test"}"#;
        let result: serde_json::Value = parse_into(body.to_string()).unwrap();
        assert_eq!(result["id"], 1);
        assert_eq!(result["title"], "test");
    }

    #[test]
    fn parse_into_invalid_json_returns_serde_error() {
        let body = "not json";
        let err = parse_into::<serde_json::Value, _>(body.to_string()).unwrap_err();
        assert!(matches!(err, PixivError::Serde { .. }));
    }

    #[test]
    fn deserialize_token_refresh_result() {
        let json = r#"{
            "access_token": "abc123",
            "refresh_token": "xyz789",
            "expires_in": 3600
        }"#;
        let result: TokenRefreshResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.access_token, "abc123");
        assert_eq!(result.refresh_token.as_deref(), Some("xyz789"));
        assert_eq!(result.expires_in, Some(3600));
    }
}
