use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

use super::EvalFn;

pub static DATA_INPUTS_EVAL_FN: EvalFn = |ctx, _obj, _args| {
    // TODO: check that obj is Value::Context
    Ok(Value::Coll(CollKind::WrappedColl {
        items: ctx
            .data_inputs
            .clone()
            .into_iter()
            .map(Value::CBox)
            .collect(),
        elem_tpe: SType::SBox,
    }))
};
