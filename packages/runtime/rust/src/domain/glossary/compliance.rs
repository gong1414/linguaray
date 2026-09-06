use std::collections::HashSet;

use super::{GlossaryComplianceIssue, GlossaryIssueKind, GlossaryMatch};

/// Reports which glossary rules `translated` breaks, at most one issue per
/// entry per kind however often the term occurs in the source.
pub fn check_compliance(
    matches: &[GlossaryMatch],
    translated: &str,
) -> Vec<GlossaryComplianceIssue> {
    let haystack = translated.to_lowercase();
    let mut issues = Vec::new();
    let mut seen = HashSet::new();

    for hit in matches {
        if !seen.insert((hit.book_id.as_str(), hit.entry_id.as_str())) {
            continue;
        }
        if !haystack.contains(&hit.translation.to_lowercase()) {
            issues.push(GlossaryComplianceIssue {
                book_id: hit.book_id.clone(),
                entry_id: hit.entry_id.clone(),
                kind: GlossaryIssueKind::MissingTranslation,
                term: hit.term.clone(),
                expected: hit.translation.clone(),
                found: None,
            });
        }
        for forbidden in &hit.forbidden {
            if haystack.contains(&forbidden.to_lowercase()) {
                issues.push(GlossaryComplianceIssue {
                    book_id: hit.book_id.clone(),
                    entry_id: hit.entry_id.clone(),
                    kind: GlossaryIssueKind::ForbiddenUsed,
                    term: hit.term.clone(),
                    expected: hit.translation.clone(),
                    found: Some(forbidden.clone()),
                });
            }
        }
    }
    issues
}
