use crate::eval::EvalError;
use crate::eval::LambdaInvoker;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::smethod::SMethod;
use ergotree_ir::types::stuple::STuple;
use ergotree_ir::types::stype::SType::SInt;

use super::env::Env;
use super::Context;
use super::EvalFn;
use alloc::sync::Arc;
use core::convert::TryFrom;

pub(crate) static INDEX_OF_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    Ok(Value::Int({
        let normalized_input_vals: Vec<Value> = match obj {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected obj to be Value::Coll, got: {0:?}",
                obj
            ))),
        }?;
        let target_element = args
            .first()
            .cloned()
            .ok_or_else(|| EvalError::NotFound("indexOf: missing first arg".to_string()))?;
        let from = args
            .get(1)
            .cloned()
            .ok_or_else(|| EvalError::NotFound("indexOf: missing second arg".to_string()))?
            .try_extract_into::<i32>()?
            .max(0);

        normalized_input_vals
            .into_iter()
            .skip(from as usize)
            .position(|it| it == target_element)
            .map(|idx| idx as i32 + from)
            .unwrap_or(-1)
    }))
};

pub(crate) fn flatmap_eval<'ctx>(
    _mc: &SMethod,
    _env: &mut Env<'ctx>,
    ctx: &Context<'ctx>,
    obj: Value<'ctx>,
    args: Vec<Value<'ctx>>,
) -> Result<Value<'ctx>, EvalError> {
    let input_v = obj;
    let lambda_v = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("flatmap: eval is missing first arg".to_string()))?;
    let input_v_clone = input_v.clone();
    let lambda = match &lambda_v {
        Value::Lambda(l) => Ok(l),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected lambda to be Value::FuncValue got: {0:?}",
            input_v_clone
        ))),
    }?;
    if lambda.args.len() > 1 {
        return Err(EvalError::UnexpectedValue(format!(
            "flatmap: expected lambda taking 1 arg but got {} args",
            lambda.args.len()
        )));
    }
    let unsupported_msg =
        "unsupported lambda in flatMap: allowed usage `xs.flatMap(x => x.property)".to_string();
    if let Expr::MethodCall(mc) = &*lambda.body {
        if !mc.expr().args.is_empty() {
            return Err(EvalError::UnexpectedValue(unsupported_msg));
        }
    }
    lambda.args.first().ok_or_else(|| {
        EvalError::NotFound("flatmap: lambda has empty arguments list".to_string())
    })?;
    // The body evaluates in the lambda's CAPTURED environment (JVM closure
    // semantics) — not in the caller's env.
    let mut invoker = LambdaInvoker::new(lambda);
    let mut lambda_call = |arg: Value<'ctx>| invoker.invoke(ctx, vec![arg]);
    let mapper_input_tpe = lambda
        .args
        .first()
        .cloned()
        .map(|arg| arg.tpe)
        .ok_or_else(|| {
            EvalError::NotFound(
                "flatmap: lambda args are empty (does not have arguments)".to_string(),
            )
        })?;
    let normalized_input_vals: Vec<Value> = match input_v {
        Value::Coll(coll) => {
            if *coll.elem_tpe() != mapper_input_tpe {
                return Err(EvalError::UnexpectedValue(format!(
                    "expected Flatmap input element type to be {0:?}, got: {1:?}",
                    mapper_input_tpe,
                    coll.elem_tpe()
                )));
            };
            Ok(coll.as_vec())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected Flatmap input to be Value::Coll, got: {0:?}",
            input_v
        ))),
    }?;
    normalized_input_vals
        .iter()
        .map(|item| lambda_call(item.clone()))
        .collect::<Result<Vec<Value>, EvalError>>()
        .map(|values| {
            CollKind::from_vec_vec(lambda.body.tpe(), values).map_err(EvalError::TryExtractFrom)
        })
        .and_then(|v| v) // flatten <Result<Result<Value, _>, _>
        .map(Value::Coll)
}

pub(crate) static ZIP_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let (type_1, coll_1) = match obj {
        Value::Coll(coll) => Ok((coll.elem_tpe().clone(), coll.as_vec())),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected obj to be Value::Coll, got: {0:?}",
            obj
        ))),
    }?;
    let arg_1 = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("zip: missing first arg".to_string()))?;
    let (type_2, coll_2) = match arg_1 {
        Value::Coll(coll) => Ok((coll.elem_tpe().clone(), coll.as_vec())),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected first arg to be Value::Coll, got: {0:?}",
            arg_1
        ))),
    }?;
    let zip = coll_1
        .into_iter()
        .zip(coll_2)
        .map(|(a, b)| Value::Tup([a, b].into()))
        .collect::<Arc<[_]>>();
    let coll_zip = CollKind::from_collection(STuple::pair(type_1, type_2).into(), zip);
    match coll_zip {
        Ok(coll) => Ok(Value::Coll(coll)),
        Err(e) => Err(EvalError::TryExtractFrom(e)),
    }
};

pub(crate) static INDICES_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let input_len = match obj {
        Value::Coll(coll) => Ok(coll.len()),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected obj to be Value::Coll, got: {0:?}",
            obj
        ))),
    }?;
    let indices_i32 = (0..input_len)
        .map(|i| Ok(Value::Int(i32::try_from(i)?)))
        .collect::<Result<Arc<[_]>, core::num::TryFromIntError>>();
    match indices_i32 {
        Ok(vec_val) => match CollKind::from_collection(SInt, vec_val) {
            Ok(coll) => Ok(Value::Coll(coll)),
            Err(e) => Err(EvalError::TryExtractFrom(e)),
        },
        Err(e) => Err(EvalError::UnexpectedValue(format!(
            "Coll length overflow: {0:?}",
            e
        ))),
    }
};

pub(crate) static PATCH_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let (input_tpe, normalized_input_vals) = match obj {
        Value::Coll(coll) => Ok((coll.elem_tpe().clone(), coll.as_vec())),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected obj to be Value::Coll, got: {0:?}",
            obj
        ))),
    }?;
    let from_index_val = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("patch: missing first arg (from)".to_string()))?;
    let patch_val = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::NotFound("patch: missing second arg (patch)".to_string()))?;
    let replaced_val = args
        .get(2)
        .cloned()
        .ok_or_else(|| EvalError::NotFound("patch: missing third arg (replaced)".to_string()))?;

    let from = from_index_val.try_extract_into::<i32>()?.max(0) as usize;
    let replaced = replaced_val.try_extract_into::<i32>()?.max(0) as usize;
    let patch = match patch_val {
        Value::Coll(coll) => Ok(coll.as_vec()),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected patch arg to be Value::Coll, got: {0:?}",
            patch_val
        ))),
    }?;

    let res = normalized_input_vals
        .iter()
        .take(from)
        .chain(patch.iter())
        .chain(normalized_input_vals.iter().skip(from + replaced))
        .cloned()
        .collect::<Arc<[_]>>();
    Ok(Value::Coll(CollKind::from_collection(input_tpe, res)?))
};

pub(crate) static UPDATED_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let (input_tpe, normalized_input_vals) = match obj {
        Value::Coll(coll) => Ok((coll.elem_tpe().clone(), coll.as_vec())),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected obj to be Value::Coll, got: {0:?}",
            obj
        ))),
    }?;

    let target_index_val = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("updated: missing first arg (index)".to_string()))?;
    let update_val = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::NotFound("updated: missing second arg (update)".to_string()))?;

    let target_index_usize = target_index_val.clone().try_extract_into::<i32>()? as usize;
    let mut res = normalized_input_vals;

    match res.get_mut(target_index_usize) {
        Some(elem) => {
            *elem = update_val;
            Ok(Value::Coll(CollKind::from_collection(input_tpe, &res[..])?))
        }
        None => Err(EvalError::UnexpectedValue(format!(
            "updated: target index out of bounds, got: {:?}",
            target_index_val
        ))),
    }
};

pub(crate) static UPDATE_MANY_EVAL_FN: EvalFn =
    |_mc, _env, _ctx, obj, args| {
        let (input_tpe, normalized_input_vals) = match obj {
            Value::Coll(coll) => Ok((coll.elem_tpe().clone(), coll.as_vec())),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected obj to be Value::Coll, got: {0:?}",
                obj
            ))),
        }?;

        let indexes_arg = args.first().cloned().ok_or_else(|| {
            EvalError::NotFound("updated: missing first arg (indexes)".to_string())
        })?;
        let updates_arg = args.get(1).cloned().ok_or_else(|| {
            EvalError::NotFound("updated: missing second arg (updates)".to_string())
        })?;

        let (updates_tpe, updates_val) = match updates_arg {
            Value::Coll(coll) => Ok((coll.elem_tpe().clone(), coll.as_vec())),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected first arg to be Value::Coll, got: {0:?}",
                updates_arg
            ))),
        }?;

        let indexes_usize = indexes_arg
            .try_extract_into::<Vec<i32>>()?
            .into_iter()
            .map(|i| i as usize)
            .collect::<Vec<usize>>();

        let inputs_len = normalized_input_vals.len();
        let indexes_len = indexes_usize.len();
        let updates_len = updates_val.len();

        if indexes_len != updates_len {
            return Err(EvalError::UnexpectedValue(format!(
                "Collections should have same length but was: \
            {0:?} and {1:?}. \n Indexes: {2:?} \n Updates: {3:?}",
                indexes_len, updates_len, indexes_usize, updates_val
            )));
        };
        if input_tpe != updates_tpe {
            return Err(EvalError::UnexpectedValue(format!(
                "Collections should be same type but was: \
            {0:?} and {1:?}. \n Inputs: {2:?} \n Updates: {3:?}",
                input_tpe, updates_tpe, normalized_input_vals, updates_val
            )));
        };

        let mut i = 0;
        let mut res = normalized_input_vals;

        while i < indexes_len {
            let pos = indexes_usize[i];
            if pos >= inputs_len {
                return Err(EvalError::UnexpectedValue(format!(
                    "updateMany index out of bounds, got: {0:?}",
                    pos
                )));
            }
            let update = updates_val[i].clone();
            match res.get_mut(pos) {
                Some(elem) => *elem = update,
                None => {
                    return Err(EvalError::UnexpectedValue(format!(
                        "updateMany index out of bounds, got: {0:?}",
                        pos
                    )))
                }
            }
            i += 1;
        }
        Ok(Value::Coll(CollKind::from_collection(input_tpe, &res[..])?))
    };

pub(crate) static REVERSE_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let Value::Coll(coll) = obj else {
        return Err(EvalError::UnexpectedValue(format!(
            "Reverse: expected Coll, found {obj:?}"
        )));
    };
    Ok(Value::from(coll.reverse()))
};

pub(crate) static STARTS_WITH_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let Value::Coll(coll) = obj else {
        return Err(EvalError::UnexpectedValue(format!(
            "endsWith: expected Coll, found {obj:?}"
        )));
    };
    let Some(Value::Coll(prefix)) = args.first() else {
        return Err(EvalError::UnexpectedValue(format!(
            "startsWith: expected Coll argument, found {:?}",
            args.first(),
        )));
    };
    if prefix.elem_tpe() != coll.elem_tpe() {
        return Err(EvalError::UnexpectedValue(format!(
            "startsWith: expected prefix to be of type {:?}, found {:?}",
            coll.elem_tpe(),
            prefix.elem_tpe()
        )));
    }
    Ok(Value::from(coll.starts_with(prefix)))
};

pub(crate) static ENDS_WITH_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let Value::Coll(coll) = obj else {
        return Err(EvalError::UnexpectedValue(format!(
            "endsWith: expected Coll, found {obj:?}"
        )));
    };
    let Some(Value::Coll(suffix)) = args.first() else {
        return Err(EvalError::UnexpectedValue(format!(
            "endsWith: expected Coll argument, found {:?}",
            args.first(),
        )));
    };
    if suffix.elem_tpe() != coll.elem_tpe() {
        return Err(EvalError::UnexpectedValue(format!(
            "endsWith: expected suffix to be of type {:?}, found {:?}",
            coll.elem_tpe(),
            suffix.elem_tpe()
        )));
    }
    Ok(Value::from(coll.ends_with(suffix)))
};

pub(crate) static GET_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let Value::Coll(coll) = obj else {
        return Err(EvalError::UnexpectedValue(format!(
            "get: expected Coll, found {obj:?}"
        )));
    };
    let index = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("Get: index argument not found".into()))?
        .try_extract_into::<i32>()?;
    Ok(Value::Opt(
        index
            .try_into()
            .ok()
            .and_then(|index| coll.get_val(index))
            .map(Box::new),
    ))
};

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use alloc::sync::Arc;

    use alloc::vec::Vec;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::constant::Literal;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::func_value::FuncArg;
    use ergotree_ir::mir::func_value::FuncValue;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::mir::value::CollKind;
    use ergotree_ir::types::scoll;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::types::stype_param::STypeVar;

    use crate::eval::test_util::{eval_out_wo_ctx, try_eval_out_wo_ctx};

    #[test]
    fn eval_index_of() {
        let index_of_expr = |coll: Vec<i64>, elem: i64, from: i32| -> Expr {
            MethodCall::new(
                coll.into(),
                scoll::INDEX_OF_METHOD.clone().with_concrete_types(
                    &[(STypeVar::t(), SType::SLong)].iter().cloned().collect(),
                ),
                vec![elem.into(), from.into()],
            )
            .unwrap()
            .into()
        };
        let res = eval_out_wo_ctx::<i32>(&index_of_expr(vec![1i64, 2i64], 2, 0));
        assert_eq!(res, 1);
        // Test searching in array starting from 1st index
        let res = eval_out_wo_ctx::<i32>(&index_of_expr(vec![1i64, 2i64], 2, 1));
        assert_eq!(res, 1);

        // Test searching in array starting from 1st index
        let res = eval_out_wo_ctx::<i32>(&index_of_expr(vec![1i64, 2i64], 2, 1));
        assert_eq!(res, 1);
        // Test searching in array starting from index greater than array length
        let res = eval_out_wo_ctx::<i32>(&index_of_expr(vec![1i64, 2i64], 2, 10000));
        assert_eq!(res, -1);
        // Test element that doesn't exist
        let res = eval_out_wo_ctx::<i32>(&index_of_expr(vec![1i64, 2i64], 3, 0));
        assert_eq!(res, -1);
    }

    #[test]
    fn eval_index_of_default() {
        let coll_const: Constant = vec![1i64, 2i64].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::INDEX_OF_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![3i64.into(), 0i32.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<i32>(&expr);
        assert_eq!(res, -1);
    }

    #[test]
    fn eval_flatmap() {
        let coll_const = Constant {
            tpe: SType::SColl(Arc::new(SType::SColl(Arc::new(SType::SLong)))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: Arc::new([vec![4i64, 5i64].into(), vec![3i64].into()]),
                elem_tpe: SType::SColl(Arc::new(SType::SLong)),
            }),
        };
        let body: Expr = MethodCall::new(
            ValUse {
                val_id: 1.into(),
                tpe: SType::SColl(Arc::new(SType::SLong)),
            }
            .into(),
            scoll::INDICES_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![],
        )
        .unwrap()
        .into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::FLATMAP_METHOD.clone().with_concrete_types(
                &[
                    (STypeVar::iv(), SType::SColl(Arc::new(SType::SLong))),
                    (STypeVar::ov(), SType::SInt),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            vec![FuncValue::new(
                vec![FuncArg {
                    idx: 1.into(),
                    tpe: SType::SColl(Arc::new(SType::SLong)),
                }],
                body,
            )
            .into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i32>>(&expr);
        assert_eq!(res, vec![0, 1, 0]);
    }

    #[test]
    fn eval_zip_empty() {
        // Both empty
        let empty_coll_const: Constant = Vec::<i64>::new().into();
        let empty_input: Constant = Vec::<bool>::new().into();
        let expr: Expr = MethodCall::new(
            empty_coll_const.into(),
            scoll::ZIP_METHOD.clone().with_concrete_types(
                &[
                    (STypeVar::t(), SType::SLong),
                    (STypeVar::iv(), SType::SBoolean),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            vec![empty_input.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<(i64, bool)>>(&expr);
        assert_eq!(res, Vec::<(i64, bool)>::new());

        // Only obj empty
        let empty_coll_const: Constant = Vec::<i64>::new().into();
        let input: Constant = vec![true, false].into();
        let expr: Expr = MethodCall::new(
            empty_coll_const.into(),
            scoll::ZIP_METHOD.clone().with_concrete_types(
                &[
                    (STypeVar::t(), SType::SLong),
                    (STypeVar::iv(), SType::SBoolean),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            vec![input.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<(i64, bool)>>(&expr);
        assert_eq!(res, Vec::<(i64, bool)>::new());

        // Only input empty
        let coll_const: Constant = vec![1i64, 2i64].into();
        let empty_input: Constant = Vec::<bool>::new().into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::ZIP_METHOD.clone().with_concrete_types(
                &[
                    (STypeVar::t(), SType::SLong),
                    (STypeVar::iv(), SType::SBoolean),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            vec![empty_input.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<(i64, bool)>>(&expr);
        assert_eq!(res, Vec::<(i64, bool)>::new());
    }

    #[test]
    fn eval_zip_same_length() {
        let coll_const: Constant = vec![1i64, 2i64].into();
        let input: Constant = vec![true, false].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::ZIP_METHOD.clone().with_concrete_types(
                &[
                    (STypeVar::t(), SType::SLong),
                    (STypeVar::iv(), SType::SBoolean),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            vec![input.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<(i64, bool)>>(&expr);
        assert_eq!(res, vec![(1i64, true), (2, false)]);
    }

    #[test]
    fn eval_zip_different_lengths() {
        // Input shorter than obj
        let coll_const: Constant = vec![1i64, 2i64].into();
        let short_input: Constant = vec![true].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::ZIP_METHOD.clone().with_concrete_types(
                &[
                    (STypeVar::t(), SType::SLong),
                    (STypeVar::iv(), SType::SBoolean),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            vec![short_input.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<(i64, bool)>>(&expr);
        assert_eq!(res, vec![(1i64, true)]);

        // Input longer than obj
        let coll_const: Constant = vec![1i64, 2i64].into();
        let long_input: Constant = vec![true, false, true].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::ZIP_METHOD.clone().with_concrete_types(
                &[
                    (STypeVar::t(), SType::SLong),
                    (STypeVar::iv(), SType::SBoolean),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            vec![long_input.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<(i64, bool)>>(&expr);
        assert_eq!(res, vec![(1i64, true), (2, false)]);
    }

    #[test]
    fn eval_indices() {
        let coll_const: Constant = vec![1i64, 2i64, 3i64].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::INDICES_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i32>>(&expr);
        assert_eq!(res, vec![0i32, 1i32, 2i32]);
    }

    #[test]
    fn eval_indices_empty_coll() {
        let coll_const: Constant = Vec::<i64>::new().into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::INDICES_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i32>>(&expr);
        assert_eq!(res, Vec::<i32>::new());
    }

    #[test]
    fn eval_patch() {
        let coll_const: Constant = vec![1i64, 5i64, 5i64].into();
        let patch_input: Vec<i64> = vec![2i64, 3i64];

        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::PATCH_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![1i32.into(), patch_input.into(), 2i32.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i64>>(&expr);
        assert_eq!(res, vec![1i64, 2i64, 3i64]);
    }

    #[test]
    fn eval_patch_addition() {
        let coll_const: Constant = vec![1i64, 2i64, 4i64, 5i64].into();
        let patch_input: Vec<i64> = vec![3i64];

        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::PATCH_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![2i32.into(), patch_input.into(), 0i32.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i64>>(&expr);
        assert_eq!(res, vec![1i64, 2i64, 3i64, 4i64, 5i64]);
    }

    #[test]
    fn eval_patch_subtraction() {
        let coll_const: Constant = vec![1i64, 2i64, 5i64, 5i64, 4i64, 5i64].into();
        let patch_input: Vec<i64> = vec![3i64];

        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::PATCH_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![2i32.into(), patch_input.into(), 2i32.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i64>>(&expr);
        assert_eq!(res, vec![1i64, 2i64, 3i64, 4i64, 5i64]);
    }

    #[test]
    fn eval_patch_index_oob() {
        let coll_const: Constant = vec![1i64, 2i64, 3i64].into();
        let patch_input: Vec<i64> = vec![4i64, 5i64];

        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::PATCH_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![9i32.into(), patch_input.into(), 9i32.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i64>>(&expr);
        assert_eq!(res, vec![1i64, 2i64, 3i64, 4i64, 5i64]);
    }

    #[test]
    fn eval_patch_index_negative() {
        let coll_const: Constant = vec![1i64, 2i64, 3i64].into();
        let patch_input: Vec<i64> = vec![4i64, 5i64];

        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::PATCH_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![(-1i32).into(), patch_input.into(), (-1i32).into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i64>>(&expr);
        assert_eq!(res, vec![4i64, 5i64, 1i64, 2i64, 3i64]);
    }

    #[test]
    fn eval_update() {
        let coll_const: Constant = vec![1i64, 2i64, 3i64].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::UPDATED_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![1i32.into(), 5i64.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i64>>(&expr);
        assert_eq!(res, vec![1i64, 5i64, 3i64]);
    }

    #[test]
    fn eval_update_oob() {
        let coll_const: Constant = vec![1i64, 2i64, 3i64].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::UPDATED_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![5i32.into(), 5i64.into()],
        )
        .unwrap()
        .into();
        assert!(try_eval_out_wo_ctx::<Vec<i64>>(&expr).is_err());
    }

    #[test]
    fn eval_update_many() {
        let coll_const: Constant = vec![1i64, 2i64, 3i64].into();

        let indexes_input: Vec<i32> = vec![1i32, 2i32];
        let updates_input: Vec<i64> = vec![5i64, 6i64];
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::UPDATE_MANY_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![indexes_input.into(), updates_input.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<i64>>(&expr);
        assert_eq!(res, vec![1i64, 5i64, 6i64]);
    }

    #[test]
    fn eval_update_many_index_oob() {
        let coll_const: Constant = vec![1i64, 2i64, 3i64].into();

        let indexes_input: Vec<i32> = vec![1i32, 5i32];
        let updates_input: Vec<i64> = vec![5i64, 6i64];
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::UPDATE_MANY_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![indexes_input.into(), updates_input.into()],
        )
        .unwrap()
        .into();
        assert!(try_eval_out_wo_ctx::<Vec<i64>>(&expr).is_err());
    }

    #[test]
    fn eval_update_many_len_mismatch() {
        let coll_const: Constant = vec![1i64, 2i64, 3i64].into();

        let indexes_input: Vec<i32> = vec![1i32];
        let updates_input: Vec<i64> = vec![5i64, 6i64];
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::UPDATE_MANY_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![indexes_input.into(), updates_input.into()],
        )
        .unwrap()
        .into();
        assert!(try_eval_out_wo_ctx::<Vec<i64>>(&expr).is_err());
    }

    #[test]
    fn eval_reverse() {
        let arr = vec![1i64, 2i64, 3i64];
        let coll_const: Constant = arr.clone().into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::REVERSE_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].into_iter().collect()),
            vec![],
        )
        .unwrap()
        .into();
        assert_eq!(
            eval_out_wo_ctx::<Vec<i64>>(&expr),
            arr.into_iter().rev().collect::<Vec<_>>(),
        );
        let arr = vec![1i8, 2i8, 3i8];
        let coll_const: Constant = arr.clone().into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::REVERSE_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SByte)].into_iter().collect()),
            vec![],
        )
        .unwrap()
        .into();
        assert_eq!(
            eval_out_wo_ctx::<Vec<i8>>(&expr),
            arr.into_iter().rev().collect::<Vec<_>>(),
        );
    }
    #[test]
    fn eval_starts_with() {
        fn starts_with(input: Vec<i64>, prefix: Vec<i64>) -> bool {
            let mc: Expr = MethodCall::new(
                Constant::from(input).into(),
                scoll::STARTS_WITH_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), SType::SLong)].into_iter().collect()),
                vec![Constant::from(prefix).into()],
            )
            .unwrap()
            .into();
            eval_out_wo_ctx(&mc)
        }
        fn starts_with_byte_array(input: Vec<i8>, prefix: Vec<i8>) -> bool {
            let mc: Expr = MethodCall::new(
                Constant::from(input).into(),
                scoll::STARTS_WITH_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), SType::SByte)].into_iter().collect()),
                vec![Constant::from(prefix).into()],
            )
            .unwrap()
            .into();
            eval_out_wo_ctx(&mc)
        }
        assert!(starts_with(vec![1, 2, 3], vec![1, 2]));
        assert!(starts_with(vec![1, 2, 3], vec![1, 2, 3]));
        assert!(!starts_with(vec![1, 2, 3], vec![1, 2, 4]));
        assert!(!starts_with(vec![1, 2, 3], vec![1, 2, 3, 4]));
        assert!(starts_with(vec![], vec![]));
        assert!(starts_with(vec![1, 2], vec![]));
        assert!(starts_with_byte_array(vec![1, 2, 3], vec![1, 2]));
        assert!(starts_with_byte_array(vec![1, 2, 3], vec![1, 2, 3]));
        assert!(!starts_with_byte_array(vec![1, 2, 3], vec![1, 2, 4]));
        assert!(!starts_with_byte_array(vec![1, 2, 3], vec![1, 2, 3, 4]));
        assert!(starts_with_byte_array(vec![], vec![]));
        assert!(starts_with_byte_array(vec![1, 2], vec![]));
    }
    #[test]
    fn eval_ends_with() {
        fn ends_with(input: Vec<i64>, suffix: Vec<i64>) -> bool {
            let mc: Expr = MethodCall::new(
                Constant::from(input).into(),
                scoll::ENDS_WITH_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), SType::SLong)].into_iter().collect()),
                vec![Constant::from(suffix).into()],
            )
            .unwrap()
            .into();
            eval_out_wo_ctx(&mc)
        }
        fn ends_with_byte_array(input: Vec<i8>, suffix: Vec<i8>) -> bool {
            let mc: Expr = MethodCall::new(
                Constant::from(input).into(),
                scoll::ENDS_WITH_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), SType::SByte)].into_iter().collect()),
                vec![Constant::from(suffix).into()],
            )
            .unwrap()
            .into();
            eval_out_wo_ctx(&mc)
        }
        assert!(!ends_with(vec![1, 2, 3], vec![1, 2]));
        assert!(ends_with(vec![1, 2, 3], vec![2, 3]));
        assert!(!ends_with(vec![1, 2, 3], vec![2, 3, 4]));
        assert!(ends_with(vec![1, 2, 3], vec![1, 2, 3]));
        assert!(ends_with(vec![], vec![]));
        assert!(ends_with(vec![1, 2], vec![]));
        assert!(!ends_with_byte_array(vec![1, 2, 3], vec![1, 2]));
        assert!(ends_with_byte_array(vec![1, 2, 3], vec![2, 3]));
        assert!(!ends_with_byte_array(vec![1, 2, 3], vec![2, 3, 4]));
        assert!(ends_with_byte_array(vec![1, 2, 3], vec![1, 2, 3]));
        assert!(ends_with_byte_array(vec![], vec![]));
        assert!(ends_with_byte_array(vec![1, 2], vec![]));
    }
    #[test]
    fn eval_get() {
        fn get(input: Vec<i64>, index: i32) -> Option<i64> {
            let mc: Expr = MethodCall::new(
                Constant::from(input).into(),
                scoll::GET_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), SType::SLong)].into_iter().collect()),
                vec![Constant::from(index).into()],
            )
            .unwrap()
            .into();
            eval_out_wo_ctx(&mc)
        }
        assert_eq!(get(vec![1, 2], 0), Some(1));
        assert_eq!(get(vec![1, 2], 1), Some(2));
        assert_eq!(get(vec![1, 2], -1), None);
        assert_eq!(get(vec![1, 2], 2), None);
        assert_eq!(get(vec![], 0), None);
    }
}
