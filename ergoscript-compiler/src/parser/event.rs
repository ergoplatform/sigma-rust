use crate::parser::ParseError;
use crate::syntax::SyntaxKind;

#[derive(Debug, PartialEq)]
pub enum Event {
    StartNode {
        kind: SyntaxKind,
        forward_parent: Option<usize>,
    },
    AddToken,
    FinishNode,
    Error(ParseError),
    Placeholder,
}
