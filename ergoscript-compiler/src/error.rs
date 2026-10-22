use line_col::LineColLookup;
use rowan::TextRange;

pub fn pretty_error_desc(source: &str, span: TextRange, error_msg: &str) -> String {
    let line_col_lookup = LineColLookup::new(source);
    // `LineColLookup::get` takes a 0-based byte index and returns a 1-based
    // (line, col) pair. The span start is already a 0-based byte index, so it
    // is looked up as-is; the span end is exclusive, so its last byte is
    // looked up instead (for an empty span, the start itself). Subtracting 1
    // from the start underflowed for spans beginning at offset 0 (e.g. an
    // error on the first token of a script) and shifted every other span's
    // highlight one character to the left.
    let start: usize = span.start().into();
    let end: usize = span.end().into();
    let last = if end > start { end - 1 } else { start };
    let (line_start, col_start) = line_col_lookup.get(start);
    let (line_end, col_end) = line_col_lookup.get(last);
    if line_end != line_start {
        return "Multiline error spans are not yet supported".to_string();
    }
    // A span can point at the line following a trailing newline (e.g. an
    // error reported at end-of-input), which `lines()` does not yield; fall
    // back to an empty line instead of panicking.
    let source_line = source.lines().nth(line_start - 1).unwrap_or("");
    let highlight = format!("{0:^>span$}", "^", span = col_end - col_start + 1);
    format!(
        "{0}\nline: {1}\n{2}\n{3:>ident$}",
        error_msg,
        line_start,
        source_line,
        highlight,
        ident = col_start + 1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: u32, end: u32) -> TextRange {
        TextRange::new(start.into(), end.into())
    }

    #[test]
    fn span_at_start_of_source() {
        // Regression: a span starting at byte offset 0 used to panic with
        // "attempt to subtract with overflow".
        let desc = pretty_error_desc("foo", span(0, 3), "msg");
        assert_eq!(desc, "msg\nline: 1\nfoo\n^^^");
    }

    #[test]
    fn highlight_aligns_with_the_span() {
        // "true" occupies bytes 4..8; the carets must sit exactly under it.
        let desc = pretty_error_desc("1 + true", span(4, 8), "msg");
        assert_eq!(desc, "msg\nline: 1\n1 + true\n  ^^^^");
    }

    #[test]
    fn span_on_second_line() {
        let desc = pretty_error_desc("val x = 1\nfoo", span(10, 13), "msg");
        assert_eq!(desc, "msg\nline: 2\nfoo\n^^^");
    }

    #[test]
    fn empty_source_and_span() {
        let desc = pretty_error_desc("", span(0, 0), "msg");
        assert_eq!(desc, "msg\nline: 1\n\n ^");
    }

    #[test]
    fn span_at_end_of_input_after_trailing_newline() {
        // The position after a trailing newline is line 2 for the lookup,
        // but `lines()` yields no such line; this used to panic on unwrap.
        let desc = pretty_error_desc("{\n", span(2, 2), "msg");
        assert_eq!(desc, "msg\nline: 2\n\n ^");
    }
}
