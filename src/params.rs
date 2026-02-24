//! API parameter enums (filter, restrict, sort, etc.).

// we consider fields in these structs self-descriptive enough
#![allow(missing_docs)]

use kv_pairs::impl_into_value_by_into_str_ref;
use strum::IntoStaticStr;

/// Filter type for API (e.g. for_ios).
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[non_exhaustive]
pub enum Filter {
    #[strum(serialize = "for_ios")]
    ForIos,
}

/// Content/illust type: illust or manga.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[non_exhaustive]
pub enum IllustType {
    #[strum(serialize = "illust")]
    Illust,
    #[strum(serialize = "manga")]
    Manga,
}

/// Restrict: public or private.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[non_exhaustive]
pub enum Restrict {
    #[strum(serialize = "public")]
    Public,
    #[strum(serialize = "private")]
    Private,
}

/// Ranking mode for illust_ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[non_exhaustive]
pub enum RankingMode {
    #[strum(serialize = "day")]
    Day,
    #[strum(serialize = "week")]
    Week,
    #[strum(serialize = "month")]
    Month,
    #[strum(serialize = "day_male")]
    DayMale,
    #[strum(serialize = "day_female")]
    DayFemale,
    #[strum(serialize = "week_original")]
    WeekOriginal,
    #[strum(serialize = "week_rookie")]
    WeekRookie,
    #[strum(serialize = "day_r18")]
    DayR18,
    #[strum(serialize = "day_male_r18")]
    DayR18Male,
    #[strum(serialize = "day_female_r18")]
    DayR18Female,
    #[strum(serialize = "week_r18")]
    WeekR18,
    #[strum(serialize = "week_r18g")]
    WeekR18Global,
}

/// Search target for search_illust / search_novel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[non_exhaustive]
pub enum SearchTarget {
    #[strum(serialize = "partial_match_for_tags")]
    PartialMatchForTags,
    #[strum(serialize = "exact_match_for_tags")]
    ExactMatchForTags,
    #[strum(serialize = "title_and_caption")]
    TitleAndCaption,
    #[strum(serialize = "keyword")]
    Keyword,
}

/// Sort order for search and listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[non_exhaustive]
pub enum Sort {
    #[strum(serialize = "date_desc")]
    DateDesc,
    #[strum(serialize = "date_asc")]
    DateAsc,
    #[strum(serialize = "popular_desc")]
    PopularDesc,
    #[strum(serialize = "popular_asc")]
    PopularAsc,
}

/// Duration filter for search (past day/week/month).
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[non_exhaustive]
pub enum Duration {
    #[strum(serialize = "last_day")]
    LastDay,
    #[strum(serialize = "last_week")]
    LastWeek,
    #[strum(serialize = "last_month")]
    LastMonth,
}

impl_into_value_by_into_str_ref! {
    Filter,
    IllustType,
    Restrict,
    RankingMode,
    SearchTarget,
    Sort,
    Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_to_str() {
        let s: &'static str = Filter::ForIos.into();
        assert_eq!(s, "for_ios");
    }

    #[test]
    fn illust_type_to_str() {
        assert_eq!(<&'static str>::from(IllustType::Illust), "illust");
        assert_eq!(<&'static str>::from(IllustType::Manga), "manga");
    }

    #[test]
    fn restrict_to_str() {
        assert_eq!(<&'static str>::from(Restrict::Public), "public");
        assert_eq!(<&'static str>::from(Restrict::Private), "private");
    }

    #[test]
    fn ranking_mode_to_str() {
        assert_eq!(<&'static str>::from(RankingMode::Day), "day");
        assert_eq!(
            <&'static str>::from(RankingMode::DayR18Male),
            "day_male_r18"
        );
        assert_eq!(
            <&'static str>::from(RankingMode::WeekR18Global),
            "week_r18g"
        );
    }

    #[test]
    fn search_target_to_str() {
        assert_eq!(
            <&'static str>::from(SearchTarget::PartialMatchForTags),
            "partial_match_for_tags"
        );
        assert_eq!(<&'static str>::from(SearchTarget::Keyword), "keyword");
    }

    #[test]
    fn sort_to_str() {
        assert_eq!(<&'static str>::from(Sort::DateDesc), "date_desc");
        assert_eq!(<&'static str>::from(Sort::DateAsc), "date_asc");
        assert_eq!(<&'static str>::from(Sort::PopularDesc), "popular_desc");
    }

    #[test]
    fn duration_to_str() {
        assert_eq!(<&'static str>::from(Duration::LastDay), "last_day");
        assert_eq!(<&'static str>::from(Duration::LastWeek), "last_week");
        assert_eq!(<&'static str>::from(Duration::LastMonth), "last_month");
    }
}
