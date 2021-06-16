//! IR expression

use std::convert::TryFrom;
use std::convert::TryInto;

use crate::has_opcode::HasOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::LiftIntoSType;
use crate::types::stype::SType;

use super::and::And;
use super::apply::Apply;
use super::bin_op::BinOp;
use super::block::BlockValue;
use super::bool_to_sigma::BoolToSigmaProp;
use super::byte_array_to_long::ByteArrayToLong;
use super::calc_blake2b256::CalcBlake2b256;
use super::calc_sha256::CalcSha256;
use super::coll_append::Append;
use super::coll_by_index::ByIndex;
use super::coll_exists::Exists;
use super::coll_filter::Filter;
use super::coll_fold::Fold;
use super::coll_forall::ForAll;
use super::coll_map::Map;
use super::coll_size::SizeOf;
use super::coll_slice::Slice;
use super::collection::Collection;
use super::constant::Constant;
use super::constant::ConstantPlaceholder;
use super::constant::TryExtractFrom;
use super::constant::TryExtractFromError;
use super::create_provedlog::CreateProveDlog;
use super::decode_point::DecodePoint;
use super::extract_amount::ExtractAmount;
use super::extract_creation_info::ExtractCreationInfo;
use super::extract_id::ExtractId;
use super::extract_reg_as::ExtractRegisterAs;
use super::extract_script_bytes::ExtractScriptBytes;
use super::func_value::FuncValue;
use super::global_vars::GlobalVars;
use super::if_op::If;
use super::logical_not::LogicalNot;
use super::long_to_byte_array::LongToByteArray;
use super::method_call::MethodCall;
use super::multiply_group::MultiplyGroup;
use super::negation::Negation;
use super::option_get::OptionGet;
use super::option_get_or_else::OptionGetOrElse;
use super::option_is_defined::OptionIsDefined;
use super::or::Or;
use super::property_call::PropertyCall;
use super::select_field::SelectField;
use super::sigma_and::SigmaAnd;
use super::sigma_or::SigmaOr;
use super::sigma_prop_bytes::SigmaPropBytes;
use super::tuple::Tuple;
use super::upcast::Upcast;
use super::val_def::ValDef;
use super::val_use::ValUse;
use super::value::Value;
use super::xor::Xor;

extern crate derive_more;
use crate::mir::atleast::Atleast;
use crate::mir::byte_array_to_bigint::ByteArrayToBigInt;
use crate::mir::create_prove_dh_tuple::CreateProveDhTuple;
use crate::mir::deserialize_context::DeserializeContext;
use crate::mir::deserialize_register::DeserializeRegister;
use crate::mir::get_var::GetVar;
use bounded_vec::BoundedVecOutOfBounds;
use derive_more::From;
use derive_more::TryInto;

#[derive(PartialEq, Eq, Debug, Clone, From, TryInto)]
/// Expression in ErgoTree
pub enum Expr {
    /// Append - Concatenation of two collections
    Append(Append),
    /// Constant value
    Const(Constant),
    /// Placeholder for a constant
    ConstPlaceholder(ConstantPlaceholder),
    /// Convert byte array to SLong
    ByteArrayToLong(ByteArrayToLong),
    /// Convert byte array to SLong
    ByteArrayToBigInt(ByteArrayToBigInt),
    /// Convert SLong to a byte array
    LongToByteArray(LongToByteArray),
    /// Collection declaration (array of expressions of the same type)
    Collection(Collection),
    /// Tuple declaration
    Tuple(Tuple),
    /// Predefined functions (global)
    /// Blake2b256 hash calculation
    CalcBlake2b256(CalcBlake2b256),
    /// Sha256 hash calculation
    CalcSha256(CalcSha256),
    /// Context variables (external)
    Context,
    /// Special global value which is used to define methods
    Global,
    /// Predefined global variables
    GlobalVars(GlobalVars),
    /// Function definition
    FuncValue(FuncValue),
    /// Function application
    Apply(Apply),
    /// Method call
    MethodCall(MethodCall),
    /// Property call
    ProperyCall(PropertyCall),
    /// Block (statements, followed by an expression)
    BlockValue(BlockValue),
    /// let-bound expression
    ValDef(ValDef),
    /// Reference to ValDef
    ValUse(ValUse),
    /// If, non-lazy - evaluate both branches
    If(If),
    /// Binary operation
    BinOp(BinOp),
    /// Logical AND
    And(And),
    /// Logical OR
    Or(Or),
    /// Byte-wise XOR
    Xor(Xor),
    /// THRESHOLD composition for sigma expressions
    Atleast(Atleast),
    /// LogicalNot
    LogicalNot(LogicalNot),
    /// Negation on numeric type
    Negation(Negation),
    /// Option.get method
    OptionGet(OptionGet),
    /// Option.isDefined method
    OptionIsDefined(OptionIsDefined),
    /// Returns the option's value if the option is nonempty, otherwise return the result of evaluating `default`.
    OptionGetOrElse(OptionGetOrElse),
    /// Box monetary value
    ExtractAmount(ExtractAmount),
    /// Extract register's value (box.RX properties)
    ExtractRegisterAs(ExtractRegisterAs),
    /// Extract box's guarding script serialized to bytes
    ExtractScriptBytes(ExtractScriptBytes),
    /// Tuple of height when block got included into the blockchain and transaction identifier with
    /// box index in the transaction outputs serialized to the byte array.
    ExtractCreationInfo(ExtractCreationInfo),
    /// Box id, Blake2b256 hash of this box's content, basically equals to `blake2b256(bytes)`
    ExtractId(ExtractId),
    /// Collection, get element by index
    ByIndex(ByIndex),
    /// Collection size
    SizeOf(SizeOf),
    /// Collection slice
    Slice(Slice),
    /// Collection fold op
    Fold(Fold),
    /// Collection map op
    Map(Map),
    /// Collection filter op
    Filter(Filter),
    /// Tests whether a predicate holds for at least one element of this collection
    Exists(Exists),
    /// Tests whether a predicate holds for all elements of this collection.
    ForAll(ForAll),
    /// Tuple field access
    SelectField(SelectField),
    /// Bool to SigmaProp
    BoolToSigmaProp(BoolToSigmaProp),
    /// Upcast numeric value
    Upcast(Upcast),
    /// Create proveDlog from GroupElement(PK)
    CreateProveDlog(CreateProveDlog),
    /// Create proveDlog from GroupElement(PK)
    CreateProveDhTuple(CreateProveDhTuple),
    /// Extract serialized bytes of a SigmaProp value
    SigmaPropBytes(SigmaPropBytes),
    /// Decode byte array to EC point
    DecodePoint(DecodePoint),
    /// AND conjunction for sigma propositions
    SigmaAnd(SigmaAnd),
    /// OR conjunction for sigma propositions
    SigmaOr(SigmaOr),
    /// Extracts Context variable by id and type
    GetVar(GetVar),
    /// Extract register of SELF box as `Coll[Byte]`, deserialize it into Value and inline into
    /// the executing script.
    DeserializeRegister(DeserializeRegister),
    /// Extracts context variable as `Coll[Byte]`, deserializes it to script and then executes
    /// this script in the current context. The original `Coll[Byte]` of the script is
    /// available as `getVar[Coll[Byte]](id)` On evaluation returns the result of the
    /// script execution in the current context
    DeserializeContext(DeserializeContext),
    /// MultiplyGroup op for GroupElement
    MultiplyGroup(MultiplyGroup),
}

impl Expr {
    /// Code (used in serialization)
    pub(crate) fn op_code(&self) -> OpCode {
        match self {
            Expr::Append(op) => op.op_code(),
            Expr::Const(_) => panic!("constant does not have op code assigned"),
            Expr::ConstPlaceholder(op) => op.op_code(),
            Expr::ByteArrayToLong(op) => op.op_code(),
            Expr::ByteArrayToBigInt(op) => op.op_code(),
            Expr::LongToByteArray(op) => op.op_code(),
            Expr::Collection(op) => op.op_code(),
            Expr::GlobalVars(op) => op.op_code(),
            Expr::MethodCall(op) => op.op_code(),
            Expr::ProperyCall(op) => op.op_code(),
            Expr::Global => OpCode::GLOBAL,
            Expr::Context => OpCode::CONTEXT,
            Expr::OptionGet(op) => op.op_code(),
            Expr::ExtractRegisterAs(op) => op.op_code(),
            Expr::BinOp(op) => op.op_code(),
            Expr::BlockValue(op) => op.op_code(),
            Expr::ValUse(op) => op.op_code(),
            Expr::FuncValue(op) => op.op_code(),
            Expr::Apply(op) => op.op_code(),
            Expr::ValDef(op) => op.op_code(),
            Expr::ExtractAmount(op) => op.op_code(),
            Expr::SelectField(op) => op.op_code(),
            Expr::Fold(op) => op.op_code(),
            Expr::CalcBlake2b256(op) => op.op_code(),
            Expr::CalcSha256(op) => op.op_code(),
            Expr::And(op) => op.op_code(),
            Expr::Or(op) => op.op_code(),
            Expr::Xor(op) => op.op_code(),
            Expr::Atleast(op) => op.op_code(),
            Expr::LogicalNot(op) => op.op_code(),
            Expr::Map(op) => op.op_code(),
            Expr::Filter(op) => op.op_code(),
            Expr::BoolToSigmaProp(op) => op.op_code(),
            Expr::Upcast(op) => op.op_code(),
            Expr::If(op) => op.op_code(),
            Expr::ByIndex(op) => op.op_code(),
            Expr::ExtractScriptBytes(op) => op.op_code(),
            Expr::SizeOf(op) => op.op_code(),
            Expr::Slice(op) => op.op_code(),
            Expr::CreateProveDlog(op) => op.op_code(),
            Expr::CreateProveDhTuple(op) => op.op_code(),
            Expr::ExtractCreationInfo(op) => op.op_code(),
            Expr::Exists(op) => op.op_code(),
            Expr::ExtractId(op) => op.op_code(),
            Expr::SigmaPropBytes(op) => op.op_code(),
            Expr::OptionIsDefined(op) => op.op_code(),
            Expr::OptionGetOrElse(op) => op.op_code(),
            Expr::Negation(op) => op.op_code(),
            Expr::ForAll(op) => op.op_code(),
            Expr::Tuple(op) => op.op_code(),
            Expr::DecodePoint(op) => op.op_code(),
            Expr::SigmaAnd(op) => op.op_code(),
            Expr::SigmaOr(op) => op.op_code(),
            Expr::GetVar(op) => op.op_code(),
            Expr::DeserializeRegister(op) => op.op_code(),
            Expr::DeserializeContext(op) => op.op_code(),
            Expr::MultiplyGroup(op) => op.op_code(),
        }
    }

    /// Type of the expression
    pub fn tpe(&self) -> SType {
        match self {
            Expr::Append(ap) => ap.tpe(),
            Expr::Const(v) => v.tpe.clone(),
            Expr::Collection(v) => v.tpe(),
            Expr::ByteArrayToLong(v) => v.tpe(),
            Expr::ByteArrayToBigInt(v) => v.tpe(),
            Expr::LongToByteArray(v) => v.tpe(),
            Expr::ConstPlaceholder(v) => v.tpe.clone(),
            Expr::CalcBlake2b256(v) => v.tpe(),
            Expr::CalcSha256(v) => v.tpe(),
            Expr::Global => SType::SGlobal,
            Expr::Context => SType::SContext,
            Expr::GlobalVars(v) => v.tpe(),
            Expr::FuncValue(v) => v.tpe(),
            Expr::Apply(v) => v.tpe(),
            Expr::MethodCall(v) => v.tpe(),
            Expr::ProperyCall(v) => v.tpe(),
            Expr::BlockValue(v) => v.tpe(),
            Expr::ValDef(v) => v.tpe(),
            Expr::ValUse(v) => v.tpe.clone(),
            Expr::BinOp(v) => v.tpe(),
            Expr::OptionGet(v) => v.tpe(),
            Expr::ExtractRegisterAs(v) => v.tpe(),
            Expr::Fold(v) => v.tpe(),
            Expr::SelectField(v) => v.tpe(),
            Expr::ExtractAmount(v) => v.tpe(),
            Expr::And(v) => v.tpe(),
            Expr::Or(v) => v.tpe(),
            Expr::Xor(v) => v.tpe(),
            Expr::Atleast(v) => v.tpe(),
            Expr::LogicalNot(v) => v.tpe(),
            Expr::Map(v) => v.tpe(),
            Expr::Filter(v) => v.tpe(),
            Expr::BoolToSigmaProp(v) => v.tpe(),
            Expr::Upcast(v) => v.tpe(),
            Expr::If(v) => v.tpe(),
            Expr::ByIndex(v) => v.tpe(),
            Expr::ExtractScriptBytes(v) => v.tpe(),
            Expr::SizeOf(v) => v.tpe(),
            Expr::Slice(v) => v.tpe(),
            Expr::CreateProveDlog(v) => v.tpe(),
            Expr::CreateProveDhTuple(v) => v.tpe(),
            Expr::ExtractCreationInfo(v) => v.tpe(),
            Expr::Exists(v) => v.tpe(),
            Expr::ExtractId(v) => v.tpe(),
            Expr::SigmaPropBytes(v) => v.tpe(),
            Expr::OptionIsDefined(v) => v.tpe(),
            Expr::OptionGetOrElse(v) => v.tpe(),
            Expr::Negation(v) => v.tpe(),
            Expr::ForAll(v) => v.tpe(),
            Expr::Tuple(v) => v.tpe(),
            Expr::DecodePoint(v) => v.tpe(),
            Expr::SigmaAnd(v) => v.tpe(),
            Expr::SigmaOr(v) => v.tpe(),
            Expr::DeserializeRegister(v) => v.tpe(),
            Expr::DeserializeContext(v) => v.tpe(),
            Expr::GetVar(v) => v.tpe(),
            Expr::MultiplyGroup(v) => v.tpe(),
        }
    }

    /// Type expected after the evaluation
    pub fn post_eval_tpe(&self) -> SType {
        match self.tpe() {
            SType::SFunc(sfunc) => *sfunc.t_range,
            tpe => tpe,
        }
    }

    /// Check if given expected_tpe type is the same as the expression's post-evaluation type
    pub fn check_post_eval_tpe(&self, expected_tpe: SType) -> Result<(), InvalidExprEvalTypeError> {
        let expr_tpe = self.post_eval_tpe();
        if expr_tpe == expected_tpe {
            Ok(())
        } else {
            Err(InvalidExprEvalTypeError(format!(
                "expected: {0:?}, got: {1:?}",
                expected_tpe, expr_tpe
            )))
        }
    }

    /// Prints the tree with newlines
    pub fn debug_tree(&self) -> String {
        let tree = format!("{:#?}", self);
        tree
    }
}

impl<T: Into<Value> + LiftIntoSType> From<T> for Expr {
    fn from(t: T) -> Self {
        Expr::Const(Constant {
            tpe: T::stype(),
            v: t.into(),
        })
    }
}

/// Unexpected argument on node construction (i.e non-Option input in OptionGet)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct InvalidArgumentError(pub String);

/// Invalid (unexpected) expr type
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct InvalidExprEvalTypeError(pub String);

impl From<InvalidExprEvalTypeError> for InvalidArgumentError {
    fn from(e: InvalidExprEvalTypeError) -> Self {
        InvalidArgumentError(format!("{0:?}", e))
    }
}

impl From<BoundedVecOutOfBounds> for InvalidArgumentError {
    fn from(e: BoundedVecOutOfBounds) -> Self {
        InvalidArgumentError(format!("{0:?}", e))
    }
}

impl<T: TryFrom<Expr>> TryExtractFrom<Expr> for T {
    fn try_extract_from(v: Expr) -> Result<Self, TryExtractFromError> {
        let res: Result<Self, TryExtractFromError> = v.clone().try_into().map_err(|_| {
            TryExtractFromError(format!(
                "Cannot extract {0:?} from {1:?}",
                std::any::type_name::<T>(),
                v
            ))
        });
        res
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
/// Arbitrary impl
pub(crate) mod arbitrary {
    use super::*;
    use crate::mir::func_value::FuncArg;
    use crate::sigma_protocol::sigma_boolean::ProveDlog;
    use crate::types::sfunc::SFunc;
    use proptest::collection::*;
    use proptest::prelude::*;

    /// Parameters for arbitrary Expr generation
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct ArbExprParams {
        /// Expr type
        pub tpe: SType,
        /// Expr tree depth (levels)
        pub depth: usize,
    }

    impl Default for ArbExprParams {
        fn default() -> Self {
            ArbExprParams {
                tpe: SType::SBoolean,
                depth: 1,
            }
        }
    }

    fn numeric_nested_expr(depth: usize, elem_tpe: &SType) -> BoxedStrategy<Expr> {
        prop_oneof![any_with::<BinOp>(ArbExprParams {
            tpe: elem_tpe.clone(),
            depth
        })
        .prop_map_into(),]
        .boxed()
    }

    fn bool_nested_expr(depth: usize) -> BoxedStrategy<Expr> {
        prop_oneof![
            any_with::<BinOp>(ArbExprParams {
                tpe: SType::SBoolean,
                depth
            })
            .prop_map_into(),
            any_with::<And>(depth).prop_map_into(),
            any_with::<Or>(depth).prop_map_into(),
            any_with::<LogicalNot>(depth).prop_map_into(),
        ]
        .boxed()
    }

    fn coll_nested_numeric(depth: usize, elem_tpe: &SType) -> BoxedStrategy<Expr> {
        let ty = elem_tpe.clone();
        vec(numeric_nested_expr(depth, elem_tpe), 0..10)
            .prop_map(move |items| Collection::new(ty.clone(), items).unwrap())
            .prop_map_into()
            .boxed()
    }

    fn sigma_prop_nested_expr(_depth: usize) -> BoxedStrategy<Expr> {
        prop_oneof![
            any::<ProveDlog>().prop_map(|pk| Expr::Const(pk.into())),
            any::<SigmaAnd>().prop_map_into(),
            any::<SigmaOr>().prop_map_into(),
        ]
        .boxed()
    }

    fn coll_nested_expr(depth: usize, elem_tpe: &SType) -> BoxedStrategy<Expr> {
        match elem_tpe {
            SType::SBoolean => vec(bool_nested_expr(depth), 0..10)
                .prop_map(|items| Collection::new(SType::SBoolean, items).unwrap())
                .prop_map_into()
                .boxed(),
            SType::SByte => coll_nested_numeric(depth, elem_tpe),
            SType::SShort => coll_nested_numeric(depth, elem_tpe),
            SType::SInt => coll_nested_numeric(depth, elem_tpe),
            SType::SLong => coll_nested_numeric(depth, elem_tpe),
            SType::SBigInt => coll_nested_numeric(depth, elem_tpe),

            SType::STypeVar(_) => prop_oneof![
                vec(bool_nested_expr(depth), 0..10).prop_map(|items| Collection::new(
                    SType::SBoolean,
                    items
                )
                .unwrap()),
                vec(numeric_nested_expr(depth, &SType::SInt), 0..10)
                    .prop_map(|items| Collection::new(SType::SInt, items).unwrap())
            ]
            .prop_map_into()
            .boxed(),
            SType::SSigmaProp => sigma_prop_nested_expr(depth),
            _ => panic!("Nested expression not implemented for {:?}", &elem_tpe),
        }
    }

    fn any_nested_expr(depth: usize) -> BoxedStrategy<Expr> {
        prop_oneof![
            bool_nested_expr(depth),
            numeric_nested_expr(depth, &SType::SByte),
            numeric_nested_expr(depth, &SType::SShort),
            numeric_nested_expr(depth, &SType::SInt),
            numeric_nested_expr(depth, &SType::SLong),
            numeric_nested_expr(depth, &SType::SBigInt),
        ]
        .boxed()
    }

    fn nested_expr(tpe: SType, depth: usize) -> BoxedStrategy<Expr> {
        match tpe {
            SType::SAny => any_nested_expr(depth),
            SType::SBoolean => bool_nested_expr(depth),
            SType::SByte => numeric_nested_expr(depth, &tpe),
            SType::SShort => numeric_nested_expr(depth, &tpe),
            SType::SInt => numeric_nested_expr(depth, &tpe),
            SType::SLong => numeric_nested_expr(depth, &tpe),
            SType::SBigInt => numeric_nested_expr(depth, &tpe),
            SType::SColl(elem_type) => coll_nested_expr(depth, elem_type.as_ref()),
            SType::SSigmaProp => sigma_prop_nested_expr(depth),
            _ => todo!("nested expr is not implemented for type: {:?}", tpe),
        }
        .boxed()
    }

    fn int_non_nested_expr() -> BoxedStrategy<Expr> {
        prop_oneof![Just(GlobalVars::Height.into()),].boxed()
    }

    fn constant(tpe: &SType) -> BoxedStrategy<Expr> {
        any_with::<Constant>(tpe.clone().into())
            .prop_map_into()
            .boxed()
    }

    fn bool_non_nested_expr() -> BoxedStrategy<Expr> {
        prop_oneof![any_with::<Constant>(SType::SBoolean.into()).prop_map_into()].boxed()
    }

    fn any_non_nested_expr() -> BoxedStrategy<Expr> {
        prop_oneof![int_non_nested_expr(), bool_non_nested_expr()].boxed()
    }

    fn coll_non_nested_expr(elem_tpe: &SType) -> BoxedStrategy<Expr> {
        match elem_tpe {
            SType::SByte => any_with::<Constant>(SType::SColl(Box::new(SType::SByte)).into())
                .prop_map(Expr::Const)
                .boxed(),
            SType::SBoolean => any_with::<Constant>(SType::SColl(Box::new(SType::SBoolean)).into())
                .prop_map(Expr::Const)
                .boxed(),
            _ => todo!("Collection of {0:?} is not yet implemented", elem_tpe),
        }
    }

    fn non_nested_expr(tpe: &SType) -> BoxedStrategy<Expr> {
        match tpe {
            SType::SAny => any_non_nested_expr(),
            SType::SInt => int_non_nested_expr(),
            SType::SBoolean => bool_non_nested_expr(),
            SType::SColl(elem_type) => coll_non_nested_expr(elem_type),
            t => constant(t),
        }
    }

    fn sfunc_expr(sfunc: SFunc) -> BoxedStrategy<Expr> {
        match (sfunc.t_dom.first().unwrap(), *sfunc.t_range) {
            (SType::SBoolean, SType::SBoolean) => any_with::<Expr>(ArbExprParams {
                tpe: SType::SBoolean,
                depth: 2,
            })
            .prop_map(|expr| {
                Expr::FuncValue(FuncValue::new(
                    vec![FuncArg {
                        idx: 1.into(),
                        tpe: SType::SBoolean,
                    }],
                    expr,
                ))
            })
            .boxed(),
            _ => todo!(),
        }
    }

    impl Arbitrary for Expr {
        type Parameters = ArbExprParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            if args.depth == 0 {
                match args.tpe {
                    SType::SFunc(sfunc) => sfunc_expr(sfunc),
                    _ => prop_oneof![
                        any_with::<Constant>(args.tpe.clone().into())
                            .prop_map(Expr::Const)
                            .boxed(),
                        non_nested_expr(&args.tpe)
                    ]
                    .boxed(),
                }
            } else {
                nested_expr(args.tpe, args.depth - 1)
            }
        }
    }
}

#[cfg(test)]
mod tests {}
