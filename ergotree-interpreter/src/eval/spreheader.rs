use ergotree_ir::{chain::preheader::PreHeader, mir::constant::TryExtractInto};

use super::EvalFn;

pub(crate) static VERSION_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok((preheader.version as i8).into())
};

pub(crate) static PARENT_ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok(Into::<Vec<i8>>::into(preheader.parent_id).into())
};

pub(crate) static TIMESTAMP_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok((preheader.timestamp as i64).into())
};

pub(crate) static N_BITS_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok((preheader.n_bits as i64).into())
};

pub(crate) static HEIGHT_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok((preheader.height as i32).into())
};

pub(crate) static MINER_PK_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok(preheader.miner_pk.into())
};

pub(crate) static VOTES_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let preheader = obj.try_extract_into::<PreHeader>()?;
    Ok(Into::<Vec<u8>>::into(preheader.votes).into())
};
