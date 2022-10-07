use ergotree_ir::{
    mir::{
        bool_to_sigma::BoolToSigmaProp,
        expr::Expr,
        func_value::{FuncArg, FuncValue},
        val_def::ValId,
        val_use::ValUse,
    },
    types::{stuple::STuple, stype::SType},
};
use ergotree_macro::ergo_tree;

#[test]
fn test_stuple() {
    let e = ergo_tree!(FuncValue(
        Vector((1, STuple(Vector(SBoolean, SBoolean)))),
        ValUse(1, STuple(Vector(SBoolean, SBoolean)))
    ));

    let body = Expr::ValUse(ValUse {
        val_id: ValId(1),
        tpe: SType::STuple(STuple::pair(SType::SBoolean, SType::SBoolean)),
    });
    let args = vec![FuncArg {
        idx: ValId(1),
        tpe: SType::STuple(STuple::pair(SType::SBoolean, SType::SBoolean)),
    }];
    let expected = Expr::FuncValue(FuncValue::new(args, body));
    assert_eq!(e, expected);
}

#[test]
fn test_lambda_0() {
    let e = ergo_tree!(FuncValue(
        Vector((1, SBoolean)),
        BoolToSigmaProp(ValUse(1, SBoolean))
    ));

    let input = Expr::ValUse(ValUse {
        val_id: ValId(1),
        tpe: SType::SBoolean,
    })
    .into();
    let args = vec![FuncArg {
        idx: ValId(1),
        tpe: SType::SBoolean,
    }];
    let body = Expr::BoolToSigmaProp(BoolToSigmaProp { input });
    let expected = Expr::FuncValue(FuncValue::new(args, body));
    assert_eq!(e, expected);
}
