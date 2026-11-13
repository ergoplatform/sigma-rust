use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_param::STypeVar;
use crate::types::smethod::MethodId;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;

/// SOption type code
pub const TYPE_CODE: TypeCode = TypeCode::OPTION;
/// SOption type name
pub static TYPE_NAME: &str = "Option";
/// Option.map
pub const MAP_METHOD_ID: MethodId = MethodId(7);
/// Option.filter
pub const FILTER_METHOD_ID: MethodId = MethodId(8);

lazy_static! {
    /// Option method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> =
        vec![
            MAP_METHOD_DESC.clone(),
            FILTER_METHOD_DESC.clone(),
        ]
    ;
}

lazy_static! {
    static ref MAP_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: MAP_METHOD_ID,
        name: "map",
        tpe: SFunc::new(
            vec![
                SType::SOption(SType::STypeVar(STypeVar::t()).into()),
                SFunc::new(
                    vec![STypeVar::t().into()],
                    STypeVar::r().into(),
                ).into()
                ],
            SType::SOption(SType::STypeVar(STypeVar::r()).into()),
        ),
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Option.map
    pub static ref MAP_METHOD: SMethod = SMethod::new(
         STypeCompanion::Option,
         MAP_METHOD_DESC.clone());
}

lazy_static! {
    static ref FILTER_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: FILTER_METHOD_ID,
        name: "filter",
        tpe: SFunc::new(
            vec![
                SType::SOption(SType::STypeVar(STypeVar::t()).into()),
                SFunc::new(
                    vec![STypeVar::t().into()],
                    SType::SBoolean,
                ).into()
                ],
            SType::SOption(SType::STypeVar(STypeVar::t()).into()),
        ),
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Option.filter
    pub static ref FILTER_METHOD: SMethod = SMethod::new(
         STypeCompanion::Option,
         FILTER_METHOD_DESC.clone());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::bin_op::BinOp;
    use crate::mir::bin_op::RelationOp;
    use crate::mir::constant::Constant;
    use crate::mir::expr::Expr;
    use crate::mir::func_value::FuncArg;
    use crate::mir::func_value::FuncValue;
    use crate::mir::method_call::MethodCall;
    use crate::mir::val_use::ValUse;
    use hashbrown::HashMap;

    fn contains_type_var(tpe: &SType, var: &STypeVar) -> bool {
        match tpe {
            SType::STypeVar(v) => v == var,
            SType::SOption(elem) => contains_type_var(elem, var),
            SType::SColl(elem) => contains_type_var(elem, var),
            SType::STuple(tuple) => tuple.items.iter().any(|item| contains_type_var(item, var)),
            SType::SFunc(func) => {
                func.t_dom.iter().any(|item| contains_type_var(item, var))
                    || contains_type_var(&func.t_range, var)
            }
            _ => false,
        }
    }

    fn gt_body() -> Expr {
        BinOp {
            kind: RelationOp::Gt.into(),
            left: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SLong,
                }
                .into(),
            ),
            right: Box::new(Expr::Const(0i64.into())),
        }
        .into()
    }

    #[test]
    fn method_descs_use_scala_type_var_names() {
        // sigmastate-interpreter's SOptionMethods names the option element
        // type T (and R for map's result type); no Option descriptor uses
        // the IV/OV variables (same naming bug class as issue #642 for Coll).
        for desc in METHOD_DESC.iter() {
            for var in [STypeVar::iv(), STypeVar::ov()] {
                assert!(
                    !contains_type_var(&SType::SFunc(desc.tpe.clone()), &var),
                    "Option.{} descriptor uses STypeVar {} instead of T/R: {:?}",
                    desc.name,
                    var.as_string(),
                    desc.tpe
                );
            }
        }
    }

    #[test]
    fn map_specialized_with_scala_type_var_names() {
        // { (x: Option[Long]) => x.map((v: Long) => v > 0) } with the
        // concrete types keyed by the Scala variable names T and R.
        let subst: HashMap<STypeVar, SType> = [
            (STypeVar::t(), SType::SLong),
            (STypeVar::r(), SType::SBoolean),
        ]
        .iter()
        .cloned()
        .collect();
        let obj: Expr = Constant::from(Some(1i64)).into();
        let arg: Expr = FuncValue::new(
            vec![FuncArg {
                idx: 1.into(),
                tpe: SType::SLong,
            }],
            gt_body(),
        )
        .into();
        let mc = MethodCall::new(
            obj,
            MAP_METHOD.clone().with_concrete_types(&subst),
            vec![arg],
        )
        .unwrap();
        assert_eq!(mc.tpe(), SType::SOption(SType::SBoolean.into()));
    }

    #[test]
    fn filter_specialized_with_scala_type_var_names() {
        // { (x: Option[Long]) => x.filter((v: Long) => v > 0) } with the
        // concrete type keyed by the Scala variable name T.
        let subst: HashMap<STypeVar, SType> =
            [(STypeVar::t(), SType::SLong)].iter().cloned().collect();
        let obj: Expr = Constant::from(Some(1i64)).into();
        let arg: Expr = FuncValue::new(
            vec![FuncArg {
                idx: 1.into(),
                tpe: SType::SLong,
            }],
            gt_body(),
        )
        .into();
        let mc = MethodCall::new(
            obj,
            FILTER_METHOD.clone().with_concrete_types(&subst),
            vec![arg],
        )
        .unwrap();
        assert_eq!(mc.tpe(), SType::SOption(SType::SLong.into()));
    }

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_CODE, MAP_METHOD_ID).map(|e| e.name()) == Ok("map"));
        assert!(SMethod::from_ids(TYPE_CODE, FILTER_METHOD_ID).map(|e| e.name()) == Ok("filter"));
    }
}
