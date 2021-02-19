use line_col::LineColLookup;
use rowan::TextRange;
use source_span::fmt::Formatter;
use source_span::fmt::Style;
use source_span::DefaultMetrics;
use source_span::Position;
use source_span::SourceBuffer;

use crate::compiler::CompileError;

// pub struct SourceContext<'a> {
//     line: u32,
//     col_start: usize,
//     col_end: usize,
//     source_line: &'a str,
// }

// impl<'a> SourceContext<'a> {
//     pub fn from_source(source: &str, span: TextRange) -> Self {
//     let lines: Vec<&str> = source.lines().collect();
//     if lines.is_empty() {
//       SourceContext{ line: 0, col_start: 0, col_end: 0, source_line: "" }
//     }
//     else {
//       lines.iter()
//           .skip(1)
//         // .scanLeft((0, lines.head.length)) { case ((_, end), line) => (end + 1, end + 1 + line.length) }
//         .zip(lines)
//         .enumerate()
//         .find (|(((start, end), _), _)|  =>  index >= start && index <= end )
//         .map {
//           case (((start, _), line), lineIndex) =>
//             SourceContext(lineIndex + 1, index - start + 1, line)
//         }.getOrElse {
//           // at least one line in the input
//           // point to the last character of the last line
//           val lastLine = lines.last
//           val iLine = lines.length - 1
//           val iCol = lastLine.length - 1
//           SourceContext(iLine, iCol, lastLine)
//         }
//     }

// }}

pub fn pretty_error_desc(source: &str, span: TextRange, error_msg: &str) -> String {
    // let ctx = SourceContext::from_source(source, span);
    // format!(
    //     "error: {0}\nline {1}: {2}\n{:>ident$}{:^>span$}",
    //     error_msg,
    //     ctx.line,
    //     ctx.source_line,
    //     ident = ctx.col_start,
    //     span = ctx.col_start - ctx.col_end
    // )

    // TODO: fix
    let chars = source.chars().map(Result::<char, CompileError>::Ok);
    let metrics = DefaultMetrics::with_tab_stop(2);

    let buffer = SourceBuffer::new(chars, Position::default(), metrics);
    let mut fmt = Formatter::new();
    let line_col_lookup = LineColLookup::new(source);
    let start_zero_based: usize = usize::from(span.start()) - 1;
    let end_zero_based: usize = usize::from(span.end()) - 1;
    let (line_start, col_start) = line_col_lookup.get(start_zero_based);
    let (line_end, col_end) = line_col_lookup.get(end_zero_based);
    let pos_end = Position::new(line_end - 1, col_end - 1);
    let span = source_span::Span::new(
        Position::new(line_start - 1, col_start - 1),
        pos_end,
        pos_end.next_column(),
    );
    fmt.add(span, Some(error_msg.to_string()), Style::Error);
    let formatted = fmt.render(buffer.iter(), span.aligned(), &metrics).unwrap();
    formatted.to_string()
}
