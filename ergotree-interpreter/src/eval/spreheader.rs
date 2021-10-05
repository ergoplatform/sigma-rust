use super::EvalFn;

use ergotree_ir::{chain::preheader::PreHeader, mir::constant::TryExtractInto};

pub(crate) static VERSION_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok((preheader.version as i8).into())
};

pub(crate) static ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok(Into::<Vec<i8>>::into(preheader.id).into())
};

pub(crate) static PARENT_ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok(Into::<Vec<i8>>::into(preheader.parent_id).into())
};