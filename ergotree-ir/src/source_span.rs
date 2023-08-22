//! Source position for an IR node in the source code

use crate::mir::bin_op::BinOp;
use crate::mir::block::BlockValue;
use crate::mir::coll_append::Append;
use crate::mir::coll_by_index::ByIndex;
use crate::mir::expr::Expr;
use crate::mir::subst_const::SubstConstants;
use crate::mir::val_def::ValDef;

/// Source position for the Expr
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct SourceSpan {
    /// Start position in the span
    pub offset: usize,
    /// The length of the span
    pub length: usize,
}

impl SourceSpan {
    /// Empty span
    pub fn empty() -> Self {
        SourceSpan {
            offset: 0,
            length: 0,
        }
    }
}

impl From<(usize, usize)> for SourceSpan {
    fn from(value: (usize, usize)) -> Self {
        SourceSpan {
            offset: value.0,
            length: value.1,
        }
    }
}

impl From<SourceSpan> for miette::SourceSpan {
    fn from(value: SourceSpan) -> Self {
        miette::SourceSpan::new(value.offset.into(), value.length.into())
    }
}

/// Wrapper for Expr with source position
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Spanned<T> {
    /// Source position
    pub source_span: SourceSpan,
    /// Wrapped value
    pub expr: T,
}

impl<T> Spanned<T> {
    /// Expression
    pub fn expr(&self) -> &T {
        &self.expr
    }
}

macro_rules! into_expr {
    ($variant: ident) => {
        impl From<$variant> for Expr {
            fn from(v: $variant) -> Self {
                Expr::$variant(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: v,
                })
            }
        }
    };
}

into_expr!(Append);
into_expr!(BlockValue);
into_expr!(ValDef);
into_expr!(BinOp);
into_expr!(ByIndex);
into_expr!(SubstConstants);

impl<T> From<T> for Spanned<T> {
    fn from(v: T) -> Self {
        Spanned {
            source_span: SourceSpan::empty(),
            expr: v,
        }
    }
}

impl Expr {
    /// Source span for the Expr
    #[allow(clippy::todo)]
    pub fn span(&self) -> SourceSpan {
        match self {
            Expr::Append(op) => op.source_span,
            Expr::Const(_) => SourceSpan::empty(),
            Expr::ConstPlaceholder(_) => SourceSpan::empty(),
            Expr::SubstConstants(op) => op.source_span,
            Expr::ByteArrayToLong(_) => todo!(),
            Expr::ByteArrayToBigInt(_) => todo!(),
            Expr::LongToByteArray(_) => SourceSpan::empty(),
            Expr::Collection(_) => SourceSpan::empty(),
            Expr::Tuple(_) => SourceSpan::empty(),
            Expr::CalcBlake2b256(_) => SourceSpan::empty(),
            Expr::CalcSha256(_) => SourceSpan::empty(),
            Expr::Context => SourceSpan::empty(),
            Expr::Global => SourceSpan::empty(),
            Expr::GlobalVars(_) => SourceSpan::empty(),
            Expr::FuncValue(_) => SourceSpan::empty(),
            Expr::Apply(_) => SourceSpan::empty(),
            Expr::MethodCall(_) => todo!(),
            Expr::ProperyCall(_) => todo!(),
            Expr::BlockValue(op) => op.source_span,
            Expr::ValDef(op) => op.source_span,
            Expr::ValUse(_) => SourceSpan::empty(),
            Expr::If(_) => SourceSpan::empty(),
            Expr::BinOp(op) => op.source_span,
            Expr::And(_) => SourceSpan::empty(),
            Expr::Or(_) => SourceSpan::empty(),
            Expr::Xor(_) => SourceSpan::empty(),
            Expr::Atleast(_) => SourceSpan::empty(),
            Expr::LogicalNot(_) => SourceSpan::empty(),
            Expr::Negation(_) => todo!(),
            Expr::BitInversion(_) => SourceSpan::empty(),
            Expr::OptionGet(_) => todo!(),
            Expr::OptionIsDefined(_) => todo!(),
            Expr::OptionGetOrElse(_) => todo!(),
            Expr::ExtractAmount(_) => SourceSpan::empty(),
            Expr::ExtractRegisterAs(_) => todo!(),
            Expr::ExtractBytes(_) => SourceSpan::empty(),
            Expr::ExtractBytesWithNoRef(_) => SourceSpan::empty(),
            Expr::ExtractScriptBytes(_) => SourceSpan::empty(),
            Expr::ExtractCreationInfo(_) => SourceSpan::empty(),
            Expr::ExtractId(_) => SourceSpan::empty(),
            Expr::ByIndex(op) => op.source_span,
            Expr::SizeOf(_) => SourceSpan::empty(),
            Expr::Slice(_) => todo!(),
            Expr::Fold(_) => todo!(),
            Expr::Map(_) => todo!(),
            Expr::Filter(_) => todo!(),
            Expr::Exists(_) => todo!(),
            Expr::ForAll(_) => todo!(),
            Expr::SelectField(_) => todo!(),
            Expr::BoolToSigmaProp(_) => SourceSpan::empty(),
            Expr::Upcast(_) => SourceSpan::empty(),
            Expr::Downcast(_) => SourceSpan::empty(),
            Expr::CreateProveDlog(_) => SourceSpan::empty(),
            Expr::CreateProveDhTuple(_) => SourceSpan::empty(),
            Expr::SigmaPropBytes(_) => SourceSpan::empty(),
            Expr::DecodePoint(_) => SourceSpan::empty(),
            Expr::SigmaAnd(_) => SourceSpan::empty(),
            Expr::SigmaOr(_) => SourceSpan::empty(),
            Expr::GetVar(_) => todo!(),
            Expr::DeserializeRegister(_) => todo!(),
            Expr::DeserializeContext(_) => todo!(),
            Expr::MultiplyGroup(_) => SourceSpan::empty(),
            Expr::Exponentiate(_) => SourceSpan::empty(),
            Expr::XorOf(_) => SourceSpan::empty(),
            Expr::TreeLookup(_) => todo!(),
            Expr::CreateAvlTree(_) => SourceSpan::empty(),
        }
    }
}
