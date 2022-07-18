use rowan::GreenNode;

use crate::lexer::Lexer;
use crate::syntax::SyntaxNode;

use super::parse_error::ParseError;
use super::sink::Sink;
use super::source::Source;
use super::Parser;

use std::fmt::Write;

// Initial version is copied from https://github.com/arzg/eldiro
// Checkout https://arzg.github.io/lang/ for description

pub fn parse(input: &str) -> Parse {
    let tokens: Vec<_> = Lexer::new(input).collect();
    let source = Source::new(&tokens);
    let parser = Parser::new(source);
    let events = parser.parse();
    let sink = Sink::new(&tokens, events);

    sink.finish()
}

pub struct Parse {
    pub green_node: GreenNode,
    pub errors: Vec<ParseError>,
}

impl Parse {
    pub fn debug_tree(&self) -> String {
        let mut s = String::new();

        let tree = format!("{:#?}", self.syntax());

        // We cut off the last byte because formatting the SyntaxNode adds on a newline at the end.
        s.push_str(&tree[0..tree.len() - 1]);

        for error in &self.errors {
            write!(&mut s, "\n{}", error).unwrap();
        }

        s
    }

    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }
}
