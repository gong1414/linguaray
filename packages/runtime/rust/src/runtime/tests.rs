use super::*;

fn unique_data_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "linguaray-runtime-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos()
    ))
}

fn create_runtime() -> Arc<Runtime> {
    let data_dir = unique_data_dir();
    Runtime::new(data_dir.display().to_string()).expect("failed to create runtime")
}

fn current_timestamp_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backwards")
        .as_millis()
        .try_into()
        .expect("timestamp does not fit in u64")
}

#[test]
fn commit_settings_updates_last_updated() {
    let runtime = create_runtime();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let before = current_timestamp_millis();
            runtime
                .clone()
                .settings()
                .update_appearance(AppearanceSettingsPatch {
                    language: Some("en".to_owned()),
                    theme_mode: None,
                    theme: None,
                })
                .await
                .expect("failed to update appearance");
            let after = current_timestamp_millis();

            let json = runtime
                .clone()
                .settings()
                .get_json()
                .await
                .expect("failed to get settings json");
            let value = serde_json::from_str::<serde_json::Value>(&json)
                .expect("settings json should parse");
            let last_updated = value
                .get("lastUpdated")
                .and_then(serde_json::Value::as_u64)
                .expect("lastUpdated should be a number");

            assert!(last_updated >= before);
            assert!(last_updated <= after);
        });
}

#[test]
fn update_shortcuts_persists_all_fields_to_settings_file() {
    let data_dir = unique_data_dir();
    let settings_file = data_dir.join("settings.json");
    let runtime = Runtime::new(data_dir.display().to_string()).expect("failed to create runtime");

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            runtime
                .clone()
                .settings()
                .update_shortcuts(ShortcutSettingsPatch {
                    toggle_mini_translator: Some("Command+Shift+Space".to_owned()),
                    extract_text_from_screen_selection: Some("Command+Shift+1".to_owned()),
                    extract_text_from_screen_capture: Some("Command+Shift+2".to_owned()),
                    extract_text_from_clipboard: Some("Command+Shift+3".to_owned()),
                    translate_input_content: Some("Option+Z".to_owned()),
                })
                .await
                .expect("failed to update shortcuts");
        });

    let saved = std::fs::read_to_string(settings_file).expect("failed to read settings file");
    let json = serde_json::from_str::<serde_json::Value>(&saved).expect("invalid settings json");
    assert_eq!(
        json.pointer("/shortcuts/toggleMiniTranslator").cloned(),
        Some(serde_json::Value::String("Command+Shift+Space".to_owned()))
    );
    assert_eq!(
        json.pointer("/shortcuts/extractTextFromScreenSelection")
            .cloned(),
        Some(serde_json::Value::String("Command+Shift+1".to_owned()))
    );
    assert_eq!(
        json.pointer("/shortcuts/extractTextFromScreenCapture")
            .cloned(),
        Some(serde_json::Value::String("Command+Shift+2".to_owned()))
    );
    assert_eq!(
        json.pointer("/shortcuts/extractTextFromClipboard").cloned(),
        Some(serde_json::Value::String("Command+Shift+3".to_owned()))
    );
    assert_eq!(
        json.pointer("/shortcuts/translateInputContent").cloned(),
        Some(serde_json::Value::String("Option+Z".to_owned()))
    );
}

#[test]
fn hydrated_provider_secrets_never_enter_settings_json() {
    let data_dir = unique_data_dir();
    let settings_file = data_dir.join("settings.json");
    let runtime = Runtime::new(data_dir.display().to_string()).expect("failed to create runtime");
    let secret = "sk-runtime-only-never-persist";
    let reference = "linguaray-secret://openai/apiKey";

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let settings = runtime.clone().settings();
            settings
                .update_provider(
                    "openai".to_owned(),
                    "openai".to_owned(),
                    HashMap::from([
                        ("apiKey".to_owned(), reference.to_owned()),
                        ("defaultModel".to_owned(), "gpt-4o-mini".to_owned()),
                    ]),
                )
                .await
                .expect("failed to add provider");
            settings
                .set_provider_secrets(
                    "openai".to_owned(),
                    HashMap::from([("apiKey".to_owned(), secret.to_owned())]),
                )
                .await
                .expect("failed to hydrate provider secret");

            let json = settings
                .get_json()
                .await
                .expect("failed to get settings json");
            assert!(!json.contains(secret));
            assert!(json.contains(reference));
        });

    let saved = std::fs::read_to_string(settings_file).expect("failed to read settings file");
    assert!(!saved.contains(secret));
    assert!(saved.contains(reference));
}

#[test]
fn provider_probe_does_not_mutate_settings() {
    let runtime = create_runtime();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let settings = runtime.clone().settings();
            let before = settings
                .get_json()
                .await
                .expect("failed to read settings before probe");

            let result = settings
                .test_provider(
                    "temporary-openai".to_owned(),
                    "openai".to_owned(),
                    HashMap::from([
                        ("apiKey".to_owned(), "temporary-secret".to_owned()),
                        ("defaultModel".to_owned(), String::new()),
                    ]),
                )
                .await;

            assert!(result.is_err(), "invalid temporary provider should fail");
            let after = settings
                .get_json()
                .await
                .expect("failed to read settings after probe");
            assert_eq!(after, before);
            assert!(settings
                .get_provider("temporary-openai".to_owned())
                .await
                .expect("failed to query provider")
                .is_none());
        });
}

#[test]
fn reset_shortcuts_persists_rust_defaults_to_settings_file() {
    let data_dir = unique_data_dir();
    let settings_file = data_dir.join("settings.json");
    let runtime = Runtime::new(data_dir.display().to_string()).expect("failed to create runtime");

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            runtime
                .clone()
                .settings()
                .update_shortcuts(ShortcutSettingsPatch {
                    toggle_mini_translator: Some("Command+Shift+Space".to_owned()),
                    extract_text_from_screen_selection: Some("Command+Shift+1".to_owned()),
                    extract_text_from_screen_capture: Some("Command+Shift+2".to_owned()),
                    extract_text_from_clipboard: Some("Command+Shift+3".to_owned()),
                    translate_input_content: Some("Command+Shift+4".to_owned()),
                })
                .await
                .expect("failed to update shortcuts");

            let reset = runtime
                .settings()
                .reset_shortcuts()
                .await
                .expect("failed to reset shortcuts");
            assert_eq!(reset, ShortcutSettings::default());
        });

    let saved = std::fs::read_to_string(settings_file).expect("failed to read settings file");
    let settings = serde_json::from_str::<Settings>(&saved).expect("failed to parse settings file");
    assert_eq!(settings.shortcuts, ShortcutSettings::default());
}

#[test]
fn service_provider_id_suffixes_are_accepted_for_compatibility() {
    assert_eq!(
        validate_service_provider_id("system+translation".to_owned(), "+translation").unwrap(),
        "system"
    );
    assert_eq!(
        validate_service_provider_id("system+dictionary".to_owned(), "+dictionary").unwrap(),
        "system"
    );
    assert_eq!(
        validate_service_provider_id("system+ocr".to_owned(), "+ocr").unwrap(),
        "system"
    );
    assert_eq!(
        validate_service_provider_id("system".to_owned(), "+translation").unwrap(),
        "system"
    );
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn system_ocr_recognizes_fixed_catalog_image() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../../apps/desktop/flutter/test/goldens/catalog/\
         workbench_success_light_macos.png",
    );
    assert!(fixture.is_file(), "OCR fixture is missing: {fixture:?}");

    let runtime = create_runtime();
    let response = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            runtime
                .ocr("system".to_owned())
                .expect("failed to get system OCR")
                .recognize_text(RecognizeTextRequest {
                    base64_image: None,
                    image_path: Some(fixture.display().to_string()),
                })
                .await
                .expect("system OCR failed")
        });

    assert!(
        response.text.to_lowercase().contains("stable"),
        "system OCR did not recognize the fixed English text: {:?}",
        response.text
    );
}

#[cfg(target_os = "macos")]
#[test]
fn system_dictionary_lookup_returns_structured_definitions() {
    let runtime = create_runtime();

    // Add the system provider explicitly (it is no longer auto-injected).
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            runtime
                .clone()
                .settings()
                .update_provider("system".to_owned(), "system".to_owned(), HashMap::new())
                .await
                .expect("failed to add system provider");
        });

    let response = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            runtime
                .clone()
                .dictionary("system".to_owned())
                .expect("failed to get dictionary")
                .lookup(LookUpRequest {
                    source_language: "en".to_owned(),
                    target_language: "zh".to_owned(),
                    word: "hello".to_owned(),
                })
                .await
                .expect("failed to look up hello")
        });

    let pronunciations = response.pronunciations.expect("pronunciations");
    assert_eq!(pronunciations.len(), 2);
    assert_eq!(pronunciations[0].r#type.as_deref(), Some("uk"));
    assert_eq!(pronunciations[1].r#type.as_deref(), Some("us"));

    let definitions = response.definitions.expect("definitions");
    assert!(
        definitions.iter().any(|definition| definition
            .values
            .as_ref()
            .map(|values| values.iter().any(|value| value.contains("问候")))
            .unwrap_or(false)),
        "expected parsed definitions to include the noun translation: {definitions:#?}"
    );
    assert!(
        definitions
            .iter()
            .flat_map(|definition| definition.values.as_deref().unwrap_or_default())
            .all(|value| !value.trim().is_empty()),
        "definitions should not contain empty values: {definitions:#?}"
    );
}

#[test]
fn translation_requires_target_language() {
    let runtime = create_runtime();
    let error = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            runtime
                .clone()
                .translation("deepl".to_owned())
                .unwrap()
                .translate(TranslateRequest {
                    source_language: Some("en".to_owned()),
                    target_language: Some(String::new()),
                    text: "hello".to_owned(),
                })
                .await
        })
        .unwrap_err();

    assert_eq!(error.to_string(), "target_language is required");
}

#[test]
fn runtime_new_returns_same_inner_for_same_data_dir() {
    let data_dir = unique_data_dir();
    let path = data_dir.display().to_string();

    let first = Runtime::new(path.clone()).expect("failed to create first runtime");
    let second = Runtime::new(path).expect("failed to create second runtime");

    assert!(
        Arc::ptr_eq(&first.inner, &second.inner),
        "Runtime::new should return a handle backed by the shared singleton inner"
    );
}

#[test]
fn runtime_new_returns_distinct_inner_for_different_data_dirs() {
    let first =
        Runtime::new(unique_data_dir().display().to_string()).expect("failed to create first");
    let second =
        Runtime::new(unique_data_dir().display().to_string()).expect("failed to create second");

    assert!(
        !Arc::ptr_eq(&first.inner, &second.inner),
        "different data dirs should produce independent runtimes"
    );
}

#[test]
fn subscribe_receives_change_for_each_section() {
    let runtime = create_runtime();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let settings = runtime.clone().settings();
            let subscription = settings.subscribe();

            settings
                .update_appearance(AppearanceSettingsPatch {
                    language: Some("en".to_owned()),
                    theme_mode: None,
                    theme: None,
                })
                .await
                .expect("update_appearance failed");
            assert_eq!(
                subscription.next().await.expect("recv failed"),
                Some(SettingsChange::Appearance)
            );

            settings
                .update_general(GeneralSettingsPatch {
                    launch_at_login: Some(true),
                    show_in_menu_bar: None,
                    default_ocr_service: None,
                    auto_copy_detected_text: None,
                    default_directory_service: None,
                    default_translation_service: None,
                    translation_targets: None,
                    input_submit_mode: None,
                    double_click_copy_result: None,
                    common_languages: None,
                })
                .await
                .expect("update_general failed");
            assert_eq!(
                subscription.next().await.expect("recv failed"),
                Some(SettingsChange::General)
            );

            settings
                .update_shortcuts(ShortcutSettingsPatch {
                    toggle_mini_translator: Some("Cmd+Space".to_owned()),
                    extract_text_from_screen_selection: None,
                    extract_text_from_screen_capture: None,
                    extract_text_from_clipboard: None,
                    translate_input_content: None,
                })
                .await
                .expect("update_shortcuts failed");
            assert_eq!(
                subscription.next().await.expect("recv failed"),
                Some(SettingsChange::Shortcuts)
            );
        });
}

#[test]
fn subscribe_fans_out_to_multiple_subscribers() {
    let runtime = create_runtime();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let settings = runtime.clone().settings();
            let sub_a = settings.subscribe();
            let sub_b = settings.subscribe();

            settings
                .update_appearance(AppearanceSettingsPatch {
                    language: Some("zh-Hans".to_owned()),
                    theme_mode: None,
                    theme: None,
                })
                .await
                .expect("update_appearance failed");

            assert_eq!(
                sub_a.next().await.expect("recv failed"),
                Some(SettingsChange::Appearance)
            );
            assert_eq!(
                sub_b.next().await.expect("recv failed"),
                Some(SettingsChange::Appearance)
            );
        });
}

#[test]
fn subscribe_observes_writes_from_other_handles() {
    // Mirrors the cross-binding scenario: writer and reader both
    // come from the same singleton; subscribing on one observes
    // writes performed on the other.
    let data_dir = unique_data_dir();
    let path = data_dir.display().to_string();
    let writer = Runtime::new(path.clone()).expect("failed to create writer runtime");
    let reader = Runtime::new(path).expect("failed to create reader runtime");

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let subscription = reader.clone().settings().subscribe();

            writer
                .clone()
                .settings()
                .update_appearance(AppearanceSettingsPatch {
                    language: Some("zh-Hans".to_owned()),
                    theme_mode: None,
                    theme: None,
                })
                .await
                .expect("writer update_appearance failed");

            assert_eq!(
                subscription.next().await.expect("recv failed"),
                Some(SettingsChange::Appearance)
            );
        });
}

#[test]
fn shared_runtime_observes_each_other_writes() {
    let data_dir = unique_data_dir();
    let path = data_dir.display().to_string();
    let writer = Runtime::new(path.clone()).expect("failed to create writer runtime");
    let reader = Runtime::new(path).expect("failed to create reader runtime");

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            writer
                .clone()
                .settings()
                .update_appearance(AppearanceSettingsPatch {
                    language: Some("zh-Hans".to_owned()),
                    theme_mode: None,
                    theme: None,
                })
                .await
                .expect("failed to update appearance via writer");

            let read_back = reader
                .clone()
                .settings()
                .get_appearance()
                .await
                .expect("failed to read appearance via reader");

            assert_eq!(read_back.language, "zh-Hans");
        });
}

#[test]
fn lookup_requires_word() {
    let runtime = create_runtime();

    // Add the system provider explicitly.
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            runtime
                .clone()
                .settings()
                .update_provider("system".to_owned(), "system".to_owned(), HashMap::new())
                .await
                .expect("failed to add system provider");
        });

    let error = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            runtime
                .clone()
                .dictionary("system".to_owned())
                .unwrap()
                .lookup(LookUpRequest {
                    source_language: "en".to_owned(),
                    target_language: "zh".to_owned(),
                    word: String::new(),
                })
                .await
        })
        .unwrap_err();

    assert_eq!(error.to_string(), "word is required");
}

fn block_on<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

fn sample_book() -> GlossaryBookInput {
    GlossaryBookInput {
        id: None,
        name: "机器学习".to_owned(),
        enabled: true,
        source_language: None,
        target_language: None,
    }
}

fn sample_entry(term: &str, translation: &str, forbidden: &[&str]) -> GlossaryEntryInput {
    GlossaryEntryInput {
        id: None,
        term: term.to_owned(),
        translation: translation.to_owned(),
        forbidden: forbidden.iter().map(|value| (*value).to_owned()).collect(),
        note: None,
        case_sensitive: false,
        whole_word: true,
    }
}

#[test]
fn glossary_survives_a_runtime_rebuilt_from_the_same_data_dir() {
    let data_dir = unique_data_dir();
    let book_id = block_on(async {
        let runtime =
            Runtime::new(data_dir.display().to_string()).expect("failed to create runtime");
        let glossary = runtime.glossary();
        let book = glossary
            .upsert_book(sample_book())
            .await
            .expect("failed to create book");
        glossary
            .upsert_entry(book.id.clone(), sample_entry("token", "词元", &["标记"]))
            .await
            .expect("failed to create entry");
        book.id
    });

    // A fresh handle for the same data dir shares the registry entry, so
    // reload through a second runtime process would look the same.
    let store =
        crate::domain::glossary::GlossaryStore::load(&data_dir).expect("failed to reload glossary");
    let entries = store
        .list_entries(&book_id, None, 0, 0)
        .expect("failed to list entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].translation, "词元");
}

#[test]
fn glossary_writes_notify_settings_subscribers() {
    let runtime = create_runtime();

    block_on(async {
        let subscription = runtime.clone().settings().subscribe();
        runtime
            .clone()
            .glossary()
            .upsert_book(sample_book())
            .await
            .expect("failed to create book");

        let change = subscription.next().await.expect("subscription ended");
        assert_eq!(change, Some(SettingsChange::Glossary));
    });
}

#[test]
fn glossary_check_flags_a_translation_that_ignores_its_terms() {
    let runtime = create_runtime();

    block_on(async {
        let glossary = runtime.clone().glossary();
        let book = glossary
            .upsert_book(sample_book())
            .await
            .expect("failed to create book");
        glossary
            .upsert_entry(book.id, sample_entry("token", "词元", &["标记"]))
            .await
            .expect("failed to create entry");

        let clean = glossary
            .check(
                "one token".to_owned(),
                "一个词元".to_owned(),
                Some("en".to_owned()),
                Some("zh".to_owned()),
            )
            .await
            .expect("failed to check translation");
        assert!(clean.is_empty());

        let issues = glossary
            .check(
                "one token".to_owned(),
                "一个标记".to_owned(),
                Some("en".to_owned()),
                Some("zh".to_owned()),
            )
            .await
            .expect("failed to check translation");
        let kinds: Vec<_> = issues.iter().map(|issue| issue.kind).collect();
        assert!(kinds.contains(&crate::domain::glossary::GlossaryIssueKind::MissingTranslation));
        assert!(kinds.contains(&crate::domain::glossary::GlossaryIssueKind::ForbiddenUsed));
    });
}

#[test]
fn prompt_template_substitutes_the_glossary_placeholder() {
    let terms = [GlossaryTerm {
        term: "token".to_owned(),
        translation: "词元".to_owned(),
        forbidden: Vec::new(),
    }];
    let rendered = render_prompt_template(
        "Translate to {{targetLanguage}}.\n{{glossary}}\nEnd.",
        "en",
        "zh",
        "token",
        &terms,
    );

    assert!(rendered.contains("Translate to zh."));
    assert!(rendered.contains("\"token\" MUST be translated as \"词元\""));
    assert!(rendered.trim_end().ends_with("End."));
}

#[test]
fn history_writes_notify_subscribers_and_round_trip() {
    let runtime = create_runtime();
    block_on(async {
        let subscription = runtime.clone().settings().subscribe();
        let entry = runtime
            .clone()
            .history()
            .upsert_entry(crate::domain::history::HistoryEntryInput {
                id: None,
                source: "hello".to_owned(),
                translation: "你好".to_owned(),
                source_language: "en".to_owned(),
                target_language: "zh-Hans".to_owned(),
                service_id: "system+translation".to_owned(),
                service_name: "System".to_owned(),
                edited: false,
            })
            .await
            .expect("failed to save history");
        assert_eq!(
            subscription.next().await.expect("history event"),
            Some(SettingsChange::History)
        );
        let entries = runtime
            .clone()
            .history()
            .list_entries(crate::domain::history::HistoryFilter::All, None)
            .await
            .expect("failed to list history");
        assert_eq!(entries, vec![entry]);
    });
}

#[test]
fn prompt_template_without_a_placeholder_still_gets_the_terms() {
    let terms = [GlossaryTerm {
        term: "token".to_owned(),
        translation: "词元".to_owned(),
        forbidden: Vec::new(),
    }];
    let rendered = render_prompt_template("Translate {{text}}.", "en", "zh", "token", &terms);

    assert!(rendered.starts_with("Translate token."));
    assert!(rendered.contains("Terminology constraints"));
}

#[test]
fn prompt_template_is_untouched_when_nothing_matched() {
    assert_eq!(
        render_prompt_template("Translate {{text}}.", "en", "zh", "hello", &[]),
        "Translate hello."
    );
}
