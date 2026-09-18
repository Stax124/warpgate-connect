use nucleo_matcher::Utf32Str;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use ratatui_textarea::TextArea;

use crate::warpgate::target::{WarpgateTarget, WarpgateTargetGroup};

pub fn first_line<'a>(text_area: &'a TextArea<'_>) -> &'a str {
    text_area.lines().first().map_or("", |line| line.as_str())
}

/// The trimmed contents of a single-line input, or `None` where the user left it blank.
pub fn trimmed_line(text_area: &TextArea) -> Option<String> {
    let trimmed = first_line(text_area).trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
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
    let candidates: Vec<(&WarpgateTarget, String)> = targets
        .iter()
        .filter(|t| t.is_ssh())
        .filter(|t| match group {
            Some(group) => t.group.as_ref().is_some_and(|g| g.name == group.name),
            None => true,
        })
        .map(|target| {
            let haystack = format!(
                "{} ({})",
                target.name,
                target.description.as_deref().unwrap_or("")
            );
            (target, haystack)
        })
        .collect();

    let ranked = rank_names(
        candidates.iter().map(|(_, haystack)| haystack.as_str()),
        query,
    );

    let mut matcher = nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT.match_paths());
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut name_buffer = Vec::new();

    ranked
        .into_iter()
        .map(|index| {
            let target = candidates[index].0;

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
                target: target.clone(),
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

/// The login Warpgate routes on: the account, then the target it should be forwarded to.
pub fn warpgate_ssh_username(warpgate_username: &str, target_name: &str) -> String {
    format!("{warpgate_username}:{target_name}")
}

pub fn get_domain_from_warpgate_url(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;

    let host = rest.split([':', '/']).next().unwrap_or_default();
    (!host.is_empty()).then(|| host.to_string())
}

#[cfg(test)]
#[path = "utils_test.rs"]
mod tests;
