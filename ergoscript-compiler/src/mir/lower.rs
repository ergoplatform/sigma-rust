use ergotree_ir::mir::bin_op::ArithOp;
use ergotree_ir::mir::bin_op::BinOp;
use ergotree_ir::mir::bin_op::BinOpKind;
use ergotree_ir::mir::bin_op::LogicalOp;
use ergotree_ir::mir::bin_op::RelationOp;
use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
use ergotree_ir::mir::byte_array_to_bigint::ByteArrayToBigInt;
use ergotree_ir::mir::byte_array_to_long::ByteArrayToLong;
use ergotree_ir::mir::calc_blake2b256::CalcBlake2b256;
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::coll_exists::Exists;
use ergotree_ir::mir::coll_filter::Filter;
use ergotree_ir::mir::coll_fold::Fold;
use ergotree_ir::mir::coll_forall::ForAll;
use ergotree_ir::mir::coll_map::Map;
use ergotree_ir::mir::coll_size::SizeOf;
use ergotree_ir::mir::coll_slice::Slice;
use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::create_provedlog::CreateProveDlog;
use ergotree_ir::mir::decode_point::DecodePoint;
use ergotree_ir::mir::downcast::Downcast;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::extract_amount::ExtractAmount;
use ergotree_ir::mir::extract_bytes::ExtractBytes;
use ergotree_ir::mir::extract_creation_info::ExtractCreationInfo;
use ergotree_ir::mir::extract_id::ExtractId;
use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
use ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes;
use ergotree_ir::mir::func_value::FuncArg;
use ergotree_ir::mir::func_value::FuncValue;
use ergotree_ir::mir::get_var::GetVar;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::if_op::If;
use ergotree_ir::mir::logical_not::LogicalNot;
use ergotree_ir::mir::long_to_byte_array::LongToByteArray;
use ergotree_ir::mir::method_call::MethodCall;
use ergotree_ir::mir::negation::Negation;
use ergotree_ir::mir::option_get::OptionGet;
use ergotree_ir::mir::option_is_defined::OptionIsDefined;
use ergotree_ir::mir::property_call::PropertyCall;
use ergotree_ir::mir::select_field::SelectField;
use ergotree_ir::mir::select_field::TupleFieldIndex;
use ergotree_ir::mir::sigma_and::SigmaAnd;
use ergotree_ir::mir::sigma_or::SigmaOr;
use ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes;
use ergotree_ir::mir::subst_const::SubstConstants;
use ergotree_ir::mir::tree_lookup::TreeLookup;
use ergotree_ir::mir::tuple::Tuple;
use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
use ergotree_ir::mir::upcast::Upcast;
use ergotree_ir::mir::val_def::ValDef;
use ergotree_ir::mir::val_def::ValId;
use ergotree_ir::mir::val_use::ValUse;
use ergotree_ir::mir::xor::Xor;
use ergotree_ir::mir::xor_of::XorOf;
use ergotree_ir::types::stuple::STuple;
use ergotree_ir::types::stype::SType;
use hir::BinaryOp;
use rowan::TextRange;

use crate::error::pretty_error_desc;
use crate::hir;

#[derive(Debug, PartialEq, Eq)]
pub struct MirLoweringError {
    msg: String,
    span: TextRange,
}

impl MirLoweringError {
    pub fn new(msg: String, span: TextRange) -> Self {
        Self { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

pub fn lower(hir_expr: hir::Expr) -> Result<Expr, MirLoweringError> {
    let mir: Expr = match &hir_expr.kind {
        hir::ExprKind::GlobalVars(hir) => match hir {
            hir::GlobalVars::Height => GlobalVars::Height.into(),
            hir::GlobalVars::SelfBox => GlobalVars::SelfBox.into(),
            hir::GlobalVars::Inputs => GlobalVars::Inputs.into(),
            hir::GlobalVars::Outputs => GlobalVars::Outputs.into(),
        },
        hir::ExprKind::Ident(_) => {
            return Err(MirLoweringError::new(
                format!("MIR error: Unresolved Ident {0:?}", hir_expr),
                hir_expr.span,
            ))
        }
        hir::ExprKind::Binary(hir) => {
            let l = lower(*hir.lhs.clone())?;
            let r = lower(*hir.rhs.clone())?;
            // SigmaProp-level && / || → SigmaAnd / SigmaOr
            // If either side is SigmaProp, auto-promote the Bool side via BoolToSigmaProp
            if matches!(hir.op.node, BinaryOp::And | BinaryOp::Or) {
                let l_sigma = l.tpe() == SType::SSigmaProp;
                let r_sigma = r.tpe() == SType::SSigmaProp;
                if l_sigma || r_sigma {
                    let l = if !l_sigma {
                        Expr::BoolToSigmaProp(BoolToSigmaProp::try_build(l).map_err(|e| {
                            MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                        })?)
                    } else {
                        l
                    };
                    let r = if !r_sigma {
                        Expr::BoolToSigmaProp(BoolToSigmaProp::try_build(r).map_err(|e| {
                            MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                        })?)
                    } else {
                        r
                    };
                    return Ok(match hir.op.node {
                        BinaryOp::And => SigmaAnd::new(vec![l, r])
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into(),
                        BinaryOp::Or => SigmaOr::new(vec![l, r])
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into(),
                        _ => unreachable!(),
                    });
                }
            }
            {
                BinOp {
                    kind: hir.op.node.clone().into(),
                    left: l.into(),
                    right: r.into(),
                }
                .into()
            }
        }
        hir::ExprKind::Literal(hir) => {
            let constant: Constant = match hir {
                hir::Literal::Int(v) => (*v).into(),
                hir::Literal::Long(v) => (*v).into(),
                hir::Literal::Bool(v) => (*v).into(),
                hir::Literal::String(_) => {
                    // String literals are only used in fromBase16 — they shouldn't reach MIR directly
                    return Err(MirLoweringError::new(
                        "String literal cannot be used directly; use fromBase16()".to_string(),
                        hir_expr.span,
                    ));
                }
            };
            constant.into()
        }
        hir::ExprKind::Apply(apply) => {
            // Check if this is a named built-in function (before lowering func)
            if let hir::ExprKind::Ident(name) = &apply.func.kind {
                // Special handling for fromBase16 — extract string at compile time
                if name == "fromBase16" {
                    let str_arg = apply.args.first().ok_or_else(|| {
                        MirLoweringError::new(
                            "fromBase16 requires a string argument".to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let hex_str = match &str_arg.kind {
                        hir::ExprKind::Literal(hir::Literal::String(s)) => s.clone(),
                        _ => {
                            return Err(MirLoweringError::new(
                                "fromBase16 argument must be a string literal".to_string(),
                                hir_expr.span,
                            ))
                        }
                    };
                    let bytes = base16::decode(hex_str.as_bytes()).map_err(|e| {
                        MirLoweringError::new(
                            format!("Invalid hex in fromBase16: {:?}", e),
                            hir_expr.span,
                        )
                    })?;
                    return Ok(Constant::from(bytes).into());
                }
                // Lower all args for other builtins
                let args: Result<Vec<Expr>, MirLoweringError> =
                    apply.args.iter().map(|a| lower(a.clone())).collect();
                let args = args?;
                match name.as_str() {
                    "sigmaProp" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "sigmaProp requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        BoolToSigmaProp {
                            input: input.into(),
                        }
                        .into()
                    }
                    "blake2b256" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "blake2b256 requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        CalcBlake2b256 {
                            input: input.into(),
                        }
                        .into()
                    }
                    "proveDlog" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "proveDlog requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        CreateProveDlog {
                            input: input.into(),
                        }
                        .into()
                    }
                    "atLeast" => {
                        let mut it = args.into_iter();
                        let bound = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "atLeast requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let input = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "atLeast requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ergotree_ir::mir::atleast::Atleast::new(bound, input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "allOf" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "allOf requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ergotree_ir::mir::and::And {
                            input: input.into(),
                        }
                        .into()
                    }
                    "anyOf" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "anyOf requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ergotree_ir::mir::or::Or {
                            input: input.into(),
                        }
                        .into()
                    }
                    "longToByteArray" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "longToByteArray requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        LongToByteArray {
                            input: input.into(),
                        }
                        .into()
                    }
                    "min" => {
                        let mut it = args.into_iter();
                        let left = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "min requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let right = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "min requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        BinOp {
                            kind: ArithOp::Min.into(),
                            left: left.into(),
                            right: right.into(),
                        }
                        .into()
                    }
                    "Coll" => {
                        // Coll[Type]() or Coll(items...) — collection constructor
                        let elem_tpe = apply.type_arg.clone().unwrap_or_else(|| {
                            args.first().map(|a| a.tpe()).unwrap_or(SType::SByte)
                        });
                        return Ok(Collection::new(elem_tpe, args)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into());
                    }
                    "getVar" => {
                        // getVar[Type](varId) — context variable access
                        let var_id_expr = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "getVar requires a variable ID argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let var_id = match &var_id_expr {
                            Expr::Const(c) => {
                                use ergotree_ir::mir::constant::Literal;
                                match &c.v {
                                    Literal::Int(id) => *id as u8,
                                    Literal::Byte(id) => *id as u8,
                                    _ => {
                                        return Err(MirLoweringError::new(
                                            "getVar variable ID must be an integer constant"
                                                .to_string(),
                                            hir_expr.span,
                                        ))
                                    }
                                }
                            }
                            _ => {
                                return Err(MirLoweringError::new(
                                    "getVar variable ID must be a constant".to_string(),
                                    hir_expr.span,
                                ))
                            }
                        };
                        // Type from [Type] annotation
                        let var_tpe = apply.type_arg.clone().unwrap_or(SType::SAny);
                        return Ok(GetVar { var_id, var_tpe }.into());
                    }
                    "decodePoint" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "decodePoint requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        DecodePoint::try_build(input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "max" => {
                        let mut it = args.into_iter();
                        let left = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "max requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let right = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "max requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        BinOp {
                            kind: ArithOp::Max.into(),
                            left: left.into(),
                            right: right.into(),
                        }
                        .into()
                    }
                    "substConstants" => {
                        let mut it = args.into_iter();
                        let script_bytes = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "substConstants requires three arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let positions = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "substConstants requires three arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let new_values = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "substConstants requires three arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        SubstConstants::new(script_bytes, positions, new_values)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "byteArrayToLong" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "byteArrayToLong requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ByteArrayToLong::try_build(input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "byteArrayToBigInt" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "byteArrayToBigInt requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ByteArrayToBigInt::try_build(input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "xor" => {
                        let mut it = args.into_iter();
                        let left = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "xor requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let right = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "xor requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        Xor::new(left, right)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "xorOf" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "xorOf requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        XorOf {
                            input: input.into(),
                        }
                        .into()
                    }
                    other => {
                        return Err(MirLoweringError::new(
                            format!("MIR error: Unknown function: {}", other),
                            hir_expr.span,
                        ))
                    }
                }
            } else if let hir::ExprKind::FieldAccess(fa) = &apply.func.kind {
                // Method call: coll.method(lambda)
                let obj = lower(*fa.object.clone())?;
                let args: Result<Vec<Expr>, MirLoweringError> =
                    apply.args.iter().map(|a| lower(a.clone())).collect();
                let args = args?;
                let method = fa.field.as_str();
                // Check if this is property access + indexing (not a collection method call)
                // e.g., SELF.tokens(0), CONTEXT.dataInputs(0)
                #[allow(clippy::nonminimal_bool)]
                if !matches!(
                    method,
                    "filter"
                        | "exists"
                        | "forall"
                        | "map"
                        | "fold"
                        | "slice"
                        | "getOrElse"
                        | "insert"
                        | "update"
                        | "remove"
                        | "getMany"
                        | "contains"
                        | "updateDigest"
                        | "updateOperations"
                ) && !(method == "get" && matches!(fa.object.tpe, Some(SType::SAvlTree)))
                {
                    // Lower the FieldAccess as a property first, then apply indexing
                    let prop = lower(*apply.func.clone())?;
                    let args: Result<Vec<Expr>, MirLoweringError> =
                        apply.args.iter().map(|a| lower(a.clone())).collect();
                    let args = args?;
                    match prop.tpe() {
                        SType::SColl(_) => {
                            let index = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "Collection index requires one argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            ByIndex::new(prop, index, None)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        _ => {
                            return Err(MirLoweringError::new(
                                "MIR error: Cannot index non-collection".to_string(),
                                hir_expr.span,
                            ))
                        }
                    }
                } else {
                    match method {
                        "filter" => {
                            let cond = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "filter requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Filter::new(obj, cond)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "exists" => {
                            let cond = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "exists requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Exists::new(obj, cond)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "forall" => {
                            let cond = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "forall requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            ForAll::new(obj, cond)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "fold" => {
                            let mut it = args.into_iter();
                            let zero = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "fold requires zero value".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let fold_op = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "fold requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let fold_op = transform_fold_lambda(fold_op, hir_expr.span)?;
                            Fold::new(obj, zero, fold_op)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "map" => {
                            let mapper = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "map requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Map::new(obj, mapper)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        // AvlTree methods
                        "get" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                            let mut it = args.into_iter();
                            let key = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "AvlTree.get requires key".into(),
                                    hir_expr.span,
                                )
                            })?;
                            let proof = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "AvlTree.get requires proof".into(),
                                    hir_expr.span,
                                )
                            })?;
                            TreeLookup::new(obj, key, proof)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "insert" | "update" | "remove" | "getMany" | "contains"
                        | "updateDigest" | "updateOperations" => {
                            use ergotree_ir::types::savltree;
                            let method = match method {
                                "insert" => savltree::INSERT_METHOD.clone(),
                                "update" => savltree::UPDATE_METHOD.clone(),
                                "remove" => savltree::REMOVE_METHOD.clone(),
                                "getMany" => savltree::GET_MANY_METHOD.clone(),
                                "contains" => savltree::CONTAINS_METHOD.clone(),
                                "updateDigest" => savltree::UPDATE_DIGEST_METHOD.clone(),
                                "updateOperations" => savltree::UPDATE_OPERATIONS_METHOD.clone(),
                                _ => unreachable!(),
                            };
                            MethodCall::new(obj, method, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "getOrElse" => {
                            let mut it = args.into_iter();
                            let index = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "getOrElse requires index argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let default = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "getOrElse requires default argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            ByIndex::new(obj, index, Some(default.into()))
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "slice" => {
                            let mut it = args.into_iter();
                            let from = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "slice requires from argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let until = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "slice requires until argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Slice::new(obj, from, until)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        _ => {
                            return Err(MirLoweringError::new(
                                format!("MIR error: Unknown method: {}", method),
                                hir_expr.span,
                            ))
                        }
                    }
                } // close the else for property-vs-method
            } else {
                // Collection indexing: coll(index)
                let func = lower(*apply.func.clone())?;
                let args: Result<Vec<Expr>, MirLoweringError> =
                    apply.args.iter().map(|a| lower(a.clone())).collect();
                let args = args?;
                match func.tpe() {
                    SType::SColl(_) => {
                        let index = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "Collection index requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ByIndex::new(func, index, None)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    SType::SFunc(_) => {
                        // Lambda/function application: f(args)
                        ergotree_ir::mir::apply::Apply::new(func, args)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    _ => {
                        return Err(MirLoweringError::new(
                            "MIR error: Cannot apply non-function/non-collection".to_string(),
                            hir_expr.span,
                        ))
                    }
                }
            }
        }
        hir::ExprKind::Block(exprs) => {
            if exprs.len() == 1 {
                // Single-expression block: unwrap
                return lower(exprs[0].clone());
            }
            // Multi-expression block: items are ValDefs, last is result
            let last = exprs.last().ok_or_else(|| {
                MirLoweringError::new("Empty block expression".to_string(), hir_expr.span)
            })?;
            let items: Result<Vec<Expr>, MirLoweringError> = exprs[..exprs.len() - 1]
                .iter()
                .map(|e| lower(e.clone()))
                .collect();
            let result = lower(last.clone())?;
            BlockValue {
                items: items?,
                result: result.into(),
            }
            .into()
        }
        hir::ExprKind::ValDef(val_def) => {
            let id = val_def.id.ok_or_else(|| {
                MirLoweringError::new(
                    format!("MIR error: ValDef without id: {}", val_def.name),
                    hir_expr.span,
                )
            })?;
            let rhs = lower(*val_def.rhs.clone())?;
            return Ok(Expr::ValDef(
                ValDef {
                    id: ValId(id),
                    rhs: rhs.into(),
                }
                .into(),
            ));
        }
        hir::ExprKind::ValUse(val_use) => {
            return Ok(Expr::ValUse(ValUse {
                val_id: ValId(val_use.id),
                tpe: val_use.tpe.clone(),
            }));
        }
        hir::ExprKind::Context => {
            return Ok(Expr::Context);
        }
        hir::ExprKind::Tuple(items) => {
            let mir_items: Result<Vec<Expr>, MirLoweringError> =
                items.iter().map(|item| lower(item.clone())).collect();
            return Ok(Tuple::new(mir_items?)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                .into());
        }
        hir::ExprKind::LogicalNot(inner) => {
            let mir_inner = lower(*inner.clone())?;
            return Ok(LogicalNot::try_build(mir_inner)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                .into());
        }
        hir::ExprKind::Negation(inner) => {
            let mir_inner = lower(*inner.clone())?;
            return Ok(Negation::try_build(mir_inner)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                .into());
        }
        hir::ExprKind::If(if_expr) => {
            let condition = lower(*if_expr.condition.clone())?;
            let true_branch = lower(*if_expr.then_branch.clone())?;
            let false_branch = lower(*if_expr.else_branch.clone())?;
            return Ok(If {
                condition: condition.into(),
                true_branch: true_branch.into(),
                false_branch: false_branch.into(),
            }
            .into());
        }
        hir::ExprKind::Lambda(lambda) => {
            let args: Vec<FuncArg> = lambda
                .param_ids
                .iter()
                .zip(lambda.params.iter())
                .map(|(id, (_, tpe))| FuncArg {
                    idx: ValId(*id),
                    tpe: tpe.clone(),
                })
                .collect();
            let body = lower(*lambda.body.clone())?;
            return Ok(Expr::FuncValue(FuncValue::new(args, body)));
        }
        hir::ExprKind::FieldAccess(fa) => {
            let obj = lower(*fa.object.clone())?;
            let obj_tpe = fa.object.tpe.clone();
            match fa.field.as_str() {
                "value" => ExtractAmount { input: obj.into() }.into(),
                "propositionBytes" => ExtractScriptBytes { input: obj.into() }.into(),
                "id" => ExtractId { input: obj.into() }.into(),
                "creationInfo" => ExtractCreationInfo { input: obj.into() }.into(),
                "bytes" => ExtractBytes { input: obj.into() }.into(),
                "size" => SizeOf { input: obj.into() }.into(),
                "tokens" => {
                    use ergotree_ir::types::sbox::TOKENS_METHOD;
                    PropertyCall::new(obj, TOKENS_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                field if field.starts_with('_') => {
                    // Tuple field access: _1, _2, etc.
                    let idx: u8 = field[1..].parse().map_err(|_| {
                        MirLoweringError::new(
                            format!("Invalid tuple field: {}", field),
                            hir_expr.span,
                        )
                    })?;
                    let field_index = TupleFieldIndex::try_from(idx).map_err(|_| {
                        MirLoweringError::new(
                            format!("Tuple field index out of bounds: {}", idx),
                            hir_expr.span,
                        )
                    })?;
                    SelectField::new(obj, field_index)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "dataInputs" => {
                    use ergotree_ir::types::scontext::DATA_INPUTS_PROPERTY;
                    PropertyCall::new(obj, DATA_INPUTS_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "selfBoxIndex" => {
                    use ergotree_ir::types::scontext::SELF_BOX_INDEX_PROPERTY;
                    PropertyCall::new(obj, SELF_BOX_INDEX_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "get" => OptionGet::try_build(obj)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into(),
                "isDefined" => OptionIsDefined::try_build(obj)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into(),
                "propBytes" => SigmaPropBytes::try_build(obj)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into(),
                "toLong" => Upcast::new(obj, SType::SLong)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into(),
                "toBigInt" => {
                    // Constant fold: literal.toBigInt → BigInt constant
                    if let Expr::Const(c) = &obj {
                        use ergotree_ir::bigint256::BigInt256;
                        let folded: Option<BigInt256> = match &c.v {
                            ergotree_ir::mir::constant::Literal::Int(v) => {
                                BigInt256::try_from(*v as i64).ok()
                            }
                            ergotree_ir::mir::constant::Literal::Long(v) => {
                                BigInt256::try_from(*v).ok()
                            }
                            _ => None,
                        };
                        if let Some(bi) = folded {
                            Constant::from(bi).into()
                        } else {
                            Upcast::new(obj, SType::SBigInt)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                    } else {
                        Upcast::new(obj, SType::SBigInt)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                }
                "toInt" => {
                    // Upcast for smaller→Int, Downcast for larger→Int
                    if matches!(obj.tpe(), SType::SLong | SType::SBigInt) {
                        Downcast::new(obj, SType::SInt)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    } else {
                        Upcast::new(obj, SType::SInt)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                }
                "toByte" => Downcast::new(obj, SType::SByte)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into(),
                "toShort" => {
                    if matches!(obj.tpe(), SType::SInt | SType::SLong | SType::SBigInt) {
                        Downcast::new(obj, SType::SShort)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    } else {
                        Upcast::new(obj, SType::SShort)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                }
                "preHeader" => {
                    use ergotree_ir::types::scontext::PRE_HEADER_PROPERTY;
                    PropertyCall::new(obj, PRE_HEADER_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "timestamp" => {
                    use ergotree_ir::types::spreheader::TIMESTAMP_PROPERTY;
                    PropertyCall::new(obj, TIMESTAMP_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "height" if matches!(fa.object.tpe, Some(SType::SPreHeader)) => {
                    use ergotree_ir::types::spreheader::HEIGHT_PROPERTY;
                    PropertyCall::new(obj, HEIGHT_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "minerPk" => {
                    use ergotree_ir::types::spreheader::MINER_PK_PROPERTY;
                    PropertyCall::new(obj, MINER_PK_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "digest" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                    use ergotree_ir::types::savltree::DIGEST_METHOD;
                    PropertyCall::new(obj, DIGEST_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "enabledOperations" => {
                    use ergotree_ir::types::savltree::ENABLED_OPERATIONS_METHOD;
                    PropertyCall::new(obj, ENABLED_OPERATIONS_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "keyLength" => {
                    use ergotree_ir::types::savltree::KEY_LENGTH_METHOD;
                    PropertyCall::new(obj, KEY_LENGTH_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                field
                    if field.len() >= 2
                        && field.starts_with('R')
                        && field[1..].parse::<i8>().is_ok() =>
                {
                    // Register access: .R4, .R5, etc.
                    let reg_id: i8 = field[1..].parse().unwrap();
                    let elem_tpe = fa.type_args.first().cloned().unwrap_or(SType::SAny);
                    let opt_tpe = SType::SOption(elem_tpe.into());
                    ExtractRegisterAs::new(obj, reg_id, opt_tpe)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                other => {
                    return Err(MirLoweringError::new(
                        format!("MIR error: Unknown field '{}' on type {:?}", other, obj_tpe),
                        hir_expr.span,
                    ))
                }
            }
        }
    };
    let hir_tpe = hir_expr.tpe.clone().ok_or_else(|| {
        MirLoweringError::new(
            format!("MIR error: missing tpe for HIR: {0:?}", hir_expr),
            hir_expr.span,
        )
    })?;
    if mir.tpe() == hir_tpe {
        Ok(mir)
    } else if mir.tpe() == SType::SSigmaProp && hir_tpe == SType::SBoolean {
        // Auto-promotion: &&/|| with a SigmaProp operand produces SigmaProp in MIR
        // even though HIR typed it as SBoolean. This is expected.
        Ok(mir)
    } else {
        Err(MirLoweringError::new(
            format!(
                "MIR error: lowered MIR type != HIR type ({0:?} != {1:?})",
                mir.tpe(),
                hir_expr.tpe
            ),
            hir_expr.span,
        ))
    }
}

/// Transform a 2-param fold lambda into a 1-param lambda with tuple destructuring.
/// Fold expects: `(tuple: (AccType, ElemType)) => body` but our compiler produces
/// `(acc: AccType, elem: ElemType) => body`. This rewrites the latter into the former.
fn transform_fold_lambda(fold_op: Expr, span: TextRange) -> Result<Expr, MirLoweringError> {
    match fold_op {
        Expr::FuncValue(fv) if fv.args().len() == 2 => {
            let arg1 = &fv.args()[0];
            let arg2 = &fv.args()[1];
            let acc_id = arg1.idx;
            let acc_tpe = arg1.tpe.clone();
            let elem_id = arg2.idx;
            let elem_tpe = arg2.tpe.clone();

            // Create a new ValId for the single tuple parameter
            let tuple_id = ValId(std::cmp::max(acc_id.0, elem_id.0) + 100);
            let tuple_tpe = SType::STuple(STuple::pair(acc_tpe.clone(), elem_tpe.clone()));

            // Replace ValUse(acc_id) with SelectField(ValUse(tuple_id), _1)
            // Replace ValUse(elem_id) with SelectField(ValUse(tuple_id), _2)
            let new_body = replace_val_uses(
                fv.body().clone(),
                acc_id,
                elem_id,
                tuple_id,
                &tuple_tpe,
                span,
            )?;

            let new_func = FuncValue::new(
                vec![FuncArg {
                    idx: tuple_id,
                    tpe: tuple_tpe,
                }],
                new_body,
            );
            Ok(Expr::FuncValue(new_func))
        }
        other => Ok(other),
    }
}

/// Recursively replace ValUse(acc_id) and ValUse(elem_id) with SelectField on the tuple param.
fn replace_val_uses(
    expr: Expr,
    acc_id: ValId,
    elem_id: ValId,
    tuple_id: ValId,
    tuple_tpe: &SType,
    span: TextRange,
) -> Result<Expr, MirLoweringError> {
    match expr {
        Expr::ValUse(vu) if vu.val_id == acc_id => {
            let tuple_use = Expr::ValUse(ValUse {
                val_id: tuple_id,
                tpe: tuple_tpe.clone(),
            });
            let fi = TupleFieldIndex::try_from(1u8).unwrap();
            Ok(SelectField::new(tuple_use, fi)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::ValUse(vu) if vu.val_id == elem_id => {
            let tuple_use = Expr::ValUse(ValUse {
                val_id: tuple_id,
                tpe: tuple_tpe.clone(),
            });
            let fi = TupleFieldIndex::try_from(2u8).unwrap();
            Ok(SelectField::new(tuple_use, fi)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::BinOp(spanned) => {
            let inner = spanned.expr().clone();
            let new_left =
                replace_val_uses(*inner.left, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_right =
                replace_val_uses(*inner.right, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(BinOp {
                kind: inner.kind,
                left: new_left.into(),
                right: new_right.into(),
            }
            .into())
        }
        Expr::If(if_op) => {
            let new_cond =
                replace_val_uses(*if_op.condition, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_true = replace_val_uses(
                *if_op.true_branch,
                acc_id,
                elem_id,
                tuple_id,
                tuple_tpe,
                span,
            )?;
            let new_false = replace_val_uses(
                *if_op.false_branch,
                acc_id,
                elem_id,
                tuple_id,
                tuple_tpe,
                span,
            )?;
            Ok(If {
                condition: new_cond.into(),
                true_branch: new_true.into(),
                false_branch: new_false.into(),
            }
            .into())
        }
        Expr::BlockValue(spanned) => {
            let inner = spanned.expr().clone();
            let new_items: Result<Vec<Expr>, _> = inner
                .items
                .into_iter()
                .map(|item| replace_val_uses(item, acc_id, elem_id, tuple_id, tuple_tpe, span))
                .collect();
            let new_result =
                replace_val_uses(*inner.result, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(BlockValue {
                items: new_items?,
                result: new_result.into(),
            }
            .into())
        }
        Expr::ValDef(spanned) => {
            let inner = spanned.expr().clone();
            let new_rhs = replace_val_uses(*inner.rhs, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Expr::ValDef(
                ValDef {
                    id: inner.id,
                    rhs: new_rhs.into(),
                }
                .into(),
            ))
        }
        Expr::Tuple(tuple) => {
            let new_items: Result<Vec<Expr>, _> = tuple
                .items
                .as_vec()
                .iter()
                .map(|item| {
                    replace_val_uses(item.clone(), acc_id, elem_id, tuple_id, tuple_tpe, span)
                })
                .collect();
            Ok(Tuple::new(new_items?)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::SelectField(sf) => {
            let inner = sf.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(SelectField::new(new_input, inner.field_index)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::BoolToSigmaProp(bts) => {
            let new_input =
                replace_val_uses(*bts.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(BoolToSigmaProp {
                input: new_input.into(),
            }
            .into())
        }
        Expr::ExtractAmount(ea) => {
            let new_input =
                replace_val_uses(*ea.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(ExtractAmount {
                input: new_input.into(),
            }
            .into())
        }
        Expr::ExtractRegisterAs(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            // elem_tpe is the inner type; ExtractRegisterAs::new expects SOption(elem_tpe)
            let opt_tpe = SType::SOption(inner.elem_tpe.clone());
            Ok(
                ExtractRegisterAs::new(new_input, inner.register_id, opt_tpe)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                    .into(),
            )
        }
        Expr::OptionGet(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(OptionGet::try_build(new_input)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::SizeOf(so) => {
            let new_input =
                replace_val_uses(*so.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(SizeOf {
                input: new_input.into(),
            }
            .into())
        }
        Expr::PropertyCall(spanned) => {
            let inner = spanned.expr().clone();
            let new_obj = replace_val_uses(*inner.obj, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(PropertyCall::new(new_obj, inner.method)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::ByIndex(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_index =
                replace_val_uses(*inner.index, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(ByIndex::new(new_input, new_index, None)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::Filter(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_cond =
                replace_val_uses(*inner.condition, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Filter::new(new_input, new_cond)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::Exists(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_cond =
                replace_val_uses(*inner.condition, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Exists::new(new_input, new_cond)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::ForAll(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_cond =
                replace_val_uses(*inner.condition, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(ForAll::new(new_input, new_cond)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::FuncValue(fv) => {
            let new_body = replace_val_uses(
                fv.body().clone(),
                acc_id,
                elem_id,
                tuple_id,
                tuple_tpe,
                span,
            )?;
            Ok(Expr::FuncValue(FuncValue::new(
                fv.args().to_vec(),
                new_body,
            )))
        }
        Expr::LogicalNot(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(LogicalNot::try_build(new_input)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::Negation(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Negation::try_build(new_input)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        // Leaf nodes or nodes that don't contain our ValUse refs
        other => Ok(other),
    }
}

impl From<hir::BinaryOp> for BinOpKind {
    fn from(op: hir::BinaryOp) -> Self {
        match op {
            BinaryOp::Plus => ArithOp::Plus.into(),
            BinaryOp::Minus => ArithOp::Minus.into(),
            BinaryOp::Multiply => ArithOp::Multiply.into(),
            BinaryOp::Divide => ArithOp::Divide.into(),
            BinaryOp::Modulo => ArithOp::Modulo.into(),
            BinaryOp::Eq => RelationOp::Eq.into(),
            BinaryOp::Neq => RelationOp::NEq.into(),
            BinaryOp::Gt => RelationOp::Gt.into(),
            BinaryOp::Lt => RelationOp::Lt.into(),
            BinaryOp::Ge => RelationOp::Ge.into(),
            BinaryOp::Le => RelationOp::Le.into(),
            BinaryOp::And => LogicalOp::And.into(),
            BinaryOp::Or => LogicalOp::Or.into(),
        }
    }
}

#[cfg(test)]
pub fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = crate::parser::parse(input);
    let syntax = parse.syntax();
    let root = crate::ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root).unwrap();
    let binder = crate::binder::Binder::new(crate::script_env::ScriptEnv::new());
    let bind = binder.bind(hir).unwrap();
    let typed = crate::type_infer::assign_type(bind).unwrap();
    let res = lower(typed).unwrap();
    expected_tree.assert_eq(&res.debug_tree());
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn bin_smoke() {
        check(
            "HEIGHT + HEIGHT",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Arith(
                                Plus,
                            ),
                            left: GlobalVars(
                                Height,
                            ),
                            right: GlobalVars(
                                Height,
                            ),
                        },
                    },
                )"#]],
        )
    }

    #[test]
    fn literal_int() {
        check(
            "42",
            expect![[r#"
                Const(
                    "42: SInt",
                )"#]],
        );
    }

    #[test]
    fn literal_long() {
        check(
            "42L",
            expect![[r#"
                Const(
                    "42: SLong",
                )"#]],
        );
    }

    #[test]
    fn bin_numeric_int() {
        check(
            "4+2",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Arith(
                                Plus,
                            ),
                            left: Const(
                                "4: SInt",
                            ),
                            right: Const(
                                "2: SInt",
                            ),
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn bin_numeric_long() {
        check(
            "4L+2L",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Arith(
                                Plus,
                            ),
                            left: Const(
                                "4: SLong",
                            ),
                            right: Const(
                                "2: SLong",
                            ),
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn literal_bool_true() {
        check(
            "true",
            expect![[r#"
                Const(
                    "true: SBoolean",
                )"#]],
        );
    }

    #[test]
    fn literal_bool_false() {
        check(
            "false",
            expect![[r#"
                Const(
                    "false: SBoolean",
                )"#]],
        );
    }

    #[test]
    fn comparison_gt() {
        check(
            "HEIGHT > 0",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Relation(
                                Gt,
                            ),
                            left: GlobalVars(
                                Height,
                            ),
                            right: Const(
                                "0: SInt",
                            ),
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn sigmaprop_bool() {
        check(
            "sigmaProp(true)",
            expect![[r#"
                BoolToSigmaProp(
                    BoolToSigmaProp {
                        input: Const(
                            "true: SBoolean",
                        ),
                    },
                )"#]],
        );
    }
}
