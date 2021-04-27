use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for Expr {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        ctx.cost_accum.add_cost_of(self)?;
        match self {
            Expr::Const(c) => Ok(c.v.clone()),
            Expr::ByteArrayToLong(op) => op.eval(env, ctx),
            Expr::ByteArrayToBigInt(op) => op.eval(env, ctx),
            Expr::LongToByteArray(op) => op.eval(env, ctx),
            Expr::CalcBlake2b256(op) => op.eval(env, ctx),
            Expr::Fold(op) => op.eval(env, ctx),
            Expr::ExtractRegisterAs(op) => op.eval(env, ctx),
            Expr::GlobalVars(op) => op.eval(env, ctx),
            Expr::MethodCall(op) => op.eval(env, ctx),
            Expr::ProperyCall(op) => op.eval(env, ctx),
            Expr::BinOp(op) => op.eval(env, ctx),
            Expr::Global => Ok(Value::Global),
            Expr::Context => Ok(Value::Context),
            Expr::OptionGet(v) => v.eval(env, ctx),
            Expr::Apply(op) => op.eval(env, ctx),
            Expr::FuncValue(op) => op.eval(env, ctx),
            Expr::ValUse(op) => op.eval(env, ctx),
            Expr::BlockValue(op) => op.eval(env, ctx),
            Expr::SelectField(op) => op.eval(env, ctx),
            Expr::ExtractAmount(op) => op.eval(env, ctx),
            Expr::ConstPlaceholder(_) => panic!("ConstPlaceholder cannot be evaluated"),
            Expr::Collection(op) => op.eval(env, ctx),
            Expr::ValDef(_) => panic!("ValDef is evaluated in BlockValue"),
            Expr::And(op) => op.eval(env, ctx),
            Expr::Or(op) => op.eval(env, ctx),
            Expr::Atleast(_) => todo!(),
            Expr::LogicalNot(op) => op.eval(env, ctx),
            Expr::Map(op) => op.eval(env, ctx),
            Expr::Filter(op) => op.eval(env, ctx),
            Expr::BoolToSigmaProp(op) => op.eval(env, ctx),
            Expr::Upcast(op) => op.eval(env, ctx),
            Expr::If(op) => op.eval(env, ctx),
            Expr::ByIndex(op) => op.eval(env, ctx),
            Expr::ExtractScriptBytes(op) => op.eval(env, ctx),
            Expr::SizeOf(op) => op.eval(env, ctx),
            Expr::CreateProveDlog(op) => op.eval(env, ctx),
            Expr::CreateProveDhTuple(op) => op.eval(env, ctx),
            Expr::ExtractCreationInfo(op) => op.eval(env, ctx),
            Expr::Exists(op) => op.eval(env, ctx),
            Expr::ExtractId(op) => op.eval(env, ctx),
            Expr::SigmaPropBytes(op) => op.eval(env, ctx),
            Expr::OptionIsDefined(op) => op.eval(env, ctx),
            Expr::OptionGetOrElse(op) => op.eval(env, ctx),
            Expr::Negation(op) => op.eval(env, ctx),
            Expr::ForAll(op) => op.eval(env, ctx),
            Expr::Tuple(op) => op.eval(env, ctx),
            Expr::DecodePoint(op) => op.eval(env, ctx),
            Expr::SigmaAnd(_) => todo!(),
            Expr::SigmaOr(_) => todo!(),
            Expr::DeserializeRegister(op) => op.eval(env, ctx),
            Expr::DeserializeContext(op) => op.eval(env, ctx),
            Expr::GetVar(op) => op.eval(env, ctx),
        }
    }
}
