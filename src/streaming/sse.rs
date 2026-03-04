use bytes::Bytes;

/// A parsed SSE event.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
    pub id: Option<String>,
}

/// Hand-rolled SSE parser. Processes a stream of bytes into SSE events.
///
/// SSE spec: lines separated by \n, fields are `event:`, `data:`, `id:`.
/// Events are delimited by blank lines.
pub struct SseParser {
    buffer: String,
    current_event: Option<String>,
    current_data: Vec<String>,
    current_id: Option<String>,
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            current_event: None,
            current_data: Vec::new(),
            current_id: None,
        }
    }

    /// Feed a chunk of bytes into the parser and return any complete events.
    pub fn feed(&mut self, chunk: &Bytes) -> Vec<SseEvent> {
        let text = String::from_utf8_lossy(chunk);
        self.buffer.push_str(&text);

        let mut events = Vec::new();

        // Process complete lines
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].trim_end_matches('\r').to_owned();
            self.buffer = self.buffer[newline_pos + 1..].to_owned();

            if line.is_empty() {
                // Blank line = event dispatch
                if !self.current_data.is_empty() {
                    events.push(SseEvent {
                        event: self.current_event.take(),
                        data: self.current_data.join("\n"),
                        id: self.current_id.take(),
                    });
                    self.current_data.clear();
                }
                continue;
            }

            if line.starts_with(':') {
                // Comment line, ignore
                continue;
            }

            let (field, value) = if let Some(colon_pos) = line.find(':') {
                let field = &line[..colon_pos];
                let value = line[colon_pos + 1..].strip_prefix(' ').unwrap_or(&line[colon_pos + 1..]);
                (field, value)
            } else {
                (line.as_str(), "")
            };

            match field {
                "event" => self.current_event = Some(value.to_owned()),
                "data" => self.current_data.push(value.to_owned()),
                "id" => self.current_id = Some(value.to_owned()),
                "retry" => {} // Ignored for SDK use
                _ => {}       // Unknown fields ignored per spec
            }
        }

        events
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_event() {
        let mut parser = SseParser::new();
        let events = parser.feed(&Bytes::from("event: message_start\ndata: {\"type\":\"message_start\"}\n\n"));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("message_start"));
        assert_eq!(events[0].data, "{\"type\":\"message_start\"}");
    }

    #[test]
    fn parse_multi_line_data() {
        let mut parser = SseParser::new();
        let events = parser.feed(&Bytes::from("data: line1\ndata: line2\n\n"));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line1\nline2");
    }

    #[test]
    fn parse_chunked_delivery() {
        let mut parser = SseParser::new();
        let events1 = parser.feed(&Bytes::from("event: ping\nda"));
        assert!(events1.is_empty());
        let events2 = parser.feed(&Bytes::from("ta: {}\n\n"));
        assert_eq!(events2.len(), 1);
        assert_eq!(events2[0].event.as_deref(), Some("ping"));
    }

    #[test]
    fn parse_comment_lines() {
        let mut parser = SseParser::new();
        let events = parser.feed(&Bytes::from(": this is a comment\ndata: hello\n\n"));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn no_event_without_blank_line() {
        let mut parser = SseParser::new();
        let events = parser.feed(&Bytes::from("data: incomplete\n"));
        assert!(events.is_empty());
    }
}
