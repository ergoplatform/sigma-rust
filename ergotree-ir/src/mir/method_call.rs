use crate::serialization::op_code::OpCode;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;

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
    /// Type
    pub fn tpe(&self) -> SType {
        match self.method.tpe() {
            SType::SFunc(sfunc) => *sfunc.t_range.clone(),
            tpe => tpe.clone(),
        }
    }

    pub(crate) fn op_code(&self) -> OpCode {
        OpCode::METHOD_CALL
    }
}
