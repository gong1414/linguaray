//! Byte-safe SSE/NDJSON decoding shared by the three LLM transports.
use std::sync::mpsc;

use futures_util::StreamExt;
use linguaray_core::{LlmStreamReceiver, StreamChunk};
use serde_json::Value;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum WireFormat {
    OpenAi,
    #[cfg(any(feature = "anthropic", test))]
    Anthropic,
    #[cfg(any(feature = "ollama", test))]
    Ollama,
}

pub(super) fn receive(
    response: reqwest::Response,
    format: WireFormat,
    secrets: Vec<String>,
) -> LlmStreamReceiver {
    let (tx, rx) = mpsc::channel();
    tokio::spawn(async move {
        let mut stream = response.bytes_stream();
        let mut decoder = Decoder::new(format);
        while let Some(bytes) = stream.next().await {
            let result = bytes
                .map_err(|error| error.to_string())
                .and_then(|bytes| decoder.push(&bytes));
            match result {
                Ok(chunks) => {
                    for chunk in chunks {
                        if tx.send(chunk).is_err() {
                            return;
                        }
                    }
                    if decoder.finished {
                        return;
                    }
                }
                Err(error) => {
                    for chunk in decoder.recovered.drain(..) {
                        if tx.send(chunk).is_err() {
                            return;
                        }
                    }
                    send_error(&tx, &error, &secrets);
                    return;
                }
            }
        }
        match decoder.end() {
            Ok(chunks) => {
                for chunk in chunks {
                    if tx.send(chunk).is_err() {
                        return;
                    }
                }
                if !decoder.finished {
                    send_error(&tx, "stream ended before completion", &secrets);
                }
            }
            Err(error) => send_error(&tx, &error, &secrets),
        }
    });
    LlmStreamReceiver { rx }
}

fn send_error(tx: &mpsc::Sender<StreamChunk>, error: &str, secrets: &[String]) {
    let secrets: Vec<_> = secrets.iter().map(String::as_str).collect();
    let error = crate::catalog::urls::truncate_error_body(&crate::catalog::urls::redact_secrets(
        error, &secrets,
    ));
    let _ = tx.send(StreamChunk {
        content: error,
        index: 0,
        finish_reason: Some("error".into()),
    });
}

struct Decoder {
    format: WireFormat,
    bytes: Vec<u8>,
    data: String,
    event: String,
    #[cfg(any(feature = "anthropic", test))]
    stop_reason: String,
    finished: bool,
    recovered: Vec<StreamChunk>,
}

impl Decoder {
    fn new(format: WireFormat) -> Self {
        Self {
            format,
            bytes: Vec::new(),
            data: String::new(),
            event: String::new(),
            #[cfg(any(feature = "anthropic", test))]
            stop_reason: "stop".into(),
            finished: false,
            recovered: Vec::new(),
        }
    }

    fn push(&mut self, bytes: &[u8]) -> Result<Vec<StreamChunk>, String> {
        self.bytes.extend_from_slice(bytes);
        let mut chunks = Vec::new();
        while let Some(end) = self.bytes.iter().position(|b| *b == b'\n') {
            let line: Vec<_> = self.bytes.drain(..=end).collect();
            let line = std::str::from_utf8(&line).map_err(|_| "invalid UTF-8 stream".to_owned())?;
            if let Err(error) = self.line(line.trim_end_matches(['\n', '\r']), &mut chunks) {
                self.recovered = chunks;
                return Err(error);
            }
            if self.finished {
                break;
            }
        }
        if self.bytes.len() + self.data.len() > 1024 * 1024 {
            return Err("stream record exceeds size limit".into());
        }
        Ok(chunks)
    }

    fn end(&mut self) -> Result<Vec<StreamChunk>, String> {
        let bytes = std::mem::take(&mut self.bytes);
        let mut chunks = Vec::new();
        if !bytes.is_empty() {
            let line =
                std::str::from_utf8(&bytes).map_err(|_| "invalid UTF-8 stream".to_owned())?;
            self.line(line.trim_end_matches('\r'), &mut chunks)?;
        }
        if !self.data.is_empty() {
            self.dispatch(&mut chunks)?;
        }
        Ok(chunks)
    }

    fn line(&mut self, line: &str, chunks: &mut Vec<StreamChunk>) -> Result<(), String> {
        #[cfg(any(feature = "ollama", test))]
        if self.format == WireFormat::Ollama {
            if !line.trim().is_empty() {
                self.record(line, chunks)?;
            }
            return Ok(());
        }
        if line.is_empty() {
            self.dispatch(chunks)?;
        } else if let Some(data) = line.strip_prefix("data:") {
            if !self.data.is_empty() {
                self.data.push('\n');
            }
            self.data.push_str(data.strip_prefix(' ').unwrap_or(data));
        } else if let Some(event) = line.strip_prefix("event:") {
            self.event = event.trim().into();
        }
        Ok(())
    }

    fn dispatch(&mut self, chunks: &mut Vec<StreamChunk>) -> Result<(), String> {
        let data = std::mem::take(&mut self.data);
        if !data.is_empty() {
            self.record(&data, chunks)?;
        }
        self.event.clear();
        Ok(())
    }

    fn record(&mut self, data: &str, chunks: &mut Vec<StreamChunk>) -> Result<(), String> {
        if data.trim() == "[DONE]" {
            self.emit(String::new(), Some("stop".into()), chunks);
            return Ok(());
        }
        let value: Value =
            serde_json::from_str(data).map_err(|_| "invalid stream JSON".to_owned())?;
        if let Some(error) = value.get("error").filter(|v| !v.is_null()) {
            return Err(error
                .as_str()
                .or_else(|| error["message"].as_str())
                .unwrap_or("provider stream error")
                .into());
        }
        match self.format {
            WireFormat::OpenAi => {
                if let Some(choices) = value["choices"].as_array() {
                    for choice in choices {
                        if choice["index"].as_u64().unwrap_or(0) != 0 {
                            continue;
                        }
                        self.emit(
                            choice["delta"]["content"].as_str().unwrap_or("").into(),
                            choice["finish_reason"].as_str().map(str::to_owned),
                            chunks,
                        );
                    }
                }
            }
            #[cfg(any(feature = "anthropic", test))]
            WireFormat::Anthropic => match value["type"].as_str().unwrap_or(&self.event) {
                "content_block_delta" if value["delta"]["type"] == "text_delta" => {
                    self.emit(
                        value["delta"]["text"].as_str().unwrap_or("").into(),
                        None,
                        chunks,
                    );
                }
                "message_delta" => {
                    if let Some(reason) = value["delta"]["stop_reason"].as_str() {
                        self.stop_reason = match reason {
                            "end_turn" | "stop_sequence" => "stop",
                            "max_tokens" => "length",
                            other => other,
                        }
                        .into();
                    }
                }
                "message_stop" => self.emit(String::new(), Some(self.stop_reason.clone()), chunks),
                _ => {}
            },
            #[cfg(any(feature = "ollama", test))]
            WireFormat::Ollama => {
                let reason = value["done"]
                    .as_bool()
                    .filter(|done| *done)
                    .map(|_| value["done_reason"].as_str().unwrap_or("stop").into());
                self.emit(
                    value["message"]["content"].as_str().unwrap_or("").into(),
                    reason,
                    chunks,
                );
            }
        }
        Ok(())
    }

    fn emit(&mut self, content: String, reason: Option<String>, chunks: &mut Vec<StreamChunk>) {
        self.finished |= reason.is_some();
        if !content.is_empty() || reason.is_some() {
            chunks.push(StreamChunk {
                content,
                index: 0,
                finish_reason: reason,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_sse_and_terminal_content_survive_every_byte_boundary() {
        let payload = "data:{\"choices\":[{\"delta\":{\"content\":\"你好🙂\"},\"finish_reason\":\"stop\"}]}\r\n\r\n";
        let mut decoder = Decoder::new(WireFormat::OpenAi);
        let mut chunks = Vec::new();
        for byte in payload.as_bytes() {
            chunks.extend(decoder.push(&[*byte]).unwrap());
        }
        assert_eq!(chunks[0].content, "你好🙂");
        assert_eq!(chunks[0].finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn anthropic_preserves_length_stop_reason() {
        let mut decoder = Decoder::new(WireFormat::Anthropic);
        let chunks = decoder.push(b"event:message_delta\ndata:{\"delta\":{\"stop_reason\":\"max_tokens\"}}\n\nevent:message_stop\ndata:{}\n\n").unwrap();
        assert_eq!(chunks[0].finish_reason.as_deref(), Some("length"));
    }

    #[test]
    fn ollama_parses_final_record_without_newline() {
        let mut decoder = Decoder::new(WireFormat::Ollama);
        decoder
            .push("{\"message\":{\"content\":\"完\"},\"done\":true}".as_bytes())
            .unwrap();
        let chunks = decoder.end().unwrap();
        assert_eq!(chunks[0].content, "完");
        assert!(decoder.finished);
    }

    #[test]
    fn error_in_same_network_buffer_preserves_earlier_content() {
        let mut decoder = Decoder::new(WireFormat::OpenAi);
        let result = decoder.push(b"data:{\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\ndata:{\"error\":{\"message\":\"failed\"}}\n\n");
        assert!(result.is_err());
        assert_eq!(decoder.recovered[0].content, "partial");
    }

    #[test]
    fn malformed_and_error_records_fail_and_plain_eof_is_not_done() {
        assert!(Decoder::new(WireFormat::OpenAi)
            .push(b"data:{oops}\n\n")
            .is_err());
        assert!(Decoder::new(WireFormat::Ollama)
            .push(b"{\"error\":\"failed\"}\n")
            .is_err());
        let mut decoder = Decoder::new(WireFormat::OpenAi);
        decoder
            .push(b"data:{\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n")
            .unwrap();
        decoder.end().unwrap();
        assert!(!decoder.finished);
    }
}
