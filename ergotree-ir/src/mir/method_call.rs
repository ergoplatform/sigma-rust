use crate::serialization::op_code::OpCode;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

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
    pub args: Vec<Expr>,
}

impl MethodCall {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(obj: Expr, method: SMethod, args: Vec<Expr>) -> Result<Self, InvalidArgumentError> {
        if method.tpe().t_dom.len() != args.len() + 1 {
            return Err(InvalidArgumentError(format!(
                "MethodCall: expected arguments count {} does not match provided arguments count {}",
                method.tpe().t_dom.len(), args.len() + 1)));
        }
        let mut expected_types = vec![obj.tpe()];
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
                "MethodCall: expected types {:?} do not match provided obj and args types {:?}",
                method.tpe().t_dom,
                expected_types
            )));
        }
        Ok(Self {
            obj: obj.into(),
            method,
            args,
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        *self.method.tpe().t_range.clone()
    }

    pub(crate) fn op_code(&self) -> OpCode {
        OpCode::METHOD_CALL
    }
}
