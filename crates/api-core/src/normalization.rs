pub fn optional_language(value: Option<String>) -> Option<String> {
    value
        .map(required_language)
        .and_then(|value| (!value.is_empty()).then_some(value))
}

pub fn required_language(value: String) -> String {
    value.trim().to_ascii_lowercase()
}

pub fn text(value: String) -> String {
    value.trim().to_owned()
}

pub fn path_prefix(path: &str) -> &str {
    let trimmed = path.trim_end_matches('/');
    if trimmed == "/" {
        ""
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::{optional_language, path_prefix};

    #[test]
    fn empty_optional_languages_are_removed() {
        assert_eq!(
            optional_language(Some(" EN ".to_owned())).as_deref(),
            Some("en")
        );
        assert_eq!(optional_language(Some("  ".to_owned())), None);
    }

    #[test]
    fn root_has_no_api_prefix() {
        assert_eq!(path_prefix("/"), "");
        assert_eq!(path_prefix("/admin/"), "/admin");
    }
}
