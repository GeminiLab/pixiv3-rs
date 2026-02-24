//! App Pixiv API (6.x app-api.pixiv.net) - port of pixivpy3.aapi.AppPixivAPI.
//! Includes base logic: auth, HTTP client, download (from BasePixivAPI).

use std::{sync::LazyLock, time::Duration};

use kv_pairs::{KVPairs, kv_pairs};
use reqwest::header::{AUTHORIZATION, HOST, HeaderMap, HeaderName, HeaderValue as HV, USER_AGENT};
use serde::de::DeserializeOwned;
use tokio::io::AsyncWriteExt;

use pixiv3_rs_proc::api_endpoints;

use crate::debug;
use crate::error::PixivError;
use crate::models::*;
use crate::params::*;
use crate::token_manager::TokenManager;

/// Simple HTTP method enum for internal requests.
///
/// 内部请求使用的简单 HTTP 方法枚举。
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum HttpMethod {
    GET,
    POST,
    DELETE,
}

/// App-API (6.x) client. Port of `AppPixivAPI` (with base auth/HTTP/download inlined).
pub struct AppPixivAPI {
    hosts: String,
    client: reqwest::Client,
    token_manager: TokenManager,
}

impl AppPixivAPI {
    /// Create an API client with no authentication. All authenticated calls will fail with `PixivError::NoAuth`.
    ///
    /// 创建无认证的 API 客户端；所有需认证的请求将返回 `PixivError::NoAuth`。
    pub fn new_no_auth() -> Self {
        debug!("Creating AppPixivAPI with no authentication");
        Self::new_with(TokenManager::new_no_auth())
    }

    /// Create an API client from an existing access token. No refresh is performed.
    ///
    /// 使用已有的 access token 创建 API 客户端，不会自动刷新。
    pub fn new_from_access_token(access_token: String) -> Self {
        debug!("Creating AppPixivAPI with access token");
        Self::new_with(TokenManager::new_from_access_token(access_token))
    }

    /// Create an API client from a refresh token. Access token will be obtained/refreshed on demand.
    ///
    /// 使用 refresh token 创建 API 客户端，access token 将在需要时获取或刷新。
    pub fn new_from_refresh_token(refresh_token: String) -> Self {
        debug!("Creating AppPixivAPI with refresh token");
        Self::new_with(TokenManager::new_from_refresh_token(refresh_token))
    }

    fn new_with(token_manager: TokenManager) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("reqwest client");
        Self {
            hosts: "https://app-api.pixiv.net".to_string(),
            client,
            token_manager,
        }
    }

    /// Require that auth has been set; otherwise return error.
    pub async fn get_access_token(&self) -> Result<String, PixivError> {
        self.token_manager.get_access_token().await
    }

    /// Set proxy hosts (e.g. pixivlite.com). Port of `set_api_proxy`.
    pub fn set_api_proxy(&mut self, proxy_hosts: &str) {
        self.hosts = proxy_hosts.to_string();
    }

    /// Low-level HTTP call (port of `requests_call`).
    async fn do_http_request(
        &self,
        method: HttpMethod,
        url: &str,
        headers: Option<HeaderMap>,
        params: Option<KVPairs<'_>>,
        data: Option<KVPairs<'_>>,
    ) -> Result<reqwest::Response, PixivError> {
        let mut req = match method {
            HttpMethod::GET => self.client.get(url),
            HttpMethod::POST => self.client.post(url),
            HttpMethod::DELETE => self.client.delete(url),
        };
        if let Some(h) = headers {
            req = req.headers(h);
        }
        if let Some(p) = params {
            req = req.query(&p.content);
        }
        if let Some(d) = data {
            req = req.form(&d.content);
        }
        let res = req.send().await?;
        Ok(res)
    }

    /// Performs an API request with optional auth and app headers. Use for custom endpoints.
    ///
    /// 执行 API 请求，可附加认证头与 App 头；在需要调用未由生成方法覆盖的接口时使用。
    pub async fn do_api_request(
        &self,
        method: HttpMethod,
        url: &str,
        headers: Option<HeaderMap>,
        params: Option<KVPairs<'_>>,
        data: Option<KVPairs<'_>>,
        with_auth: bool,
    ) -> Result<reqwest::Response, PixivError> {
        let mut headers = headers.unwrap_or_default();
        if self.hosts != "https://app-api.pixiv.net" {
            headers.insert(HOST, HV::from_static("app-api.pixiv.net"));
        }

        if !headers.contains_key("user-agent") {
            headers.insert(HeaderName::from_static("app-os"), HV::from_static("ios"));
            headers.insert(
                HeaderName::from_static("app-os-version"),
                HV::from_static("14.6"),
            );
            headers.insert(
                USER_AGENT,
                HV::from_static("PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)"),
            );
        }
        if with_auth {
            let access_token = self.get_access_token().await?;
            headers.insert(
                AUTHORIZATION,
                HV::from_str(&format!("Bearer {}", access_token)).map_err(|e| {
                    PixivError::BadAccessToken {
                        access_token,
                        message: format!("{}", e),
                    }
                })?,
            );
        }
        self.do_http_request(method, url, Some(headers), params, data)
            .await
    }
}

/// Structured API calls (generated by `pixiv3-rs-proc`).
impl AppPixivAPI {
    api_endpoints!(
        /// User detail. Port of `user_detail`.
        ///
        /// 用户详情。
        user_detail -> UserInfoDetailed {
            GET "/v1/user/detail",
            params [
                user_id: u64,
                filter: Option<Filter> = Filter::ForIos,
            ],
        };

        /// User illusts list. Port of `user_illusts`.
        ///
        /// 用户作品列表。
        user_illusts -> UserIllustrations (paged illusts: IllustrationInfo) {
            GET "/v1/user/illusts",
            params [
                user_id: u64,
                type_ @ "type": Option<IllustType> = IllustType::Illust,
                filter: Option<Filter>,
                offset: Option<&str>,
            ]
        };

        /// User bookmarked illusts. Port of `user_bookmarks_illust`.
        ///
        /// 用户收藏作品列表。
        user_bookmarks_illust -> UserBookmarksIllustrations (paged illusts: IllustrationInfo) {
            GET "/v1/user/bookmarks/illust",
            params [
                user_id: u64,
                restrict: Option<Restrict> = Restrict::Public,
                filter: Option<Filter> = Filter::ForIos,
                max_bookmark_id: Option<&str>,
                tag: Option<&str>,
            ]
        };

        /// User bookmarked novels. Port of `user_bookmarks_novel`.
        ///
        /// 用户收藏小说列表。
        user_bookmarks_novel -> UserBookmarksNovel (paged novels: NovelInfo) {
            GET "/v1/user/bookmarks/novel",
            params [
                user_id: u64,
                restrict: Option<Restrict> = Restrict::Public,
                filter: Option<Filter> = Filter::ForIos,
                max_bookmark_id: Option<&str>,
                tag: Option<&str>,
            ]
        };

        /// Related users. Port of `user_related`. offset sent as "0" when None.
        ///
        /// 相关用户。
        user_related -> ParsedJson {
            GET "/v1/user/related",
            params [
                seed_user_id: u64,
                filter: Option<Filter> = Filter::ForIos,
                offset: Option<&str> = "0",
            ]
        };

        /// Recommended users. Port of `user_recommended`.
        ///
        /// 推荐用户。
        user_recommended -> ParsedJson {
            GET "/v1/user/recommended",
            params [
                filter: Option<Filter> = Filter::ForIos,
                offset: Option<&str>,
            ]
        };

        /// New works from followed users. Port of `illust_follow`.
        ///
        /// 关注用户的新作。
        illust_follow -> ParsedJson {
            GET "/v2/illust/follow",
            params [
                restrict: Option<Restrict> = Restrict::Public,
                offset: Option<&str>,
            ]
        };

        /// Illust detail. Port of `illust_detail`.
        ///
        /// 作品详情。
        illust_detail -> IllustDetail {
            GET "/v1/illust/detail",
            params [ illust_id: u64 ]
        };

        /// Illust comments. Port of `illust_comments`.
        ///
        /// 作品评论。
        illust_comments -> ParsedJson {
            GET "/v3/illust/comments",
            params [
                illust_id: u64,
                offset: Option<&str>,
                include_total_comments: Option<bool>,
            ]
        };

        /// Illust ranking. Port of `illust_ranking`.
        ///
        /// 作品排行。
        illust_ranking -> ParsedJson {
            GET "/v1/illust/ranking",
            params [
                mode: Option<RankingMode> = RankingMode::Day,
                filter: Option<Filter> = Filter::ForIos,
                date: Option<&str>,
                offset: Option<&str>,
            ]
        };

        /// Trending tags for illust. Port of `trending_tags_illust`.
        ///
        /// 趋势标签。
        trending_tags_illust -> ParsedJson {
            GET "/v1/trending-tags/illust",
            params [ filter: Option<Filter> = Filter::ForIos ]
        };

        /// Search illusts. Port of `search_illust`.
        ///
        /// 搜索插画。
        search_illust -> SearchIllustrations (paged illusts: IllustrationInfo) {
            GET "/v1/search/illust",
            params [
                word: &str,
                search_target: Option<SearchTarget> = SearchTarget::PartialMatchForTags,
                sort: Option<Sort> = Sort::DateDesc,
                duration: Option<&str>,
                start_date: Option<&str>,
                end_date: Option<&str>,
                filter: Option<Filter> = Filter::ForIos,
                search_ai_type: Option<u8>,
                offset: Option<&str>,
            ]
        };

        /// Search novels. Port of `search_novel`.
        ///
        /// 搜索小说。
        search_novel -> SearchNovel (paged novels: NovelInfo) {
            GET "/v1/search/novel",
            params [
                word: &str,
                search_target: Option<SearchTarget> = SearchTarget::PartialMatchForTags,
                sort: Option<Sort> = Sort::DateDesc,
                merge_plain_keyword_results: Option<&str> = "true",
                include_translated_tag_results: Option<&str> = "true",
                start_date: Option<&str>,
                end_date: Option<&str>,
                filter: Option<&str>,
                search_ai_type: Option<u8>,
                offset: Option<&str>,
            ]
        };

        /// Search users. Port of `search_user`.
        ///
        /// 搜索用户。
        search_user -> ParsedJson {
            GET "/v1/search/user",
            params [
                word: &str,
                sort: Option<Sort> = Sort::DateDesc,
                duration: Option<&str>,
                filter: Option<Filter> = Filter::ForIos,
                offset: Option<&str>,
            ]
        };

        /// Illust bookmark detail. Port of `illust_bookmark_detail`.
        ///
        /// 作品收藏详情。
        illust_bookmark_detail -> ParsedJson {
            GET "/v2/illust/bookmark/detail",
            params [ illust_id: u64 ]
        };

        /// User bookmark tags for illust. Port of `user_bookmark_tags_illust`.
        ///
        /// 用户收藏标签列表。
        user_bookmark_tags_illust -> ParsedJson {
            GET "/v1/user/bookmark-tags/illust",
            params [
                user_id: u64,
                restrict: Option<Restrict> = Restrict::Public,
                offset: Option<&str>,
            ]
        };

        /// User following list. Port of `user_following`.
        ///
        /// Following 用户列表。
        user_following -> UserFollowing (paged user_previews: UserPreview) {
            GET "/v1/user/following",
            params [
                user_id: u64,
                restrict: Option<Restrict> = Restrict::Public,
                offset: Option<&str>,
            ]
        };

        /// User followers. Port of `user_follower`.
        ///
        /// Followers 用户列表。
        user_follower -> ParsedJson {
            GET "/v1/user/follower",
            params [
                user_id: u64,
                filter: Option<Filter> = Filter::ForIos,
                offset: Option<&str>,
            ]
        };

        /// MyPixiv friends. Port of `user_mypixiv`.
        ///
        /// 好P友。
        user_mypixiv -> ParsedJson {
            GET "/v1/user/mypixiv",
            params [ user_id: u64, offset: Option<&str> ]
        };

        /// User list (blocklist). Port of `user_list`.
        ///
        /// 黑名单用户。
        user_list -> ParsedJson {
            GET "/v2/user/list",
            params [
                user_id: u64,
                filter: Option<Filter> = Filter::ForIos,
                offset: Option<&str>,
            ]
        };

        /// Ugoira metadata. Port of `ugoira_metadata`.
        ///
        /// 获取 ugoira 信息。
        ugoira_metadata -> ParsedJson {
            GET "/v1/ugoira/metadata",
            params [ illust_id: u64 ]
        };

        /// User novels list. Port of `user_novels`.
        ///
        /// 用户小说列表。
        user_novels -> UserNovels (paged novels: NovelInfo) {
            GET "/v1/user/novels",
            params [
                user_id: u64,
                filter: Option<Filter> = Filter::ForIos,
                offset: Option<&str>,
            ]
        };

        /// Novel series detail. Port of `novel_series`.
        ///
        /// 小说系列详情。
        novel_series -> ParsedJson {
            GET "/v2/novel/series",
            params [
                series_id: u64,
                filter: Option<Filter> = Filter::ForIos,
                last_order: Option<&str>,
            ]
        };

        /// Novel detail. Port of `novel_detail`.
        ///
        /// 小说详情。
        novel_detail -> NovelInfo {
            GET "/v2/novel/detail",
            params [ novel_id: u64 ]
        };

        /// Novel comments. Port of `novel_comments`.
        ///
        /// 小说评论。
        novel_comments -> NovelComments (paged comments: Comment) {
            GET "/v1/novel/comments",
            params [
                novel_id: u64,
                offset: Option<&str>,
                include_total_comments: Option<bool>,
            ]
        };

        /// New novels. Port of `novel_new`.
        ///
        /// 小说新作。
        novel_new -> ParsedJson {
            GET "/v1/novel/new",
            params [
                filter: Option<Filter> = Filter::ForIos,
                max_novel_id: Option<&str>,
            ]
        };

        /// New illusts from everyone. Port of `illust_new`.
        ///
        /// 大家的新作。
        illust_new -> ParsedJson {
            GET "/v1/illust/new",
            params [
                content_type: Option<IllustType> = IllustType::Illust,
                filter: Option<Filter> = Filter::ForIos,
                max_illust_id: Option<&str>,
            ]
        };

        /// New novels from followed users. Port of `novel_follow`.
        ///
        /// 正在关注的用户的新小说。
        novel_follow -> ParsedJson {
            GET "/v1/novel/follow",
            params [
                restrict: Option<Restrict> = Restrict::Public,
                offset: Option<u32>,
            ]
        };

        /// Delete bookmark. Port of `illust_bookmark_delete`.
        ///
        /// 删除收藏。
        illust_bookmark_delete -> EmptyObject {
            POST "/v1/illust/bookmark/delete",
            data [ illust_id: u64 ]
        };

        /// Follow user. Port of `user_follow_add`. Python default: restrict="public".
        ///
        /// 关注用户。
        user_follow_add -> EmptyObject {
            POST "/v1/user/follow/add",
            data [
                user_id: u64,
                restrict: Option<Restrict> = Restrict::Public,
            ]
        };

        /// Unfollow user. Port of `user_follow_delete`.
        ///
        /// 取消关注用户。
        user_follow_delete -> EmptyObject {
            POST "/v1/user/follow/delete",
            data [ user_id: u64 ]
        };

        /// Edit user AI-show setting. Port of `user_edit_ai_show_settings`.
        ///
        /// 设置用户选项中是否展现AI生成作品。
        user_edit_ai_show_settings -> EmptyObject {
            POST "/v1/user/ai-show-settings/edit",
            data [ setting @ "show_ai": &str ]
        };

        /// Related illusts. Port of `illust_related`. Python defaults: filter="for_ios".
        ///
        /// 相关作品列表。
        illust_related -> ParsedJson {
            GET "/v2/illust/related",
            params [
                illust_id: u64,
                filter: Option<Filter> = Filter::ForIos,
                seed_illust_ids @ "seed_illust_ids[]": Option<&[String]> => seed_illust_ids.unwrap_or(&[]),
                offset: Option<&str>,
                viewed @ "viewed[]": Option<&[String]> => viewed.unwrap_or(&[]),
            ]
        };

        /// Add bookmark. Port of `illust_bookmark_add`. Python default: restrict="public".
        ///
        /// 新增收藏。
        illust_bookmark_add -> ParsedJson {
            POST "/v2/illust/bookmark/add",
            data [
                illust_id: u64,
                restrict: Option<Restrict> = Restrict::Public,
                tags @ "tags[]": Option<&[String]> => tags.map(|t| t.join(" ")),
            ]
        };

        /// Recommended novels. Port of `novel_recommended`. Python defaults: include_ranking_label=True, filter="for_ios".
        ///
        /// 小说推荐。
        novel_recommended -> ParsedJson {
            GET "/v1/novel/recommended",
            params [
                include_ranking_label: Option<bool> = true,
                filter: Option<Filter> = Filter::ForIos,
                offset: Option<&str>,
                include_ranking_novels: Option<bool>,
                already_recommended: Option<&[String]> => already_recommended.map(|arr| arr.join(",")),
                max_bookmark_id_for_recommend: Option<&str>,
                include_privacy_policy: Option<&str>,
            ]
        };
    );
}

/// Non-structured API calls (port of `AppPixivAPI` methods).
impl AppPixivAPI {
    // ---------- Illust (manual: URL by with_auth) ----------
    /// Recommended illusts. Port of `illust_recommended`. Python defaults: content_type="illust", include_ranking_label=True, filter="for_ios".
    ///
    /// 插画推荐。
    #[allow(clippy::too_many_arguments)]
    pub async fn illust_recommended(
        &self,
        content_type: Option<IllustType>,
        include_ranking_label: Option<bool>,
        filter: Option<Filter>,
        max_bookmark_id_for_recommend: Option<&str>,
        min_bookmark_id_for_recent_illust: Option<&str>,
        offset: Option<&str>,
        include_ranking_illusts: Option<bool>,
        bookmark_illust_ids: Option<&[String]>,
        include_privacy_policy: Option<&str>,
        viewed: Option<&[String]>,
        with_auth: bool,
    ) -> Result<ParsedJson, PixivError> {
        let content_type = content_type.unwrap_or(IllustType::Illust);
        let include_ranking_label = include_ranking_label.unwrap_or(true);
        let filter = filter.unwrap_or(Filter::ForIos);
        let url = if with_auth {
            format!("{}/v1/illust/recommended", self.hosts)
        } else {
            format!("{}/v1/illust/recommended-nologin", self.hosts)
        };
        let mut params = kv_pairs!(
            "content_type" => content_type,
            "include_ranking_label" => include_ranking_label,
            "filter" => filter,
        );
        params.push(
            "max_bookmark_id_for_recommend",
            max_bookmark_id_for_recommend,
        );
        params.push(
            "min_bookmark_id_for_recent_illust",
            min_bookmark_id_for_recent_illust,
        );
        params.push("offset", offset);
        params.push("include_ranking_illusts", include_ranking_illusts);
        if let Some(v) = viewed {
            for x in v {
                params.push_owned("viewed[]", x.clone());
            }
        }
        if !with_auth {
            if let Some(ids) = bookmark_illust_ids {
                params.push_owned("bookmark_illust_ids", ids.join(","));
            }
        }
        params.push("include_privacy_policy", include_privacy_policy);
        let r = self
            .do_api_request(HttpMethod::GET, &url, None, Some(params), None, with_auth)
            .await?;
        parse_response_into(r).await
    }

    /// Novel via webview, raw HTML. Port of `webview_novel(raw=True)`.
    ///
    /// 小说 (webview) 的原始 HTML 表示。
    pub async fn webview_novel_raw(
        &self,
        novel_id: u64,
        with_auth: bool,
    ) -> Result<String, PixivError> {
        let url = format!("{}/webview/v2/novel", self.hosts);
        let params = kv_pairs!(
            "id" => novel_id,
            "viewer_version" => "20221031_ai",
        );
        let r = self
            .do_api_request(HttpMethod::GET, &url, None, Some(params), None, with_auth)
            .await?;
        Ok(r.text().await?)
    }

    /// Novel via webview. Port of `webview_novel(raw=False)`.
    ///
    /// 小说 (webview)。
    pub async fn webview_novel(
        &self,
        novel_id: u64,
        with_auth: bool,
    ) -> Result<WebviewNovel, PixivError> {
        /// Cached regex for extracting novel JSON from webview response (avoids recompiling on every call).
        static WEBVIEW_NOVEL_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"novel:\s(\{.+\}),\s+isOwnWork").expect("valid regex")
        });

        let text = self.webview_novel_raw(novel_id, with_auth).await?;

        match WEBVIEW_NOVEL_REGEX.captures(&text).and_then(|c| c.get(1)) {
            Some(json_str) => parse_into(json_str.as_str()),
            None => Err(PixivError::UnintelligibleResponse { body: text }),
        }
    }

    /// Showcase article detail (no login required). Port of `showcase_article`. Manual: custom headers / host.
    ///
    /// 特辑详情（无需登录）。
    pub async fn showcase_article(&self, showcase_id: u64) -> Result<ParsedJson, PixivError> {
        let url = "https://www.pixiv.net/ajax/showcase/article";
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HV::from_static(
                "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/63.0.3239.132 Safari/537.36",
            ),
        );
        headers.insert("Referer", HV::from_static("https://www.pixiv.net"));
        let params = kv_pairs!(
            "article_id" => showcase_id,
        );
        let r = self
            .do_api_request(
                HttpMethod::GET,
                url,
                Some(headers),
                Some(params),
                None,
                false,
            )
            .await?;
        parse_response_into(r).await
    }

    /// Download URL to file. Port of `download`.
    ///
    /// 将 URL 下载到文件。
    pub async fn download(
        &self,
        url: &str,
        path: &std::path::Path,
        name: Option<&str>,
        replace: bool,
        referer: &str,
    ) -> Result<bool, PixivError> {
        let filename = name.unwrap_or_else(|| url.split('/').next_back().unwrap_or("download"));
        let filepath = path.join(filename);
        if !replace && tokio::fs::try_exists(&filepath).await.unwrap_or(false) {
            return Ok(false);
        }
        let mut res = self
            .client
            .get(url)
            .header("Referer", referer)
            .send()
            .await?;

        let mut file = tokio::fs::File::create(&filepath).await?;
        while let Some(chunk) = res.chunk().await? {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        Ok(true)
    }
}

/// Paged API calls (NOT port of `AppPixivAPI` methods).
impl AppPixivAPI {
    /// Fetch the next page of results from a paged API response. The URL is typically from `next_url` in the previous response.
    ///
    /// 从分页 API 响应的 next_url 获取下一页结果。
    pub async fn visit_next_url<T: DeserializeOwned>(
        &self,
        next_url: &str,
        with_auth: bool,
    ) -> Result<T, PixivError> {
        let r = self
            .do_api_request(HttpMethod::GET, next_url, None, None, None, with_auth)
            .await?;
        parse_response_into(r).await
    }
}
