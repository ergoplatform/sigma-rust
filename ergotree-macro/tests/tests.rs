use ergotree_ir::{
    mir::{
        bool_to_sigma::BoolToSigmaProp,
        expr::Expr,
        func_value::{FuncArg, FuncValue},
        select_field::{SelectField, TupleFieldIndex},
        val_def::ValId,
        val_use::ValUse,
    },
    types::{stuple::STuple, stype::SType},
};
use ergotree_macro::ergo_tree;
use paste::paste;

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
fn test_tuple_select_field_simple() {
    let e = ergo_tree!(FuncValue(
        Vector((1, STuple(Vector(SBoolean, SBoolean)))),
        SelectField.typed[BoolValue](ValUse(1, STuple(Vector(SBoolean, SBoolean))), 1.toByte)
    ));

    let input = Expr::ValUse(ValUse {
        val_id: ValId(1),
        tpe: SType::STuple(STuple::pair(SType::SBoolean, SType::SBoolean)),
    });
    let body = Expr::SelectField(
        SelectField::new(input, TupleFieldIndex::try_from(1_u8).unwrap()).unwrap(),
    );
    let args = vec![FuncArg {
        idx: ValId(1),
        tpe: SType::STuple(STuple::pair(SType::SBoolean, SType::SBoolean)),
    }];
    let expected = Expr::FuncValue(FuncValue::new(args, body));
    assert_eq!(e, expected);
}

#[test]
fn test_tuple_select_field_with_coll() {
    let e = ergo_tree!(FuncValue(
        Vector((1, STuple(Vector(SCollectionType(SByte), SBoolean)))),
        SelectField.typed[Value[
            SCollection[ SInt.type ]
        ]](
            ValUse(1, STuple(Vector(SCollectionType(SByte), SBoolean))),
            1.toByte
        )
    ));

    let input = Expr::ValUse(ValUse {
        val_id: ValId(1),
        tpe: SType::STuple(STuple::pair(
            SType::SColl(SType::SByte.into()),
            SType::SBoolean,
        )),
    });
    let body = Expr::SelectField(
        SelectField::new(input, TupleFieldIndex::try_from(1_u8).unwrap()).unwrap(),
    );
    let args = vec![FuncArg {
        idx: ValId(1),
        tpe: SType::STuple(STuple::pair(
            SType::SColl(SType::SByte.into()),
            SType::SBoolean,
        )),
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

#[test]
fn test_nested_tuples() {
    // For the following ergoscript:
    //  { (t: (Boolean, (Int, Long))) => t._2 }
    let e = ergo_tree!(FuncValue(
        Vector((1, STuple(Vector(SBoolean, STuple(Vector(SInt, SLong)))))),
        SelectField.typed[Value[STuple]](
            ValUse(1, STuple(Vector(SBoolean, STuple(Vector(SInt, SLong))))),
            2.toByte
        )
    ));

    let input = Expr::ValUse(ValUse {
        val_id: ValId(1),
        tpe: SType::STuple(STuple::pair(
            SType::SBoolean,
            SType::STuple(STuple::pair(SType::SInt, SType::SLong)),
        )),
    });
    let body = Expr::SelectField(
        SelectField::new(input, TupleFieldIndex::try_from(2_u8).unwrap()).unwrap(),
    );
    let args = vec![FuncArg {
        idx: ValId(1),
        tpe: SType::STuple(STuple::pair(
            SType::SBoolean,
            SType::STuple(STuple::pair(SType::SInt, SType::SLong)),
        )),
    }];
    let expected = Expr::FuncValue(FuncValue::new(args, body));
    assert_eq!(e, expected);
}

/// This macro creates a unit test for parsing and tokenizing the following ergoscript:
///   { (x: $type_name) -> x }
macro_rules! identity_fn {
    ($type_name:ident) => {
        paste! {
            #[test]
            fn [<test_identity_ $type_name:snake>]() {
                let e = ergo_tree!(FuncValue(
                    Vector((1, $type_name)),
                    ValUse(1, $type_name)
                ));
                let args = vec![FuncArg {
                    idx: ValId(1),
                    tpe: SType::$type_name,
                }];
                let body = Expr::ValUse(ValUse {
                    val_id: ValId(1),
                    tpe: SType::$type_name,
                });
                let expected = Expr::FuncValue(FuncValue::new(args, body));
                assert_eq!(e, expected);
            }
        }
    };
}

identity_fn!(SAny);
identity_fn!(SUnit);
identity_fn!(SBoolean);
identity_fn!(SShort);
identity_fn!(SInt);
identity_fn!(SLong);
identity_fn!(SBigInt);
identity_fn!(SGroupElement);
identity_fn!(SSigmaProp);
identity_fn!(SBox);
identity_fn!(SAvlTree);
identity_fn!(SContext);
identity_fn!(SHeader);
identity_fn!(SPreHeader);
identity_fn!(SGlobal);

/// This macro creates a unit test for parsing and tokenizing the following ergoscript:
///   { (x: Coll[$type_name]) -> x }
macro_rules! identity_fn_coll {
    ($type_name:ident) => {
        paste! {
            #[test]
            fn [<test_identity_coll_ $type_name:snake>]() {
                let e = ergo_tree!(FuncValue(
                    Vector((1, SCollectionType($type_name))),
                    ValUse(1, SCollectionType($type_name))
                ));
                let args = vec![FuncArg {
                    idx: ValId(1),
                    tpe: SType::SColl(SType::$type_name.into()),
                }];
                let body = Expr::ValUse(ValUse {
                    val_id: ValId(1),
                    tpe: SType::SColl(SType::$type_name.into()),
                });
                let expected = Expr::FuncValue(FuncValue::new(args, body));
                assert_eq!(e, expected);
            }
        }
    };
}

identity_fn_coll!(SAny);
identity_fn_coll!(SUnit);
identity_fn_coll!(SBoolean);
identity_fn_coll!(SShort);
identity_fn_coll!(SInt);
identity_fn_coll!(SLong);
identity_fn_coll!(SBigInt);
identity_fn_coll!(SGroupElement);
identity_fn_coll!(SSigmaProp);
identity_fn_coll!(SBox);
identity_fn_coll!(SAvlTree);
identity_fn_coll!(SContext);
identity_fn_coll!(SHeader);
identity_fn_coll!(SPreHeader);
identity_fn_coll!(SGlobal);
