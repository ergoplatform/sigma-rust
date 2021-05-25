use line_col::LineColLookup;
use rowan::TextRange;

pub fn pretty_error_desc(source: &str, span: TextRange, error_msg: &str) -> String {
    let line_col_lookup = LineColLookup::new(source);
    let start_zero_based: usize = usize::from(span.start()) - 1;
    let end_zero_based: usize = usize::from(span.end()) - 1;
    let (line_start, col_start) = line_col_lookup.get(start_zero_based);
    let (line_end, col_end) = line_col_lookup.get(end_zero_based);
    if line_end != line_start {
        return "Multiline error spans are not yet supported".to_string();
    }
    let source_line = source.lines().nth(line_start - 1).unwrap();
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
