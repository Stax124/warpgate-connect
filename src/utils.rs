use nucleo_matcher::Utf32Str;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use ratatui::style::Color;

use crate::warpgate::structs::{WarpgateFilterableTarget, WarpgateTarget, WarpgateTargetGroup};

pub fn get_color_from_group_color(group_color: Option<&str>) -> Color {
    match group_color {
        Some("Primary") => Color::Blue,
        Some("Danger") => Color::Red,
        Some("Warning") => Color::Yellow,
        Some("Success") => Color::Green,
        _ => Color::Gray,
    }
}

/// A target that survived filtering, with the positions in its name that the query matched.
#[derive(Debug, Clone)]
pub struct MatchedTarget {
    pub target: WarpgateTarget,
    pub name_indices: Vec<u32>,
}

/// Narrows `targets` to the SSH ones in `group`, then ranks them against `query` by fuzzy match.
pub fn filter_targets(
    targets: &[WarpgateTarget],
    group: Option<&WarpgateTargetGroup>,
    query: &str,
) -> Vec<MatchedTarget> {
    let mut matcher = nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT.match_paths());

    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let matches = pattern.match_list(
        targets
            .iter()
            .filter(|t| t.is_ssh())
            .filter(|t| match group {
                Some(group) => t.group.as_ref().is_some_and(|g| g.name == group.name),
                None => true,
            })
            .cloned()
            .map(WarpgateFilterableTarget::new),
        &mut matcher,
    );

    let mut name_buffer = Vec::new();

    matches
        .into_iter()
        .map(|m| {
            let target = m.0.warpgate_target;

            // Ranking runs over "name (description)" so that a description search works, which
            // leaves its positions unusable for the name column; these are matched a second time.
            let mut name_indices = Vec::new();
            pattern.indices(
                Utf32Str::new(&target.name, &mut name_buffer),
                &mut matcher,
                &mut name_indices,
            );
            name_indices.sort_unstable();
            name_indices.dedup();

            MatchedTarget {
                name_indices,
                target,
            }
        })
        .collect()
}

struct RankedName<'a> {
    index: usize,
    name: &'a str,
}

impl AsRef<str> for RankedName<'_> {
    fn as_ref(&self) -> &str {
        self.name
    }
}

/// Ranks `names` against `query`, returning the indices of the ones that matched, best first.
pub fn rank_names<'a>(names: impl IntoIterator<Item = &'a str>, query: &str) -> Vec<usize> {
    let mut matcher = nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT.match_paths());

    Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart)
        .match_list(
            names
                .into_iter()
                .enumerate()
                .map(|(index, name)| RankedName { index, name }),
            &mut matcher,
        )
        .into_iter()
        .map(|(ranked, _)| ranked.index)
        .collect()
}

pub fn get_domain_from_warpgate_url(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"^https?://([^:/]+)").unwrap();
    re.captures(url)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

#[cfg(test)]
#[path = "utils_test.rs"]
mod tests;
