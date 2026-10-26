use hashbrown::HashMap;

use alloc::boxed::Box;

use alloc::vec;
use alloc::vec::Vec;

use crate::serialization::op_code::OpCode;
use crate::traversable::impl_traversable_expr;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;
use crate::types::stype_param::STypeVar;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/** Represents in ErgoTree an invocation of method of the object `obj` with arguments `args`.
 * The SMethod instances in STypeCompanions may have type STypeIdent in methods types,
 * but valid ErgoTree should have SMethod instances specialized for specific types of
 * obj and args using `specializeFor`.
 * This means, if we save typeId, methodId, and we save all the arguments,
 * we can restore the specialized SMethod instance.
 * This work by induction, if we assume all arguments are monomorphic,
 * then we can make MethodCall monomorphic.
 * Thus, all ErgoTree instances are monomorphic by construction.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodCall {
    /// Object on which method will be invoked
    pub obj: Box<Expr>,
    /// Method to be invoked
    pub method: SMethod,
    /// Arguments passed to the method on invocation
    pub args: Box<[Expr]>,
    /// Arguments that cannot be inferred from function signature, such as Box.getReg[T]()
    pub explicit_type_args: HashMap<STypeVar, SType>,
}

impl MethodCall {
    fn new_inner(
        method: SMethod,
        args: Vec<Expr>,
        obj: Expr,
        explicit_type_args: HashMap<STypeVar, SType>,
    ) -> Result<MethodCall, InvalidArgumentError> {
        if method.tpe().t_dom.len() != args.len() + 1 {
            return Err(InvalidArgumentError(format!(
                "MethodCall: expected arguments count {} does not match provided arguments count {}",
                method.tpe().t_dom.len(), args.len() + 1)));
        }
        if method.method_raw.explicit_type_args.len() != explicit_type_args.len() {
            return Err(InvalidArgumentError(format!("MethodCall: expected explicit type args count {} does not match provided type args count {}",
                method.method_raw.explicit_type_args.len(), explicit_type_args.len())));
        }
        if let Some(missing_tpe) = method
            .method_raw
            .explicit_type_args
            .iter()
            .find(|&tpe| !explicit_type_args.contains_key(tpe))
        {
            return Err(InvalidArgumentError(format!(
                "MethodCall: explicit_type_args does not include substitution for STypeVar {missing_tpe:?}",
            )));
        }
        let mut expected_types: Vec<SType> = vec![obj.tpe()];
        let arg_types: Vec<SType> = args.clone().into_iter().map(|a| a.tpe()).collect();
        expected_types.extend(arg_types);
        if !method
            .tpe()
            .t_dom
            .iter()
            .zip(&expected_types)
            .all(|(expected, actual)| expected == actual)
        {
            return Err(InvalidArgumentError(format!(
                "MethodCall: expected types {expected_types:?} do not match provided obj and args types {:?}",
                method.tpe().t_dom,
            )));
        }
        Ok(Self {
            obj: obj.into(),
            method: method.with_concrete_types(&explicit_type_args),
            args: args.into_boxed_slice(),
            explicit_type_args,
        })
    }

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(obj: Expr, method: SMethod, args: Vec<Expr>) -> Result<Self, InvalidArgumentError> {
        MethodCall::new_inner(method, args, obj, Default::default())
    }

    /// Create new object with explicit type args
    pub fn with_type_args(
        obj: Expr,
        method: SMethod,
        args: Vec<Expr>,
        type_args: HashMap<STypeVar, SType>,
    ) -> Result<Self, InvalidArgumentError> {
        MethodCall::new_inner(method, args, obj, type_args)
    }

    /// Type
    pub fn tpe(&self) -> SType {
        *self.method.tpe().t_range.clone()
    }
}

impl HasStaticOpCode for MethodCall {
    const OP_CODE: OpCode = OpCode::METHOD_CALL;
}

impl_traversable_expr!(MethodCall, boxed obj, arr args);
