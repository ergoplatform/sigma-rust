use ergotree_ir::{
    mir::{
        bool_to_sigma::BoolToSigmaProp,
        expr::Expr,
        func_value::{FuncArg, FuncValue},
        select_field::{SelectField, TupleFieldIndex},
        val_def::{ValId, ValDef},
        val_use::ValUse, bin_op::{BinOp, ArithOp}, constant::Constant, tuple::Tuple, block::BlockValue,
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
fn test_simple_arithmetic() {
    let e = ergo_tree!(FuncValue(
        Vector((1, SInt)), 
        ArithOp(ValUse(1, SInt), IntConstant(-345), OpCode @@ (-102.toByte))
    ));

    let left = Expr::ValUse(ValUse {
        val_id: ValId(1),
        tpe: SType::SInt,
    })
    .into();
    let right = Expr::Const(Constant::from(-345_i32)).into();
    let body = Expr::BinOp(BinOp {
        kind: ergotree_ir::mir::bin_op::BinOpKind::Arith(ergotree_ir::mir::bin_op::ArithOp::Plus),
        left,
        right,
    });
    let args = vec![FuncArg {
        idx: ValId(1),
        tpe: SType::SInt,
    }];
    let expected = Expr::FuncValue(FuncValue::new(args, body));
    assert_eq!(e, expected);
}

#[test]
fn test_arithmetic_in_block() {
    // This example comes from the scala JIT test suite:
    // { (x: (Byte, Byte)) =>
    //     val a = x._1; val b = x._2
    //     val plus = a + b
    //     val minus = a - b
    //     val mul = a * b
    //     val div = a / b
    //     val mod = a % b
    //     (plus, (minus, (mul, (div, mod))))
    // }
    let e = ergo_tree!(
        FuncValue(
            Vector((1, STuple(Vector(SByte, SByte)))),
            BlockValue(
              Vector(
                ValDef(
                  3,
                  List(),
                  SelectField.typed[ByteValue](ValUse(1, STuple(Vector(SByte, SByte))), 1.toByte)
                ),
                ValDef(
                  4,
                  List(),
                  SelectField.typed[ByteValue](ValUse(1, STuple(Vector(SByte, SByte))), 2.toByte)
                )
              ),
              Tuple(
                Vector(
                  ArithOp(ValUse(3, SByte), ValUse(4, SByte), OpCode @@ (-102.toByte)),
                  Tuple(
                    Vector(
                      ArithOp(ValUse(3, SByte), ValUse(4, SByte), OpCode @@ (-103.toByte)),
                      Tuple(
                        Vector(
                          ArithOp(ValUse(3, SByte), ValUse(4, SByte), OpCode @@ (-100.toByte)),
                          Tuple(
                            Vector(
                              ArithOp(ValUse(3, SByte), ValUse(4, SByte), OpCode @@ (-99.toByte)),
                              ArithOp(ValUse(3, SByte), ValUse(4, SByte), OpCode @@ (-98.toByte))
                            )
                          )
                        )
                      )
                    )
                  )
                )
              )
            )
          )
    );

    let items = vec![
        ValDef { 
            id: ValId(3),
            rhs: Expr::SelectField(
                SelectField::new( 
                    Expr::ValUse(ValUse { val_id: ValId(1), tpe: SType::STuple(STuple::pair(SType::SByte, SType::SByte)) }), 
                    TupleFieldIndex::try_from(1).unwrap()
                ).unwrap()).into() 
        }.into(),
        ValDef { 
            id: ValId(4),
            rhs: Expr::SelectField(
                SelectField::new( 
                    Expr::ValUse(ValUse { val_id: ValId(1), tpe: SType::STuple(STuple::pair(SType::SByte, SType::SByte)) }), 
                    TupleFieldIndex::try_from(2).unwrap()
                ).unwrap()).into() 
        }.into(),
    ];

    let val_use3: Box<Expr> = Expr::ValUse(ValUse {
        val_id: ValId(3),
        tpe: SType::SByte,
    }).into();
    let val_use4: Box<Expr> = Expr::ValUse(ValUse {
        val_id: ValId(4),
        tpe: SType::SByte,
    }).into();

    let make_def = |op| {
        Expr::BinOp(BinOp { 
            kind: ergotree_ir::mir::bin_op::BinOpKind::Arith(op),
            left: val_use3.clone(), 
            right: val_use4.clone(),
        })
    };

    let plus = make_def(ArithOp::Plus);
    let minus = make_def(ArithOp::Minus);
    let mul = make_def(ArithOp::Multiply);
    let div = make_def(ArithOp::Divide);
    let modulo = make_def(ArithOp::Modulo);

    let t3 = Expr::Tuple(Tuple::new(vec![ div, modulo ]).unwrap());
    let t2 = Expr::Tuple(Tuple::new(vec![ mul, t3 ]).unwrap());
    let t1 = Expr::Tuple(Tuple::new(vec![ minus, t2 ]).unwrap());
    let result = Expr::Tuple(Tuple::new(vec![ plus, t1 ]).unwrap()).into();
    
    let args = vec![FuncArg {
        idx: ValId(1),
        tpe: SType::STuple(STuple::pair(SType::SByte, SType::SByte)),
    }];
    let body = Expr::BlockValue(BlockValue { items, result});
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
