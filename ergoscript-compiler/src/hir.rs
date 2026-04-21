//! High-level Intermediate Representation
//! Refered as frontend representation in sigmastate

#[allow(dead_code)]
mod rewrite;
pub mod optimize;

use ergotree_ir::types::stuple::STuple;
use ergotree_ir::types::stype::SType;
#[allow(unused_imports)]
pub use rewrite::rewrite;

use super::ast;
use crate::ast::AstError;
use crate::error::pretty_error_desc;
use crate::syntax::SyntaxKind;
use text_size::TextRange;

extern crate derive_more;

pub fn lower(ast: ast::Root) -> Result<Expr, HirLoweringError> {
    let exprs: Vec<ast::Expr> = ast.children().collect();
    if exprs.len() > 1 {
        return Err(HirLoweringError::new(
            format!("More than one root expr found: {:?}", exprs),
            ast.span(),
        ));
    }
    let first_expr = exprs
        .first()
        .ok_or_else(|| AstError::new(format!("Cannot parse empty root: {:?}", ast), ast.span()))?;
    Expr::lower(first_expr)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: TextRange,
    pub tpe: Option<SType>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HirLoweringError {
    msg: String,
    span: TextRange,
}

impl HirLoweringError {
    pub fn new(msg: String, span: TextRange) -> Self {
        HirLoweringError { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

impl From<AstError> for HirLoweringError {
    fn from(ast: AstError) -> Self {
        HirLoweringError::new(format!("AST error: {0}", ast.msg), ast.span)
    }
}

impl Expr {
    pub fn lower(expr: &ast::Expr) -> Result<Expr, HirLoweringError> {
        match expr {
            ast::Expr::BinaryExpr(ast) => Ok(Expr {
                kind: Binary::lower(ast)?.into(),
                span: ast.span(),
                tpe: None,
            }),
            ast::Expr::Ident(ast) => {
                let name = ast.name()?;
                Ok(Expr {
                    kind: ExprKind::Ident(name.text().to_string()),
                    span: ast.span(),
                    tpe: None,
                })
            }
            ast::Expr::Literal(ast) => {
                let v = ast.parse()?;
                let expr = match v {
                    ast::LiteralValue::Int(v) => Expr {
                        kind: Literal::Int(v).into(),
                        span: ast.span(),
                        tpe: Some(SType::SInt),
                    },
                    ast::LiteralValue::Long(v) => Expr {
                        kind: Literal::Long(v).into(),
                        span: ast.span(),
                        tpe: Some(SType::SLong),
                    },
                };
                Ok(expr)
            }
            ast::Expr::StringLiteral(ast) => {
                let v = ast.value()?;
                Ok(Expr {
                    kind: Literal::String(v).into(),
                    span: ast.span(),
                    tpe: None, // String type resolved when used (e.g., in fromBase16)
                })
            }
            ast::Expr::BoolLiteral(ast) => {
                let v = ast.value()?;
                Ok(Expr {
                    kind: Literal::Bool(v).into(),
                    span: ast.span(),
                    tpe: Some(SType::SBoolean),
                })
            }
            ast::Expr::FuncCall(ast) => {
                let func = Expr::lower(&ast.func_expr()?)?;
                let args: Result<Vec<Expr>, HirLoweringError> =
                    ast.args().iter().map(Expr::lower).collect();
                let type_arg = ast.type_arg_string().and_then(|s| parse_type_str(&s));
                Ok(Expr {
                    kind: ExprKind::Apply(Apply {
                        func: Box::new(func),
                        args: args?,
                        type_arg,
                    }),
                    span: ast.span(),
                    tpe: None,
                })
            }
            ast::Expr::Block(ast) => {
                let exprs: Result<Vec<Expr>, HirLoweringError> =
                    ast.exprs().iter().map(Expr::lower).collect();
                let exprs = exprs?;
                if exprs.len() == 1 {
                    Ok(exprs.into_iter().next().unwrap())
                } else {
                    Ok(Expr {
                        kind: ExprKind::Block(exprs),
                        span: ast.span(),
                        tpe: None,
                    })
                }
            }
            ast::Expr::IfExpr(ast) => {
                let condition = Expr::lower(&ast.condition()?)?;
                let then_branch = Expr::lower(&ast.then_branch()?)?;
                let else_branch = Expr::lower(&ast.else_branch()?)?;
                Ok(Expr {
                    kind: ExprKind::If(IfExprHir {
                        condition: Box::new(condition),
                        then_branch: Box::new(then_branch),
                        else_branch: Box::new(else_branch),
                    }),
                    span: ast.span(),
                    tpe: None,
                })
            }
            ast::Expr::Lambda(ast) => {
                let params: Vec<(String, SType)> = ast
                    .params()
                    .into_iter()
                    .map(|(name, type_name)| {
                        let tpe = parse_type_str(&type_name).unwrap_or(SType::SAny);
                        (name, tpe)
                    })
                    .collect();
                let body = Expr::lower(&ast.body()?)?;
                Ok(Expr {
                    kind: ExprKind::Lambda(LambdaExpr {
                        params,
                        param_ids: vec![],
                        body: Box::new(body),
                    }),
                    span: ast.span(),
                    tpe: None,
                })
            }
            ast::Expr::FieldAccess(ast) => {
                let object = Expr::lower(&ast.object()?)?;
                let field = ast.field_name()?;
                let type_args: Vec<SType> = ast
                    .type_arg_string()
                    .and_then(|t| parse_type_str(&t))
                    .into_iter()
                    .collect();
                Ok(Expr {
                    kind: ExprKind::FieldAccess(FieldAccessExpr {
                        object: Box::new(object),
                        field,
                        type_args,
                    }),
                    span: ast.span(),
                    tpe: None,
                })
            }
            ast::Expr::PrefixExpr(ast) => {
                let op = ast.op()?;
                let operand = Expr::lower(&ast.operand()?)?;
                match op.kind() {
                    SyntaxKind::Bang => Ok(Expr {
                        kind: ExprKind::LogicalNot(Box::new(operand)),
                        span: ast.span(),
                        tpe: None,
                    }),
                    SyntaxKind::Minus => Ok(Expr {
                        kind: ExprKind::Negation(Box::new(operand)),
                        span: ast.span(),
                        tpe: None,
                    }),
                    _ => Err(HirLoweringError::new(
                        format!("Unknown prefix operator: {:?}", op),
                        ast.span(),
                    )),
                }
            }
            ast::Expr::TupleExpr(ast) => {
                let items: Result<Vec<Expr>, HirLoweringError> =
                    ast.items().iter().map(Expr::lower).collect();
                Ok(Expr {
                    kind: ExprKind::Tuple(items?),
                    span: ast.span(),
                    tpe: None,
                })
            }
            ast::Expr::VariableDef(ast) => {
                let name = ast.name()?;
                let tpe = ast.type_annotation().and_then(|t| parse_type_str(&t));
                let rhs = Expr::lower(&ast.value()?)?;
                Ok(Expr {
                    kind: ExprKind::ValDef(ValDef {
                        name,
                        id: None,
                        tpe: tpe.clone(),
                        rhs: Box::new(rhs),
                    }),
                    span: ast.span(),
                    tpe,
                })
            }
        }
    }

    #[cfg(test)]
    pub fn debug_tree(&self) -> String {
        let tree = format!("{:#?}", self);
        tree
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Spanned<T: Clone> {
    pub node: T,
    pub span: TextRange,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Binary {
    pub op: Spanned<BinaryOp>,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl Binary {
    fn lower(ast: &ast::BinaryExpr) -> Result<Binary, HirLoweringError> {
        let syntax_token = ast.op()?;
        let op = match syntax_token.kind() {
            SyntaxKind::Plus => BinaryOp::Plus,
            SyntaxKind::Minus => BinaryOp::Minus,
            SyntaxKind::Star => BinaryOp::Multiply,
            SyntaxKind::Slash => BinaryOp::Divide,
            SyntaxKind::Percent => BinaryOp::Modulo,
            SyntaxKind::And => BinaryOp::And,
            SyntaxKind::Or => BinaryOp::Or,
            SyntaxKind::EqEq => BinaryOp::Eq,
            SyntaxKind::NotEq => BinaryOp::Neq,
            SyntaxKind::Gt => BinaryOp::Gt,
            SyntaxKind::Lt => BinaryOp::Lt,
            SyntaxKind::GtEq => BinaryOp::Ge,
            SyntaxKind::LtEq => BinaryOp::Le,
            _ => {
                return Err(HirLoweringError::new(
                    format!("unknown binary operator: {:?}", ast.op()),
                    syntax_token.text_range(),
                ))
            }
        };

        let lhs = Expr::lower(&ast.lhs()?);
        let rhs = Expr::lower(&ast.rhs()?);

        Ok(Binary {
            op: Spanned {
                node: op,
                span: syntax_token.text_range(),
            },
            lhs: Box::new(lhs?),
            rhs: Box::new(rhs?),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Ident(String),
    Binary(Binary),
    GlobalVars(GlobalVars),
    Literal(Literal),
    Apply(Apply),
    Block(Vec<Expr>),
    ValDef(ValDef),
    ValUse(ValUse),
    FieldAccess(FieldAccessExpr),
    Lambda(LambdaExpr),
    If(IfExprHir),
    Context,
    Tuple(Vec<Expr>),
    Negation(Box<Expr>),
    LogicalNot(Box<Expr>),
}

impl From<Binary> for ExprKind {
    fn from(v: Binary) -> Self { ExprKind::Binary(v) }
}
impl From<GlobalVars> for ExprKind {
    fn from(v: GlobalVars) -> Self { ExprKind::GlobalVars(v) }
}
impl From<Literal> for ExprKind {
    fn from(v: Literal) -> Self { ExprKind::Literal(v) }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GlobalVars {
    Height,
    SelfBox,
    Inputs,
    Outputs,
}

impl GlobalVars {
    /// Type
    pub fn tpe(&self) -> SType {
        match self {
            GlobalVars::Height => SType::SInt,
            GlobalVars::SelfBox => SType::SBox,
            GlobalVars::Inputs => SType::SColl(SType::SBox.into()),
            GlobalVars::Outputs => SType::SColl(SType::SBox.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccessExpr {
    pub object: Box<Expr>,
    pub field: String,
    pub type_args: Vec<SType>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    Int(i32),
    Long(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Apply {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
    /// Type argument from Coll[Byte](...), getVar[Int](...) etc.
    pub type_arg: Option<SType>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ValDef {
    pub name: String,
    pub id: Option<u32>,
    pub tpe: Option<SType>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ValUse {
    pub id: u32,
    pub tpe: SType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LambdaExpr {
    /// (name, type) pairs
    pub params: Vec<(String, SType)>,
    /// Assigned ValIds for each param (set by binder)
    pub param_ids: Vec<u32>,
    pub body: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfExprHir {
    pub condition: Box<Expr>,
    pub then_branch: Box<Expr>,
    pub else_branch: Box<Expr>,
}

pub fn parse_type_name(name: &str) -> Option<SType> {
    match name {
        "Int" => Some(SType::SInt),
        "Long" => Some(SType::SLong),
        "Boolean" => Some(SType::SBoolean),
        "Byte" => Some(SType::SByte),
        "Short" => Some(SType::SShort),
        "BigInt" => Some(SType::SBigInt),
        "SigmaProp" => Some(SType::SSigmaProp),
        "GroupElement" => Some(SType::SGroupElement),
        "Box" => Some(SType::SBox),
        "AvlTree" => Some(SType::SAvlTree),
        "Any" => Some(SType::SAny),
        _ => None,
    }
}

/// Parse a compound type string like "Long", "Coll[Byte]", "(Long,Long)", "(Coll[Byte],Long)"
pub fn parse_type_str(s: &str) -> Option<SType> {
    let s = s.trim();
    if s.starts_with('(') && s.ends_with(')') {
        // Tuple type: (T1, T2, ...)
        let inner = &s[1..s.len() - 1];
        let parts = split_type_args(inner);
        let types: Vec<SType> = parts.into_iter().filter_map(|p| parse_type_str(p.trim())).collect();
        if types.len() >= 2 {
            STuple::try_from(types).ok().map(SType::STuple)
        } else {
            None
        }
    } else if let Some(bracket_pos) = s.find('[') {
        // Generic type: Name[InnerType]
        let name = &s[..bracket_pos];
        let inner = &s[bracket_pos + 1..s.len() - 1]; // strip [ and ]
        let inner_type = parse_type_str(inner)?;
        match name {
            "Coll" => Some(SType::SColl(inner_type.into())),
            "Option" => Some(SType::SOption(inner_type.into())),
            _ => None,
        }
    } else {
        parse_type_name(s)
    }
}

/// Split comma-separated type args respecting nesting
fn split_type_args(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::compiler::compile_hir;

    fn check(input: &str, expected_tree: expect_test::Expect) {
        let res = compile_hir(input);

        let expected_out = res
            .map(|tree| tree.debug_tree())
            .unwrap_or_else(|e| e.pretty_desc(input));
        expected_tree.assert_eq(&expected_out);
    }

    #[test]
    fn long_literal() {
        check(
            "42L",
            expect![[r#"
            Expr {
                kind: Literal(
                    Long(
                        42,
                    ),
                ),
                span: 0..3,
                tpe: Some(
                    SLong,
                ),
            }"#]],
        );
    }
}
