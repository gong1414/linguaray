//! App language tags (`zh` / `en` / `ja` / `auto`) → vendor codes.
//! Pass-through for already-vendor-shaped tags.

pub fn deepl(code: &str) -> Option<String> {
    if code.is_empty() || code == "auto" {
        return None;
    }
    Some(
        match code {
            "zh" => "ZH",
            "en" => "EN",
            "ja" => "JA",
            "de" => "DE",
            "fr" => "FR",
            "es" => "ES",
            "ko" => "KO",
            "pt" => "PT",
            "it" => "IT",
            "ru" => "RU",
            other => other,
        }
        .to_string(),
    )
}

pub fn microsoft(code: &str) -> Option<String> {
    if code.is_empty() || code == "auto" {
        return None;
    }
    Some(
        match code {
            "zh" => "zh-Hans".to_string(),
            other => other.to_string(),
        },
    )
}

pub fn baidu(code: &str) -> String {
    match code {
        "" | "auto" => "auto".into(),
        "ja" => "jp".into(),
        "ko" => "kor".into(),
        "fr" => "fra".into(),
        "es" => "spa".into(),
        other => other.to_string(),
    }
}

pub fn youdao(code: &str) -> String {
    match code {
        "" | "auto" => "auto".into(),
        "zh" => "zh-CHS".into(),
        other => other.to_string(),
    }
}

pub fn tencent(code: &str) -> String {
    if code.is_empty() {
        "auto".into()
    } else {
        code.to_string()
    }
}
