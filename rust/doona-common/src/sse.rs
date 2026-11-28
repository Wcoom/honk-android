//! Pure-Rust SSE (Server-Sent Events) parser for `/api/v1/logs` and `/api/v1/events`.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SseFrame {
    pub id: Option<String>,
    pub event: String,
    pub data: String,
}

/// Incremental SSE frame parser handling `\r\n` and `\n` line endings across arbitrary chunk boundaries.
#[derive(Default)]
pub struct SseParser {
    buffer: String,
    cur_id: Option<String>,
    cur_event: Option<String>,
    cur_data: Vec<String>,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: &str) -> Vec<SseFrame> {
        self.buffer.push_str(chunk);
        let mut frames = Vec::new();
        while let Some(pos) = self.buffer.find('\n') {
            let mut line = self.buffer[..pos].to_string();
            self.buffer.drain(..=pos);
            if line.ends_with('\r') {
                line.pop();
            }
            if line.is_empty() {
                if !self.cur_data.is_empty() || self.cur_event.is_some() {
                    frames.push(SseFrame {
                        id: self.cur_id.take(),
                        event: self
                            .cur_event
                            .take()
                            .unwrap_or_else(|| "message".to_string()),
                        data: self.cur_data.join("\n"),
                    });
                    self.cur_data.clear();
                } else {
                    self.cur_id = None;
                }
                continue;
            }
            if line.starts_with(':') {
                continue;
            }
            let (field, raw_val) = match line.find(':') {
                Some(idx) => {
                    let rest = &line[idx + 1..];
                    let val = rest.strip_prefix(' ').unwrap_or(rest);
                    (&line[..idx], val)
                }
                None => (line.as_str(), ""),
            };
            match field {
                "id" => self.cur_id = Some(raw_val.to_string()),
                "event" => self.cur_event = Some(raw_val.to_string()),
                "data" => self.cur_data.push(raw_val.to_string()),
                _ => {}
            }
        }
        frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_split_sse_chunks_and_comments() {
        let mut parser = SseParser::new();
        assert!(parser.push(": heartbeat\n\n").is_empty());
        let first = parser.push("id: 42\nevent: log\ndata: {\"level\":\"info\"");
        assert!(first.is_empty());
        let second = parser.push("}\n\nid: 43\nevent: runtime.updated\ndata: {}\r\n\r\n");
        assert_eq!(second.len(), 2);
        assert_eq!(second[0].id.as_deref(), Some("42"));
        assert_eq!(second[0].event, "log");
        assert_eq!(second[0].data, "{\"level\":\"info\"}");
        assert_eq!(second[1].id.as_deref(), Some("43"));
        assert_eq!(second[1].event, "runtime.updated");
    }
}
