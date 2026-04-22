use crate::error::pretty_error_desc;

use super::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
use text_size::TextRange;

#[derive(Debug, PartialEq, Eq)]
pub struct AstError {
    pub msg: String,
    pub span: TextRange,
}

impl AstError {
    pub fn new(msg: String, span: TextRange) -> Self {
        AstError { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

#[derive(Debug)]
pub struct Root(SyntaxNode);

impl Root {
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Root {
            Some(Self(node))
        } else {
            None
        }
    }

    pub fn children(&self) -> impl Iterator<Item = Expr> {
        self.0.children().filter_map(Expr::cast)
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct Ident(SyntaxNode);

impl Ident {
    pub fn name(&self) -> Result<SyntaxToken, AstError> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::Ident)
            .ok_or_else(|| AstError::new(format!("Empty Ident.name in: {:?}", self.0), self.span()))
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Expr {
    Ident(Ident),
    BinaryExpr(BinaryExpr),
    Literal(Literal),
    BoolLiteral(BoolLiteral),
    StringLiteral(StringLiteral),
    FuncCall(FuncCall),
    Block(BlockExpr),
    VariableDef(VariableDef),
    FieldAccess(FieldAccess),
    Lambda(Lambda),
    IfExpr(IfExpr),
    TupleExpr(TupleExpr),
    PrefixExpr(PrefixExpr),
}

impl Expr {
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        let result = match node.kind() {
            SyntaxKind::Ident => Self::Ident(Ident(node)),
            SyntaxKind::InfixExpr => Self::BinaryExpr(BinaryExpr(node)),
            SyntaxKind::IntNumber => Self::Literal(Literal(node)),
            SyntaxKind::LongNumber => Self::Literal(Literal(node)),
            SyntaxKind::BoolLiteral => Self::BoolLiteral(BoolLiteral(node)),
            SyntaxKind::StringLiteral => Self::StringLiteral(StringLiteral(node)),
            SyntaxKind::FuncCall => Self::FuncCall(FuncCall(node)),
            SyntaxKind::BlockExpr => Self::Block(BlockExpr(node)),
            SyntaxKind::VariableDef => Self::VariableDef(VariableDef(node)),
            SyntaxKind::FieldAccess => Self::FieldAccess(FieldAccess(node)),
            SyntaxKind::Lambda => Self::Lambda(Lambda(node)),
            SyntaxKind::IfExpr => Self::IfExpr(IfExpr(node)),
            SyntaxKind::TupleExpr => Self::TupleExpr(TupleExpr(node)),
            SyntaxKind::PrefixExpr => Self::PrefixExpr(PrefixExpr(node)),
            SyntaxKind::ParenExpr => {
                // Transparently unwrap parenthesized expressions
                return node.children().find_map(Expr::cast);
            }
            _ => return None,
        };

        Some(result)
    }
}

#[derive(Debug)]
pub struct BinaryExpr(SyntaxNode);

impl BinaryExpr {
    pub fn lhs(&self) -> Result<Expr, AstError> {
        self.0.children().find_map(Expr::cast).ok_or_else(|| {
            AstError::new(
                format!("Cannot find lhs in {:?}", self.0.children()),
                self.0.text_range(),
            )
        })
    }

    pub fn rhs(&self) -> Result<Expr, AstError> {
        self.0
            .children()
            .filter_map(Expr::cast)
            .nth(1)
            .ok_or_else(|| {
                AstError::new(
                    format!("Cannot find rhs in {:?}", self.0.children()),
                    self.0.text_range(),
                )
            })
    }

    pub fn op(&self) -> Result<SyntaxToken, AstError> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| {
                matches!(
                    token.kind(),
                    SyntaxKind::Plus
                        | SyntaxKind::Minus
                        | SyntaxKind::Star
                        | SyntaxKind::Slash
                        | SyntaxKind::Percent
                        | SyntaxKind::And
                        | SyntaxKind::Or
                        | SyntaxKind::EqEq
                        | SyntaxKind::NotEq
                        | SyntaxKind::Gt
                        | SyntaxKind::Lt
                        | SyntaxKind::GtEq
                        | SyntaxKind::LtEq,
                )
            })
            .ok_or_else(|| {
                AstError::new(
                    format!("Cannot find bin op in {:?}", self.0),
                    self.0.text_range(),
                )
            })
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub enum LiteralValue {
    Int(i32),
    Long(i64),
}

#[derive(Debug)]
pub struct Literal(SyntaxNode);

impl Literal {
    pub fn parse(&self) -> Result<LiteralValue, AstError> {
        let text = self.0.first_token().unwrap().text().to_string();
        if text.ends_with('L') {
            text.strip_suffix('L')
                .unwrap()
                .parse()
                .ok()
                .map(LiteralValue::Long)
        } else {
            text.parse().ok().map(LiteralValue::Int)
        }
        .ok_or_else(|| {
            AstError::new(
                format!("Failed to parse Literal from: {:?}", self.0),
                self.span(),
            )
        })
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct BoolLiteral(SyntaxNode);

impl BoolLiteral {
    pub fn value(&self) -> Result<bool, AstError> {
        let token = self.0.first_token().ok_or_else(|| {
            AstError::new(format!("Empty BoolLiteral: {:?}", self.0), self.span())
        })?;
        match token.kind() {
            SyntaxKind::TrueKw => Ok(true),
            SyntaxKind::FalseKw => Ok(false),
            _ => Err(AstError::new(
                format!("Unexpected token in BoolLiteral: {:?}", token),
                self.span(),
            )),
        }
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct StringLiteral(SyntaxNode);

impl StringLiteral {
    pub fn value(&self) -> Result<String, AstError> {
        let text = self
            .0
            .first_token()
            .ok_or_else(|| {
                AstError::new(format!("Empty StringLiteral: {:?}", self.0), self.span())
            })?
            .text()
            .to_string();
        // Strip surrounding quotes
        Ok(text.trim_matches('"').to_string())
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct FuncCall(SyntaxNode);

impl FuncCall {
    pub fn func_expr(&self) -> Result<Expr, AstError> {
        // The first child Expr is the function/callee
        self.0.children().find_map(Expr::cast).ok_or_else(|| {
            AstError::new(
                format!("Cannot find func expr in FuncCall: {:?}", self.0),
                self.span(),
            )
        })
    }

    pub fn args(&self) -> Vec<Expr> {
        // Args are all Expr children after the first one (the callee)
        self.0.children().filter_map(Expr::cast).skip(1).collect()
    }

    /// Returns type arg string from `[Type]` brackets, e.g. "Byte" from `Coll[Byte](...)`
    pub fn type_arg_string(&self) -> Option<String> {
        let mut depth = 0;
        let mut result = String::new();
        for element in self.0.children_with_tokens() {
            if let Some(token) = element.into_token() {
                match token.kind() {
                    SyntaxKind::LBracket if depth == 0 => depth = 1,
                    SyntaxKind::LBracket => {
                        depth += 1;
                        result.push_str(token.text());
                    }
                    SyntaxKind::RBracket => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        result.push_str(token.text());
                    }
                    _ if depth > 0 => {
                        if !matches!(token.kind(), SyntaxKind::Whitespace | SyntaxKind::Comment) {
                            result.push_str(token.text());
                        }
                    }
                    _ => {}
                }
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct VariableDef(SyntaxNode);

impl VariableDef {
    pub fn name(&self) -> Result<String, AstError> {
        // First Ident token is the variable name
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::Ident)
            .map(|t| t.text().to_string())
            .ok_or_else(|| {
                AstError::new(
                    format!("Cannot find name in VariableDef: {:?}", self.0),
                    self.span(),
                )
            })
    }

    pub fn type_annotation(&self) -> Option<String> {
        // After colon, reconstruct the full type string until '='
        let mut found_colon = false;
        let mut type_str = String::new();
        let mut depth = 0;
        for element in self.0.children_with_tokens() {
            if let Some(token) = element.into_token() {
                if token.kind() == SyntaxKind::Colon && !found_colon {
                    found_colon = true;
                    continue;
                }
                if found_colon {
                    if token.kind() == SyntaxKind::Equals && depth == 0 {
                        break;
                    }
                    if matches!(token.kind(), SyntaxKind::LParen | SyntaxKind::LBracket) {
                        depth += 1;
                    }
                    if matches!(token.kind(), SyntaxKind::RParen | SyntaxKind::RBracket) {
                        depth -= 1;
                    }
                    if !matches!(token.kind(), SyntaxKind::Whitespace | SyntaxKind::Comment) {
                        type_str.push_str(token.text());
                    }
                }
            }
        }
        if type_str.is_empty() {
            None
        } else {
            Some(type_str)
        }
    }

    pub fn value(&self) -> Result<Expr, AstError> {
        // The value is the first child Expr node
        self.0.children().find_map(Expr::cast).ok_or_else(|| {
            AstError::new(
                format!("Cannot find value in VariableDef: {:?}", self.0),
                self.span(),
            )
        })
    }

    /// Returns true if this VariableDef came from a `def` (function definition).
    pub fn is_def(&self) -> bool {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .any(|t| t.kind() == SyntaxKind::FnKw)
    }

    /// For `def` definitions, extract parameter names and types.
    /// Returns Vec of (name, optional_type_string).
    pub fn def_params(&self) -> Vec<(String, Option<String>)> {
        let mut params = Vec::new();
        let mut in_parens = false;
        let mut current_name: Option<String> = None;
        let mut current_type = String::new();
        let mut found_colon = false;

        for element in self.0.children_with_tokens() {
            if let Some(token) = element.into_token() {
                match token.kind() {
                    SyntaxKind::LParen => {
                        in_parens = true;
                    }
                    SyntaxKind::RParen => {
                        // Push last param if any
                        if let Some(name) = current_name.take() {
                            let tpe = if current_type.is_empty() {
                                None
                            } else {
                                Some(current_type.clone())
                            };
                            params.push((name, tpe));
                        }
                        break;
                    }
                    SyntaxKind::Comma if in_parens => {
                        if let Some(name) = current_name.take() {
                            let tpe = if current_type.is_empty() {
                                None
                            } else {
                                Some(current_type.clone())
                            };
                            params.push((name, tpe));
                        }
                        current_type.clear();
                        found_colon = false;
                    }
                    SyntaxKind::Colon if in_parens && current_name.is_some() => {
                        found_colon = true;
                        current_type.clear();
                    }
                    SyntaxKind::Ident if in_parens => {
                        if found_colon {
                            current_type.push_str(token.text());
                        } else if current_name.is_none() {
                            current_name = Some(token.text().to_string());
                        }
                    }
                    SyntaxKind::LBracket | SyntaxKind::RBracket if in_parens && found_colon => {
                        current_type.push_str(token.text());
                    }
                    _ => {}
                }
            }
        }
        params
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct BlockExpr(SyntaxNode);

impl BlockExpr {
    pub fn exprs(&self) -> Vec<Expr> {
        self.0.children().filter_map(Expr::cast).collect()
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct FieldAccess(SyntaxNode);

impl FieldAccess {
    pub fn object(&self) -> Result<Expr, AstError> {
        self.0.children().find_map(Expr::cast).ok_or_else(|| {
            AstError::new(
                format!("Cannot find object in FieldAccess: {:?}", self.0),
                self.span(),
            )
        })
    }

    pub fn field_name(&self) -> Result<String, AstError> {
        let mut found_dot = false;
        for element in self.0.children_with_tokens() {
            if let Some(token) = element.into_token() {
                if token.kind() == SyntaxKind::Dot {
                    found_dot = true;
                } else if found_dot && token.kind() == SyntaxKind::Ident {
                    return Ok(token.text().to_string());
                }
            }
        }
        Err(AstError::new(
            format!("Cannot find field name in FieldAccess: {:?}", self.0),
            self.span(),
        ))
    }

    /// Returns the complete type string between `[` and `]`, e.g. "Long", `Coll[Byte]`, "(Long, Long)"
    pub fn type_arg_string(&self) -> Option<String> {
        let mut depth = 0;
        let mut result = String::new();
        for element in self.0.children_with_tokens() {
            if let Some(token) = element.into_token() {
                match token.kind() {
                    SyntaxKind::LBracket if depth == 0 => {
                        depth = 1;
                    }
                    SyntaxKind::LBracket => {
                        depth += 1;
                        result.push_str(token.text());
                    }
                    SyntaxKind::RBracket => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        result.push_str(token.text());
                    }
                    _ if depth > 0 => {
                        if !matches!(token.kind(), SyntaxKind::Whitespace | SyntaxKind::Comment) {
                            result.push_str(token.text());
                        }
                    }
                    _ => {}
                }
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct Lambda(SyntaxNode);

impl Lambda {
    /// Returns (name, type_string) pairs for each parameter.
    /// Type strings may be compound: "Box", `Coll[Byte]`, `(Coll[Byte],Long)`
    pub fn params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        let tokens: Vec<_> = self
            .0
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .collect();
        // Find tokens between outer ( and ) tracking nesting depth
        let mut in_params = false;
        let mut current_name: Option<String> = None;
        #[allow(unused_assignments)]
        let mut collecting_type = false;
        let mut type_str = String::new();
        let mut depth = 0; // nesting depth for ( ) and [ ]
        for token in &tokens {
            match token.kind() {
                SyntaxKind::LParen if !in_params => {
                    in_params = true;
                }
                SyntaxKind::RParen if in_params && depth == 0 => {
                    // End of param list
                    if collecting_type && current_name.is_some() {
                        params.push((current_name.take().unwrap(), type_str.clone()));
                        type_str.clear();
                    }
                    break;
                }
                SyntaxKind::Comma if in_params && depth == 0 => {
                    // Separator between params
                    if collecting_type && current_name.is_some() {
                        params.push((current_name.take().unwrap(), type_str.clone()));
                        type_str.clear();
                        collecting_type = false;
                    }
                    current_name = None;
                }
                SyntaxKind::Colon if in_params && depth == 0 && current_name.is_some() => {
                    collecting_type = true;
                    type_str.clear();
                }
                _ if in_params && collecting_type => {
                    if matches!(token.kind(), SyntaxKind::LParen | SyntaxKind::LBracket) {
                        depth += 1;
                    }
                    if matches!(token.kind(), SyntaxKind::RParen | SyntaxKind::RBracket) {
                        depth -= 1;
                    }
                    if !matches!(token.kind(), SyntaxKind::Whitespace | SyntaxKind::Comment) {
                        type_str.push_str(token.text());
                    }
                }
                SyntaxKind::Ident if in_params && !collecting_type && current_name.is_none() => {
                    current_name = Some(token.text().to_string());
                }
                _ => {}
            }
        }
        params
    }

    pub fn body(&self) -> Result<Expr, AstError> {
        // The body is the child Expr node (after =>)
        self.0.children().find_map(Expr::cast).ok_or_else(|| {
            AstError::new(
                format!("Cannot find body in Lambda: {:?}", self.0),
                self.span(),
            )
        })
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct PrefixExpr(SyntaxNode);

impl PrefixExpr {
    pub fn op(&self) -> Result<SyntaxToken, AstError> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| matches!(token.kind(), SyntaxKind::Minus | SyntaxKind::Bang))
            .ok_or_else(|| {
                AstError::new(
                    format!("Cannot find op in PrefixExpr: {:?}", self.0),
                    self.span(),
                )
            })
    }

    pub fn operand(&self) -> Result<Expr, AstError> {
        self.0.children().find_map(Expr::cast).ok_or_else(|| {
            AstError::new(
                format!("Cannot find operand in PrefixExpr: {:?}", self.0),
                self.span(),
            )
        })
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct TupleExpr(SyntaxNode);

impl TupleExpr {
    pub fn items(&self) -> Vec<Expr> {
        self.0.children().filter_map(Expr::cast).collect()
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct IfExpr(SyntaxNode);

impl IfExpr {
    pub fn condition(&self) -> Result<Expr, AstError> {
        // First child expr is the condition (inside parens, but ParenExpr wraps it)
        self.0.children().find_map(Expr::cast).ok_or_else(|| {
            AstError::new(
                format!("Cannot find condition in IfExpr: {:?}", self.0),
                self.span(),
            )
        })
    }

    pub fn then_branch(&self) -> Result<Expr, AstError> {
        self.0
            .children()
            .filter_map(Expr::cast)
            .nth(1)
            .ok_or_else(|| {
                AstError::new(
                    format!("Cannot find then branch in IfExpr: {:?}", self.0),
                    self.span(),
                )
            })
    }

    pub fn else_branch(&self) -> Result<Expr, AstError> {
        self.0
            .children()
            .filter_map(Expr::cast)
            .nth(2)
            .ok_or_else(|| {
                AstError::new(
                    format!("Cannot find else branch in IfExpr: {:?}", self.0),
                    self.span(),
                )
            })
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}
