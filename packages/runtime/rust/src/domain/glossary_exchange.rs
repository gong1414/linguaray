//! CSV and TBX interchange for glossary books.

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::glossary::{GlossaryBook, GlossaryEntry, GlossaryEntryInput};

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GlossaryExchangeFormat {
    Csv,
    Tbx,
}

pub fn encode(
    book: &GlossaryBook,
    entries: &[GlossaryEntry],
    format: GlossaryExchangeFormat,
) -> Result<String, String> {
    match format {
        GlossaryExchangeFormat::Csv => encode_csv(entries),
        GlossaryExchangeFormat::Tbx => Ok(encode_tbx(book, entries)),
    }
}

pub fn decode(
    content: &str,
    book: &GlossaryBook,
    format: GlossaryExchangeFormat,
) -> Result<Vec<GlossaryEntryInput>, String> {
    match format {
        GlossaryExchangeFormat::Csv => decode_csv(content),
        GlossaryExchangeFormat::Tbx => decode_tbx(content, book),
    }
}

fn encode_csv(entries: &[GlossaryEntry]) -> Result<String, String> {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    writer
        .write_record([
            "term",
            "translation",
            "forbidden",
            "note",
            "case_sensitive",
            "whole_word",
        ])
        .map_err(|error| format!("failed to write CSV header: {error}"))?;
    for entry in entries {
        writer
            .write_record([
                entry.term.as_str(),
                entry.translation.as_str(),
                &entry.forbidden.join("|"),
                entry.note.as_deref().unwrap_or_default(),
                if entry.case_sensitive { "true" } else { "false" },
                if entry.whole_word { "true" } else { "false" },
            ])
            .map_err(|error| format!("failed to write CSV row: {error}"))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|error| format!("failed to finish CSV: {error}"))?;
    String::from_utf8(bytes).map_err(|error| format!("CSV is not UTF-8: {error}"))
}

fn decode_csv(content: &str) -> Result<Vec<GlossaryEntryInput>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());
    let headers = reader
        .headers()
        .map_err(|error| format!("failed to read CSV header: {error}"))?
        .clone();
    let column = |aliases: &[&str]| {
        headers.iter().position(|header| {
            aliases
                .iter()
                .any(|alias| header.trim().eq_ignore_ascii_case(alias))
        })
    };
    let term = column(&["term", "source", "source_text", "original"])
        .ok_or_else(|| "CSV needs a term or source column".to_owned())?;
    let translation = column(&["translation", "target", "target_text", "translated"])
        .ok_or_else(|| "CSV needs a translation or target column".to_owned())?;
    let forbidden = column(&["forbidden", "forbidden_terms", "alternatives"]);
    let note = column(&["note", "notes", "comment"]);
    let case_sensitive = column(&["case_sensitive", "caseSensitive"]);
    let whole_word = column(&["whole_word", "wholeWord"]);

    let mut entries = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|error| format!("failed to read CSV row: {error}"))?;
        entries.push(GlossaryEntryInput {
            id: None,
            term: row.get(term).unwrap_or_default().trim().to_owned(),
            translation: row
                .get(translation)
                .unwrap_or_default()
                .trim()
                .to_owned(),
            forbidden: forbidden
                .and_then(|index| row.get(index))
                .unwrap_or_default()
                .split(['|', ';'])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
            note: note
                .and_then(|index| row.get(index))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            case_sensitive: case_sensitive
                .and_then(|index| row.get(index))
                .is_some_and(parse_bool),
            whole_word: whole_word
                .and_then(|index| row.get(index))
                .map(parse_bool)
                .unwrap_or(true),
        });
    }
    Ok(entries)
}

fn encode_tbx(book: &GlossaryBook, entries: &[GlossaryEntry]) -> String {
    let source_language = book.source_language.as_deref().unwrap_or("und");
    let target_language = book.target_language.as_deref().unwrap_or("und");
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<tbx type=\"TBX-Basic\" style=\"dca\">\n<text><body>\n",
    );
    for entry in entries {
        xml.push_str(&format!("<termEntry id=\"{}\">\n", xml_escape(&entry.id)));
        xml.push_str(&format!(
            "<langSet xml:lang=\"{}\"><tig><term>{}</term></tig></langSet>\n",
            xml_escape(source_language),
            xml_escape(&entry.term)
        ));
        xml.push_str(&format!(
            "<langSet xml:lang=\"{}\"><tig><term>{}</term>",
            xml_escape(target_language),
            xml_escape(&entry.translation)
        ));
        for forbidden in &entry.forbidden {
            xml.push_str(&format!(
                "<termNote type=\"linguaray:forbidden\">{}</termNote>",
                xml_escape(forbidden)
            ));
        }
        xml.push_str("</tig></langSet>\n");
        if let Some(note) = &entry.note {
            xml.push_str(&format!(
                "<descrip type=\"linguaray:note\">{}</descrip>\n",
                xml_escape(note)
            ));
        }
        xml.push_str(&format!(
            "<descrip type=\"linguaray:case-sensitive\">{}</descrip>\n",
            entry.case_sensitive
        ));
        xml.push_str(&format!(
            "<descrip type=\"linguaray:whole-word\">{}</descrip>\n",
            entry.whole_word
        ));
        xml.push_str("</termEntry>\n");
    }
    xml.push_str("</body></text>\n</tbx>\n");
    xml
}

#[derive(Default)]
struct TbxEntry {
    terms: Vec<(Option<String>, String)>,
    forbidden: Vec<String>,
    note: Option<String>,
    case_sensitive: bool,
    whole_word: Option<bool>,
}

fn decode_tbx(content: &str, book: &GlossaryBook) -> Result<Vec<GlossaryEntryInput>, String> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut entries = Vec::new();
    let mut pending: Option<TbxEntry> = None;
    let mut language: Option<String> = None;
    let mut capture: Option<String> = None;
    let mut text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => match element.name().as_ref() {
                b"termEntry" => pending = Some(TbxEntry::default()),
                b"langSet" => language = attribute(&element, b"xml:lang"),
                b"term" => begin_capture(&mut capture, &mut text, "term"),
                b"termNote"
                    if attribute(&element, b"type").as_deref()
                        == Some("linguaray:forbidden") =>
                {
                    begin_capture(&mut capture, &mut text, "forbidden")
                }
                b"descrip" => {
                    if let Some(kind) = attribute(&element, b"type") {
                        begin_capture(&mut capture, &mut text, &kind);
                    }
                }
                _ => {}
            },
            Ok(Event::Text(value)) => {
                if capture.is_some() {
                    let raw = String::from_utf8_lossy(value.as_ref());
                    text.push_str(
                        &quick_xml::escape::unescape(&raw)
                            .map_err(|error| format!("invalid TBX text: {error}"))?,
                    );
                }
            }
            Ok(Event::CData(value)) => {
                if capture.is_some() {
                    text.push_str(&String::from_utf8_lossy(value.as_ref()));
                }
            }
            Ok(Event::End(element)) => match element.name().as_ref() {
                b"term" => {
                    if let Some(entry) = &mut pending {
                        let value = text.trim();
                        if !value.is_empty() {
                            entry.terms.push((language.clone(), value.to_owned()));
                        }
                    }
                    capture = None;
                }
                b"termNote" | b"descrip" => {
                    if let (Some(entry), Some(kind)) = (&mut pending, capture.take()) {
                        let value = text.trim();
                        match kind.as_str() {
                            "forbidden" if !value.is_empty() => {
                                entry.forbidden.push(value.to_owned());
                            }
                            "linguaray:note" if !value.is_empty() => {
                                entry.note = Some(value.to_owned());
                            }
                            "linguaray:case-sensitive" => {
                                entry.case_sensitive = parse_bool(value);
                            }
                            "linguaray:whole-word" => entry.whole_word = Some(parse_bool(value)),
                            _ => {}
                        }
                    }
                }
                b"langSet" => language = None,
                b"termEntry" => {
                    if let Some(entry) = pending.take() {
                        if let Some(input) = finish_tbx_entry(entry, book) {
                            entries.push(input);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(error) => return Err(format!("failed to parse TBX: {error}")),
            _ => {}
        }
    }
    Ok(entries)
}

fn finish_tbx_entry(entry: TbxEntry, book: &GlossaryBook) -> Option<GlossaryEntryInput> {
    let source_index = language_term_index(&entry.terms, book.source_language.as_deref(), None)
        .unwrap_or(0);
    let target_index = language_term_index(
        &entry.terms,
        book.target_language.as_deref(),
        Some(source_index),
    )
    .or_else(|| (0..entry.terms.len()).find(|index| *index != source_index))?;
    Some(GlossaryEntryInput {
        id: None,
        term: entry.terms.get(source_index)?.1.clone(),
        translation: entry.terms.get(target_index)?.1.clone(),
        forbidden: entry.forbidden,
        note: entry.note,
        case_sensitive: entry.case_sensitive,
        whole_word: entry.whole_word.unwrap_or(true),
    })
}

fn language_term_index(
    terms: &[(Option<String>, String)],
    expected: Option<&str>,
    excluded: Option<usize>,
) -> Option<usize> {
    let expected = expected?.split(['-', '_']).next()?;
    terms.iter().enumerate().position(|(index, (language, _))| {
        Some(index) != excluded
            && language.as_deref().is_some_and(|language| {
                language
                    .split(['-', '_'])
                    .next()
                    .is_some_and(|value| value.eq_ignore_ascii_case(expected))
            })
    })
}

fn begin_capture(capture: &mut Option<String>, text: &mut String, kind: &str) {
    *capture = Some(kind.to_owned());
    text.clear();
}

fn attribute(element: &BytesStart<'_>, key: &[u8]) -> Option<String> {
    element
        .attributes()
        .with_checks(false)
        .flatten()
        .find(|attribute| attribute.key.as_ref() == key)
        .and_then(|attribute| {
            let raw = String::from_utf8_lossy(attribute.value.as_ref());
            quick_xml::escape::unescape(&raw)
                .ok()
                .map(|value| value.into_owned())
        })
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn xml_escape(value: &str) -> String {
    quick_xml::escape::escape(value).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> GlossaryBook {
        GlossaryBook {
            id: "book-1".to_owned(),
            name: "Machine learning".to_owned(),
            enabled: true,
            source_language: Some("en".to_owned()),
            target_language: Some("zh-Hans".to_owned()),
            entry_count: 1,
            created_at: 1,
            updated_at: 1,
        }
    }

    fn entry() -> GlossaryEntry {
        GlossaryEntry {
            id: "term-1".to_owned(),
            term: "teacher forcing".to_owned(),
            translation: "教师强制".to_owned(),
            forbidden: vec!["强制教学".to_owned()],
            note: Some("training strategy".to_owned()),
            case_sensitive: false,
            whole_word: true,
            hits: 0,
            created_at: 1,
            updated_at: 1,
        }
    }

    #[test]
    fn csv_round_trip_preserves_rules() {
        let content = encode(&book(), &[entry()], GlossaryExchangeFormat::Csv).unwrap();
        let decoded = decode(&content, &book(), GlossaryExchangeFormat::Csv).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].term, "teacher forcing");
        assert_eq!(decoded[0].translation, "教师强制");
        assert_eq!(decoded[0].forbidden, ["强制教学"]);
        assert_eq!(decoded[0].note.as_deref(), Some("training strategy"));
        assert!(decoded[0].whole_word);
    }

    #[test]
    fn tbx_round_trip_uses_book_languages() {
        let content = encode(&book(), &[entry()], GlossaryExchangeFormat::Tbx).unwrap();
        let decoded = decode(&content, &book(), GlossaryExchangeFormat::Tbx).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].term, "teacher forcing");
        assert_eq!(decoded[0].translation, "教师强制");
        assert_eq!(decoded[0].forbidden, ["强制教学"]);
    }
}
