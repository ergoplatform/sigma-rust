use thiserror::Error;

use crate::mir::and::And;
use crate::mir::apply::Apply;
use crate::mir::atleast::Atleast;
use crate::mir::bin_op::BinOp;
use crate::mir::bit_inversion::BitInversion;
use crate::mir::block::BlockValue;
use crate::mir::bool_to_sigma::BoolToSigmaProp;
use crate::mir::byte_array_to_bigint::ByteArrayToBigInt;
use crate::mir::byte_array_to_long::ByteArrayToLong;
use crate::mir::calc_blake2b256::CalcBlake2b256;
use crate::mir::calc_sha256::CalcSha256;
use crate::mir::coll_append::Append;
use crate::mir::coll_by_index::ByIndex;
use crate::mir::coll_exists::Exists;
use crate::mir::coll_filter::Filter;
use crate::mir::coll_fold::Fold;
use crate::mir::coll_forall::ForAll;
use crate::mir::coll_map::Map;
use crate::mir::coll_size::SizeOf;
use crate::mir::coll_slice::Slice;
use crate::mir::collection::Collection;
use crate::mir::constant::Constant;
use crate::mir::create_avl_tree::CreateAvlTree;
use crate::mir::create_prove_dh_tuple::CreateProveDhTuple;
use crate::mir::create_provedlog::CreateProveDlog;
use crate::mir::decode_point::DecodePoint;
use crate::mir::deserialize_context::DeserializeContext;
use crate::mir::deserialize_register::DeserializeRegister;
use crate::mir::downcast::Downcast;
use crate::mir::exponentiate::Exponentiate;
use crate::mir::expr::Expr;
use crate::mir::extract_amount::ExtractAmount;
use crate::mir::extract_bytes::ExtractBytes;
use crate::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef;
use crate::mir::extract_creation_info::ExtractCreationInfo;
use crate::mir::extract_id::ExtractId;
use crate::mir::extract_reg_as::ExtractRegisterAs;
use crate::mir::extract_script_bytes::ExtractScriptBytes;
use crate::mir::func_value::FuncValue;
use crate::mir::get_var::GetVar;
use crate::mir::global_vars::GlobalVars;
use crate::mir::if_op::If;
use crate::mir::logical_not::LogicalNot;
use crate::mir::long_to_byte_array::LongToByteArray;
use crate::mir::method_call::MethodCall;
use crate::mir::multiply_group::MultiplyGroup;
use crate::mir::negation::Negation;
use crate::mir::option_get::OptionGet;
use crate::mir::option_get_or_else::OptionGetOrElse;
use crate::mir::option_is_defined::OptionIsDefined;
use crate::mir::or::Or;
use crate::mir::property_call::PropertyCall;
use crate::mir::select_field::SelectField;
use crate::mir::sigma_and::SigmaAnd;
use crate::mir::sigma_or::SigmaOr;
use crate::mir::sigma_prop_bytes::SigmaPropBytes;
use crate::mir::subst_const::SubstConstants;
use crate::mir::tree_lookup::TreeLookup;
use crate::mir::tuple::Tuple;
use crate::mir::unary_op::OneArgOpTryBuild;
use crate::mir::upcast::Upcast;
use crate::mir::val_def::ValDef;
use crate::mir::val_use::ValUse;
use crate::mir::xor::Xor;
use crate::mir::xor_of::XorOf;
use crate::source_span::SourceSpan;
use crate::source_span::Spanned;
use crate::types::stype::SType;

use super::PosTrackingWriter;
use super::Printer;

/// Print error
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Debug, Clone, Error)]
pub enum PrintError {
    #[error("fmt error: {0:?}")]
    FmtError(#[from] std::fmt::Error),
}

impl Expr {
    /// Returns pretty printed tree
    pub fn pretty_print(&self) -> Result<(Expr, String), PrintError> {
        let mut printer = PosTrackingWriter::new();
        let spanned_expr = self.print(&mut printer)?;
        let printed_expr_str = printer.get_buf();
        Ok((spanned_expr, printed_expr_str.to_owned()))
    }
}

/// Print trait for Expr that sets the source span for the resulting Expr
pub trait Print {
    /// Print the expression and return the resulting expression with source span
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError>;
}

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
            Expr::SubstConstants(v) => v.expr().print(w),
            Expr::ByteArrayToLong(v) => v.expr().print(w),
            Expr::ByteArrayToBigInt(v) => v.expr().print(w),
            Expr::LongToByteArray(v) => v.print(w),
            Expr::Collection(v) => v.print(w),
            Expr::Tuple(v) => v.print(w),
            Expr::CalcBlake2b256(v) => v.print(w),
            Expr::CalcSha256(v) => v.print(w),
            Expr::Context => {
                write!(w, "CONTEXT")?;
                Ok(self.clone())
            }
            Expr::Global => {
                write!(w, "GLOBAL")?;
                Ok(self.clone())
            }
            Expr::FuncValue(v) => v.print(w),
            Expr::Apply(v) => v.print(w),
            Expr::MethodCall(v) => v.expr().print(w),
            Expr::PropertyCall(v) => v.expr().print(w),
            Expr::If(v) => v.print(w),
            Expr::And(v) => v.expr().print(w),
            Expr::Or(v) => v.expr().print(w),
            Expr::Xor(v) => v.print(w),
            Expr::Atleast(v) => v.print(w),
            Expr::LogicalNot(v) => v.expr().print(w),
            Expr::Negation(v) => v.expr().print(w),
            Expr::BitInversion(v) => v.print(w),
            Expr::OptionGet(v) => v.expr().print(w),
            Expr::OptionIsDefined(v) => v.expr().print(w),
            Expr::OptionGetOrElse(v) => v.expr().print(w),
            Expr::ExtractAmount(v) => v.print(w),
            Expr::ExtractRegisterAs(v) => v.expr().print(w),
            Expr::ExtractBytes(v) => v.print(w),
            Expr::ExtractBytesWithNoRef(v) => v.print(w),
            Expr::ExtractScriptBytes(v) => v.print(w),
            Expr::ExtractCreationInfo(v) => v.print(w),
            Expr::ExtractId(v) => v.print(w),
            Expr::SizeOf(v) => v.print(w),
            Expr::Slice(v) => v.expr().print(w),
            Expr::Fold(v) => v.expr().print(w),
            Expr::Map(v) => v.expr().print(w),
            Expr::Filter(v) => v.expr().print(w),
            Expr::Exists(v) => v.expr().print(w),
            Expr::ForAll(v) => v.expr().print(w),
            Expr::SelectField(v) => v.expr().print(w),
            Expr::BoolToSigmaProp(v) => v.print(w),
            Expr::Upcast(v) => v.print(w),
            Expr::Downcast(v) => v.print(w),
            Expr::CreateProveDlog(v) => v.print(w),
            Expr::CreateProveDhTuple(v) => v.print(w),
            Expr::SigmaPropBytes(v) => v.print(w),
            Expr::DecodePoint(v) => v.print(w),
            Expr::SigmaAnd(v) => v.print(w),
            Expr::SigmaOr(v) => v.print(w),
            Expr::GetVar(v) => v.expr().print(w),
            Expr::DeserializeRegister(v) => v.print(w),
            Expr::DeserializeContext(v) => v.print(w),
            Expr::MultiplyGroup(v) => v.print(w),
            Expr::Exponentiate(v) => v.print(w),
            Expr::XorOf(v) => v.print(w),
            Expr::TreeLookup(v) => v.expr().print(w),
            Expr::CreateAvlTree(v) => v.print(w),
        }
    }
}

impl Print for BlockValue {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        writeln!(w, "{{")?;
        w.inc_ident();
        let mut items = Vec::new();
        for item in &self.items {
            w.print_indent()?;
            items.push(item.print(w)?);
            writeln!(w)?;
        }
        // indent for result
        w.print_indent()?;
        let res = self.result.print(w)?;
        writeln!(w)?;
        w.dec_ident();
        w.print_indent()?;
        writeln!(w, "}}")?;
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

impl Print for Exists {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        let offset = w.current_pos();
        write!(w, ".exists(")?;
        let condition = self.condition.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: Exists::new(input, condition).unwrap(),
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
        w.print_indent()?;
        write!(w, ")")?;
        w.dec_ident();
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
        w.inc_ident();
        writeln!(w, "{{")?;
        w.inc_ident();
        w.print_indent()?;
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
        w.print_indent()?;
        let body = self.body().print(w)?;
        w.dec_ident();
        writeln!(w)?;
        w.print_indent()?;
        writeln!(w, "}}")?;
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
        w.print_indent()?;
        write!(w, ")")?;
        w.dec_ident();
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
        write!(w, "._{}", self.field_index)?;
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

impl Print for MethodCall {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let obj = self.obj.print(w)?;
        write!(w, ".{}", self.method.name())?;
        write!(w, "(")?;
        let args = self
            .args
            .iter()
            .map(|a| -> Result<Expr, PrintError> { a.print(w) })
            .collect::<Result<Vec<_>, _>>()?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            source_span: SourceSpan { offset, length },
            expr: MethodCall {
                obj: Box::new(obj),
                method: self.method.clone(),
                args,
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

impl Print for SigmaAnd {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        writeln!(w, "allOf(")?;
        let items = self.items.try_mapped_ref(|i| -> Result<Expr, PrintError> {
            w.inc_ident();
            w.print_indent()?;
            let item = i.print(w)?;
            write!(w, ", ")?;
            writeln!(w)?;
            w.dec_ident();
            Ok(item)
        })?;
        w.print_indent()?;
        write!(w, ")")?;
        Ok(SigmaAnd { items }.into())
    }
}

impl Print for SigmaOr {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        writeln!(w, "anyOf(")?;
        let items = self.items.try_mapped_ref(|i| -> Result<Expr, PrintError> {
            w.inc_ident();
            w.print_indent()?;
            let item = i.print(w)?;
            write!(w, ", ")?;
            writeln!(w)?;
            w.dec_ident();
            Ok(item)
        })?;
        w.print_indent()?;
        write!(w, ")")?;
        Ok(SigmaOr { items }.into())
    }
}

impl Print for And {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        write!(w, "&&")?;
        let input = self.input.print(w)?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            expr: And {
                input: Box::new(input),
            },
            source_span: SourceSpan { offset, length },
        }
        .into())
    }
}

impl Print for Or {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        write!(w, "||")?;
        let input = self.input.print(w)?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            expr: Or {
                input: Box::new(input),
            },
            source_span: SourceSpan { offset, length },
        }
        .into())
    }
}

impl Print for CreateProveDlog {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "proveDlog(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(CreateProveDlog {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for GetVar {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "getVar({})", self.var_id)?;
        Ok(self.clone().into())
    }
}

impl Print for BoolToSigmaProp {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "sigmaProp(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(BoolToSigmaProp {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for Upcast {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "upcast(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Upcast::new(input, self.tpe.clone()).unwrap().into())
    }
}

impl Print for ExtractScriptBytes {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        write!(w, ".propBytes")?;
        Ok(ExtractScriptBytes {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for ExtractAmount {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        write!(w, ".value")?;
        Ok(ExtractAmount {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for LogicalNot {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "!")?;
        let input = self.input.print(w)?;
        Ok(LogicalNot {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for CalcBlake2b256 {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "blake2b256(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(CalcBlake2b256 {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for Negation {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "-")?;
        let input = self.input.print(w)?;
        Ok(Negation {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for ExtractId {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        write!(w, ".id")?;
        Ok(ExtractId {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for Apply {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let func = self.func.print(w)?;
        write!(w, "(")?;
        let args = self
            .args
            .iter()
            .map(|a| -> Result<Expr, PrintError> { a.print(w) })
            .collect::<Result<Vec<_>, _>>()?;
        write!(w, ")")?;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Apply::new(func, args).unwrap().into())
    }
}

impl Print for Collection {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "Coll[{}](", self.tpe())?;
        match self {
            Collection::BoolConstants(bools) => {
                for b in bools {
                    write!(w, "{}, ", b)?;
                }
                write!(w, ")")?;
                Ok(Collection::from_bools(bools.clone()).into())
            }
            Collection::Exprs { elem_tpe, items } => {
                let items = items
                    .iter()
                    .map(|i| {
                        write!(w, ", ")?;
                        i.print(w)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                write!(w, ")")?;
                #[allow(clippy::unwrap_used)] // we only added spans
                Ok(Collection::new(elem_tpe.clone(), items).unwrap().into())
            }
        }
    }
}

impl Print for ExtractBytes {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        write!(w, ".bytes")?;
        Ok(ExtractBytes {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for ByteArrayToLong {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "byteArrayToLong(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(ByteArrayToLong {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for ByteArrayToBigInt {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "byteArrayToBigInt(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(ByteArrayToBigInt {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for LongToByteArray {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "longToByteArray(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(LongToByteArray {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for CalcSha256 {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "sha256(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(CalcSha256 {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for Xor {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let left = self.left.print(w)?;
        write!(w, "^")?;
        let right = self.right.print(w)?;
        Ok(Xor {
            left: left.into(),
            right: right.into(),
        }
        .into())
    }
}

impl Print for Atleast {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, ".atLeast(")?;
        let bound = self.bound.print(w)?;
        write!(w, ", ")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(Atleast {
            input: Box::new(input),
            bound: Box::new(bound),
        }
        .into())
    }
}

impl Print for SubstConstants {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        write!(w, ".substConstants(")?;
        let script_bytes = self.script_bytes.print(w)?;
        write!(w, ", ")?;
        let positions = self.positions.print(w)?;
        write!(w, ", ")?;
        let new_values = self.new_values.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            expr: SubstConstants {
                script_bytes: script_bytes.into(),
                positions: positions.into(),
                new_values: new_values.into(),
            },
            source_span: SourceSpan { offset, length },
        }
        .into())
    }
}

impl Print for BitInversion {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "~")?;
        let input = self.input.print(w)?;
        Ok(BitInversion {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for OptionGetOrElse {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let input = self.input.print(w)?;
        write!(w, ".getOrElse(")?;
        let default = self.default.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            expr: OptionGetOrElse::new(input, default).unwrap(),
            source_span: SourceSpan { offset, length },
        }
        .into())
    }
}

impl Print for ExtractBytesWithNoRef {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let input = self.input.print(w)?;
        write!(w, ".bytesWithNoRef")?;
        Ok(ExtractBytesWithNoRef {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for Slice {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let input = self.input.print(w)?;
        write!(w, ".slice(")?;
        let from = self.from.print(w)?;
        write!(w, ", ")?;
        let until = self.until.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            expr: Slice {
                input: Box::new(input),
                from: Box::new(from),
                until: Box::new(until),
            },
            source_span: SourceSpan { offset, length },
        }
        .into())
    }
}

impl Print for ForAll {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        let input = self.input.print(w)?;
        write!(w, ".forall(")?;
        let condition = self.condition.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        #[allow(clippy::unwrap_used)] // we only added spans
        Ok(Spanned {
            expr: ForAll::new(input, condition).unwrap(),
            source_span: SourceSpan { offset, length },
        }
        .into())
    }
}

impl Print for Downcast {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "downcast(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(Downcast {
            input: Box::new(input),
            tpe: self.tpe.clone(),
        }
        .into())
    }
}

impl Print for CreateProveDhTuple {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "proveDHTuple(")?;
        let g = self.g.print(w)?;
        write!(w, ", ")?;
        let h = self.h.print(w)?;
        write!(w, ", ")?;
        let u = self.u.print(w)?;
        write!(w, ", ")?;
        let v = self.v.print(w)?;
        write!(w, ")")?;
        Ok(CreateProveDhTuple {
            g: Box::new(g),
            h: Box::new(h),
            u: Box::new(u),
            v: Box::new(v),
        }
        .into())
    }
}

impl Print for SigmaPropBytes {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "sigmaPropBytes(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(SigmaPropBytes {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for DecodePoint {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "decodePoint(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(DecodePoint {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for DeserializeRegister {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "deserializeRegister({})", self.reg)?;
        Ok(self.clone().into())
    }
}

impl Print for DeserializeContext {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "deserializeContext({})", self.id)?;
        Ok(self.clone().into())
    }
}

impl Print for MultiplyGroup {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "multiplyGroup(")?;
        let left = self.left.print(w)?;
        write!(w, ", ")?;
        let right = self.right.print(w)?;
        write!(w, ")")?;
        Ok(MultiplyGroup {
            left: left.into(),
            right: right.into(),
        }
        .into())
    }
}

impl Print for Exponentiate {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "exponentiate(")?;
        let left = self.left.print(w)?;
        write!(w, ", ")?;
        let right = self.right.print(w)?;
        write!(w, ")")?;
        Ok(Exponentiate {
            left: left.into(),
            right: right.into(),
        }
        .into())
    }
}

impl Print for XorOf {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "xorOf(")?;
        let input = self.input.print(w)?;
        write!(w, ")")?;
        Ok(XorOf {
            input: Box::new(input),
        }
        .into())
    }
}

impl Print for TreeLookup {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let offset = w.current_pos();
        write!(w, "treeLookup(")?;
        let tree = self.tree.print(w)?;
        write!(w, ", ")?;
        let key = self.key.print(w)?;
        write!(w, ", ")?;
        let proof = self.proof.print(w)?;
        write!(w, ")")?;
        let length = w.current_pos() - offset;
        Ok(Spanned {
            expr: TreeLookup {
                tree: Box::new(tree),
                key: Box::new(key),
                proof: Box::new(proof),
            },
            source_span: SourceSpan { offset, length },
        }
        .into())
    }
}

impl Print for CreateAvlTree {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        write!(w, "avlTree(")?;
        let digest = self.digest.print(w)?;
        write!(w, ", ")?;
        let key_length = self.key_length.print(w)?;
        write!(w, ")")?;
        Ok(CreateAvlTree {
            digest: Box::new(digest),
            key_length: Box::new(key_length),
            flags: self.flags.clone(),
            value_length: self.value_length.clone(),
        }
        .into())
    }
}
