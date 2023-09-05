use thiserror::Error;

use crate::mir::bin_op::BinOp;
use crate::mir::block::BlockValue;
use crate::mir::coll_append::Append;
use crate::mir::coll_by_index::ByIndex;
use crate::mir::coll_filter::Filter;
use crate::mir::coll_fold::Fold;
use crate::mir::coll_map::Map;
use crate::mir::coll_size::SizeOf;
use crate::mir::constant::Constant;
use crate::mir::expr::Expr;
use crate::mir::extract_creation_info::ExtractCreationInfo;
use crate::mir::extract_reg_as::ExtractRegisterAs;
use crate::mir::func_value::FuncValue;
use crate::mir::global_vars::GlobalVars;
use crate::mir::if_op::If;
use crate::mir::option_get::OptionGet;
use crate::mir::option_is_defined::OptionIsDefined;
use crate::mir::property_call::PropertyCall;
use crate::mir::select_field::SelectField;
use crate::mir::tuple::Tuple;
use crate::mir::unary_op::OneArgOpTryBuild;
use crate::mir::val_def::ValDef;
use crate::mir::val_use::ValUse;
use crate::source_span::SourceSpan;
use crate::source_span::Spanned;
use crate::types::stype::SType;

use super::Printer;

/// Print error
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Debug, Clone, Error)]
pub enum PrintError {
    #[error("fmt error: {0:?}")]
    FmtError(#[from] std::fmt::Error),
}

/// Print trait for Expr that sets the source span for the resulting Expr
pub trait Print {
    /// Print the expression and return the resulting expression with source span
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError>;
}

#[allow(clippy::todo)]
impl Print for Expr {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        match self {
            Expr::Append(v) => v.expr().print(w),
            Expr::BlockValue(v) => v.expr().print(w),
            Expr::ValDef(v) => v.expr().print(w),
            Expr::ValUse(v) => v.print(w),
            Expr::Const(v) => v.print(w),
            Expr::BinOp(v) => v.expr().print(w),
            Expr::GlobalVars(v) => v.print(w),
            Expr::ByIndex(v) => v.expr().print(w),
            Expr::ConstPlaceholder(_) => Ok(self.clone()),
            Expr::SubstConstants(_) => todo!(),
            Expr::ByteArrayToLong(_) => todo!(),
            Expr::ByteArrayToBigInt(_) => todo!(),
            Expr::LongToByteArray(_) => todo!(),
            Expr::Collection(_) => todo!(),
            Expr::Tuple(v) => v.print(w),
            Expr::CalcBlake2b256(_) => todo!(),
            Expr::CalcSha256(_) => todo!(),
            Expr::Context => todo!(),
            Expr::Global => todo!(),
            Expr::FuncValue(v) => v.print(w),
            Expr::Apply(_) => todo!(),
            Expr::MethodCall(_) => todo!(),
            Expr::PropertyCall(v) => v.expr().print(w),
            Expr::If(v) => v.print(w),
            Expr::And(_) => todo!(),
            Expr::Or(_) => todo!(),
            Expr::Xor(_) => todo!(),
            Expr::Atleast(_) => todo!(),
            Expr::LogicalNot(_) => todo!(),
            Expr::Negation(_) => todo!(),
            Expr::BitInversion(_) => todo!(),
            Expr::OptionGet(v) => v.expr().print(w),
            Expr::OptionIsDefined(v) => v.expr().print(w),
            Expr::OptionGetOrElse(_) => todo!(),
            Expr::ExtractAmount(_) => todo!(),
            Expr::ExtractRegisterAs(v) => v.expr().print(w),
            Expr::ExtractBytes(_) => todo!(),
            Expr::ExtractBytesWithNoRef(_) => todo!(),
            Expr::ExtractScriptBytes(_) => todo!(),
            Expr::ExtractCreationInfo(v) => v.print(w),
            Expr::ExtractId(_) => todo!(),
            Expr::SizeOf(v) => v.print(w),
            Expr::Slice(_) => todo!(),
            Expr::Fold(v) => v.expr().print(w),
            Expr::Map(v) => v.expr().print(w),
            Expr::Filter(v) => v.expr().print(w),
            Expr::Exists(_) => todo!(),
            Expr::ForAll(_) => todo!(),
            Expr::SelectField(v) => v.expr().print(w),
            Expr::BoolToSigmaProp(_) => todo!(),
            Expr::Upcast(_) => todo!(),
            Expr::Downcast(_) => todo!(),
            Expr::CreateProveDlog(_) => todo!(),
            Expr::CreateProveDhTuple(_) => todo!(),
            Expr::SigmaPropBytes(_) => todo!(),
            Expr::DecodePoint(_) => todo!(),
            Expr::SigmaAnd(_) => todo!(),
            Expr::SigmaOr(_) => todo!(),
            Expr::GetVar(_) => todo!(),
            Expr::DeserializeRegister(_) => todo!(),
            Expr::DeserializeContext(_) => todo!(),
            Expr::MultiplyGroup(_) => todo!(),
            Expr::Exponentiate(_) => todo!(),
            Expr::XorOf(_) => todo!(),
            Expr::TreeLookup(_) => todo!(),
            Expr::CreateAvlTree(_) => todo!(),
        }
    }
}

impl Print for BlockValue {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        writeln!(w, "{{")?;
        w.inc_ident();
        let indent = w.get_indent();
        let mut items = Vec::new();
        for item in &self.items {
            write!(w, "{:indent$}", "", indent = indent)?;
            items.push(item.print(w)?);
            writeln!(w)?;
        }
        // indent for result
        write!(w, "{:indent$}", "", indent = indent)?;
        let res = self.result.print(w)?;
        w.dec_ident();
        writeln!(w, "\n}}")?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: BlockValue {
                items,
                result: Box::new(res),
            },
        }
        .into())
    }
}

impl Print for ValDef {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        write!(w, "val v{} = ", self.id)?;
        let rhs = self.rhs.print(w)?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: ValDef {
                id: self.id,
                rhs: Box::new(rhs),
            },
        }
        .into())
    }
}

impl Print for Constant {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "{:?}", self.v)?;
        Ok(self.clone().into())
    }
}

impl Print for ValUse {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "v{}", self.val_id)?;
        Ok(self.clone().into())
    }
}

impl Print for Append {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let input = self.input.print(w)?;
        write!(w, ".append(")?;
        let col_2 = self.col_2.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: Append {
                input: Box::new(input),
                col_2: Box::new(col_2),
            },
        }
        .into())
    }
}

impl Print for BinOp {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let left = self.left.print(w)?;
        write!(w, " {} ", self.kind)?;
        let right = self.right.print(w)?;
        let length = w.current_pos() - offset;
        // dbg!(offset, length);
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: BinOp {
                kind: self.kind,
                left: Box::new(left),
                right: Box::new(right),
            },
        }
        .into())
    }
}

impl Print for GlobalVars {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "{}", self)?;
        Ok(self.clone().into())
    }
}

impl Print for ByIndex {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        let offset = w.current_pos();
        write!(w, "(")?;
        let index = self.index.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: ByIndex::new(input, index, self.default.clone()).unwrap(),
        }
        .into())
    }
}

impl Print for Map {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        let offset = w.current_pos();
        write!(w, ".map(")?;
        let mapper = self.mapper.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: Map::new(input, mapper).unwrap(),
        }
        .into())
    }
}

impl Print for Fold {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        let offset = w.current_pos();
        write!(w, ".fold(")?;
        let zero = self.zero.print(w)?;
        write!(w, ")(")?;
        let fold_op = self.fold_op.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: Fold::new(input, zero, fold_op).unwrap(),
        }
        .into())
    }
}

impl Print for FuncValue {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        writeln!(w, "{{")?;
        w.inc_ident();
        writeln!(
            w,
            "({}) => ",
            self.args()
                .iter()
                .map(|a| format!("{}", a))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        w.inc_ident();
        let body = self.body().print(w)?;
        w.dec_ident();
        write!(w, "}}")?;
        w.dec_ident();
        Ok(FuncValue::new(self.args().to_vec(), body).into())
    }
}

impl Print for Filter {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        let offset = w.current_pos();
        write!(w, ".filter(")?;
        let condition = self.condition.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: Filter::new(input, condition).unwrap(),
        }
        .into())
    }
}

impl Print for If {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "if (")?;
        let condition = self.condition.print(w)?;
        write!(w, ") ")?;
        let true_branch = self.true_branch.print(w)?;
        write!(w, " else ")?;
        let false_branch = self.false_branch.print(w)?;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(If {
            condition: condition.into(),
            true_branch: true_branch.into(),
            false_branch: false_branch.into(),
        }
        .into())
    }
}

impl Print for OptionIsDefined {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let input = self.input.print(w)?;
        write!(w, ".isDefined()")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: OptionIsDefined {
                input: Box::new(input),
            },
        }
        .into())
    }
}

impl Print for ExtractRegisterAs {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let input = self.input.print(w)?;
        write!(w, ".getReg")?;
        write!(w, "({})", self.register_id)?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: ExtractRegisterAs::new(
                input,
                self.register_id,
                SType::SOption(self.elem_tpe.clone().into()),
            )
            .unwrap(),
        }
        .into())
    }
}

impl Print for SelectField {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let input = self.input.print(w)?;
        write!(w, "_{}", self.field_index)?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: SelectField::new(input, self.field_index).unwrap(),
        }
        .into())
    }
}

impl Print for ExtractCreationInfo {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        write!(w, ".creationInfo")?;
        Ok(ExtractCreationInfo {
            input: input.into(),
        }
        .into())
    }
}

impl Print for PropertyCall {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let obj = self.obj.print(w)?;
        write!(w, ".{}", self.method.name())?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: PropertyCall {
                obj: Box::new(obj),
                method: self.method.clone(),
            },
        }
        .into())
    }
}

impl Print for OptionGet {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let input = self.input.print(w)?;
        write!(w, ".get")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: OptionGet::try_build(input).unwrap(),
        }
        .into())
    }
}

impl Print for SizeOf {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        write!(w, ".size")?;
        Ok(SizeOf {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for Tuple {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "(")?;
        let items = self.items.try_mapped_ref(|i| {
            write!(w, ", ")?;
            i.print(w)
        })?;
        write!(w, ")")?;
        Ok(Tuple { items }.into())
    }
}
