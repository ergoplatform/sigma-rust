use super::bin_op::bin_op_sigma_parse;
use super::bin_op::bin_op_sigma_serialize;
use super::{op_code::OpCode, sigma_byte_writer::SigmaByteWrite};
use crate::has_opcode::HasOpCode;
use crate::has_opcode::HasStaticOpCode;
use crate::mir::and::And;
use crate::mir::apply::Apply;
use crate::mir::atleast::Atleast;
use crate::mir::bin_op::ArithOp;
use crate::mir::bin_op::BitOp;
use crate::mir::bin_op::LogicalOp;
use crate::mir::bin_op::RelationOp;
use crate::mir::bit_inversion::BitInversion;
use crate::mir::block::BlockValue;
use crate::mir::bool_to_sigma::BoolToSigmaProp;
use crate::mir::byte_array_to_bigint::ByteArrayToBigInt;
use crate::mir::byte_array_to_long::ByteArrayToLong;
use crate::mir::calc_blake2b256::CalcBlake2b256;
use crate::mir::calc_sha256::CalcSha256;
use crate::mir::coll_append::Append;
use crate::mir::coll_by_index::ByIndex;
use crate::mir::coll_exists::Exists;
use crate::mir::coll_filter::Filter;
use crate::mir::coll_fold::Fold;
use crate::mir::coll_forall::ForAll;
use crate::mir::coll_map::Map;
use crate::mir::coll_size::SizeOf;
use crate::mir::coll_slice::Slice;
use crate::mir::collection::bool_const_coll_sigma_parse;
use crate::mir::collection::coll_sigma_parse;
use crate::mir::collection::coll_sigma_serialize;
use crate::mir::constant::Constant;
use crate::mir::constant::ConstantPlaceholder;
use crate::mir::create_avl_tree::CreateAvlTree;
use crate::mir::create_prove_dh_tuple::CreateProveDhTuple;
use crate::mir::create_provedlog::CreateProveDlog;
use crate::mir::decode_point::DecodePoint;
use crate::mir::deserialize_context::DeserializeContext;
use crate::mir::deserialize_register::DeserializeRegister;
use crate::mir::downcast::Downcast;
use crate::mir::exponentiate::Exponentiate;
use crate::mir::expr::Expr;
use crate::mir::extract_amount::ExtractAmount;
use crate::mir::extract_bytes::ExtractBytes;
use crate::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef;
use crate::mir::extract_creation_info::ExtractCreationInfo;
use crate::mir::extract_id::ExtractId;
use crate::mir::extract_reg_as::ExtractRegisterAs;
use crate::mir::extract_script_bytes::ExtractScriptBytes;
use crate::mir::func_value::FuncValue;
use crate::mir::get_var::GetVar;
use crate::mir::global_vars::GlobalVars;
use crate::mir::if_op::If;
use crate::mir::logical_not::LogicalNot;
use crate::mir::long_to_byte_array::LongToByteArray;
use crate::mir::method_call::MethodCall;
use crate::mir::multiply_group::MultiplyGroup;
use crate::mir::negation::Negation;
use crate::mir::option_get::OptionGet;
use crate::mir::option_get_or_else::OptionGetOrElse;
use crate::mir::option_is_defined::OptionIsDefined;
use crate::mir::or::Or;
use crate::mir::property_call::PropertyCall;
use crate::mir::select_field::SelectField;
use crate::mir::sigma_and::SigmaAnd;
use crate::mir::sigma_or::SigmaOr;
use crate::mir::sigma_prop_bytes::SigmaPropBytes;
use crate::mir::subst_const::SubstConstants;
use crate::mir::tree_lookup::TreeLookup;
use crate::mir::tuple::Tuple;
use crate::mir::upcast::Upcast;
use crate::mir::val_def::ValDef;
use crate::mir::val_use::ValUse;
use crate::mir::xor::Xor;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};

use crate::mir::xor_of::XorOf;
use crate::serialization::types::TypeCode;

impl Expr {
    /// Parse expression from byte stream. This function should be used instead of
    /// `sigma_parse` when tag byte is already read for look-ahead
    pub fn parse_with_tag<R: SigmaByteRead>(r: &mut R, tag: u8) -> Result<Self, SigmaParsingError> {
        let res = if tag <= OpCode::LAST_CONSTANT_CODE.value() {
            let t_code = TypeCode::parse(tag)?;
            let constant = Constant::parse_with_type_code(r, t_code)?;
            Ok(Expr::Const(constant))
        } else {
            let op_code = OpCode::parse(tag);
            match op_code {
                OpCode::APPEND => Ok(Append::sigma_parse(r)?.into()),
                OpCode::FOLD => Ok(Fold::sigma_parse(r)?.into()),
                ConstantPlaceholder::OP_CODE => {
                    let cp = ConstantPlaceholder::sigma_parse(r)?;
                    if r.substitute_placeholders() {
                        // ConstantPlaceholder itself can be created only if a corresponding
                        // constant is in the constant_store, thus unwrap() is safe here
                        #[allow(clippy::unwrap_used)]
                        let c = r.constant_store().get(cp.id).unwrap();
                        Ok(Expr::Const(c.clone()))
                    } else {
                        Ok(Expr::ConstPlaceholder(cp))
                    }
                }
                OpCode::HEIGHT => Ok(Expr::GlobalVars(GlobalVars::Height)),
                OpCode::SELF_BOX => Ok(Expr::GlobalVars(GlobalVars::SelfBox)),
                OpCode::INPUTS => Ok(Expr::GlobalVars(GlobalVars::Inputs)),
                OpCode::OUTPUTS => Ok(Expr::GlobalVars(GlobalVars::Outputs)),
                OpCode::MINER_PUBKEY => Ok(Expr::GlobalVars(GlobalVars::MinerPubKey)),
                OpCode::GROUP_GENERATOR => Ok(Expr::GlobalVars(GlobalVars::GroupGenerator)),
                OpCode::GLOBAL => Ok(Expr::Global),
                OpCode::PROPERTY_CALL => {
                    Ok(Expr::PropertyCall(PropertyCall::sigma_parse(r)?.into()))
                }
                OpCode::METHOD_CALL => Ok(Expr::MethodCall(MethodCall::sigma_parse(r)?.into())),
                OpCode::CONTEXT => Ok(Expr::Context),
                OptionGet::OP_CODE => Ok(OptionGet::sigma_parse(r)?.into()),
                OptionIsDefined::OP_CODE => Ok(OptionIsDefined::sigma_parse(r)?.into()),
                OptionGetOrElse::OP_CODE => Ok(OptionGetOrElse::sigma_parse(r)?.into()),
                ExtractRegisterAs::OP_CODE => Ok(ExtractRegisterAs::sigma_parse(r)?.into()),
                ExtractScriptBytes::OP_CODE => Ok(ExtractScriptBytes::sigma_parse(r)?.into()),
                ExtractBytes::OP_CODE => Ok(ExtractBytes::sigma_parse(r)?.into()),
                ExtractBytesWithNoRef::OP_CODE => Ok(ExtractBytesWithNoRef::sigma_parse(r)?.into()),
                ExtractCreationInfo::OP_CODE => Ok(ExtractCreationInfo::sigma_parse(r)?.into()),
                ExtractId::OP_CODE => Ok(ExtractId::sigma_parse(r)?.into()),
                OpCode::EQ => Ok(bin_op_sigma_parse(RelationOp::Eq.into(), r)?),
                OpCode::NEQ => Ok(bin_op_sigma_parse(RelationOp::NEq.into(), r)?),
                Negation::OP_CODE => Ok(Negation::sigma_parse(r)?.into()),
                BitInversion::OP_CODE => Ok(BitInversion::sigma_parse(r)?.into()),
                OpCode::LOGICAL_NOT => Ok(LogicalNot::sigma_parse(r)?.into()),
                OpCode::BIN_AND => Ok(bin_op_sigma_parse(LogicalOp::And.into(), r)?),
                OpCode::BIN_OR => Ok(bin_op_sigma_parse(LogicalOp::Or.into(), r)?),
                OpCode::BIN_XOR => Ok(bin_op_sigma_parse(LogicalOp::Xor.into(), r)?),
                OpCode::GT => Ok(bin_op_sigma_parse(RelationOp::Gt.into(), r)?),
                OpCode::LT => Ok(bin_op_sigma_parse(RelationOp::Lt.into(), r)?),
                OpCode::GE => Ok(bin_op_sigma_parse(RelationOp::Ge.into(), r)?),
                OpCode::LE => Ok(bin_op_sigma_parse(RelationOp::Le.into(), r)?),
                OpCode::PLUS => Ok(bin_op_sigma_parse(ArithOp::Plus.into(), r)?),
                OpCode::MINUS => Ok(bin_op_sigma_parse(ArithOp::Minus.into(), r)?),
                OpCode::MULTIPLY => Ok(bin_op_sigma_parse(ArithOp::Multiply.into(), r)?),
                OpCode::DIVISION => Ok(bin_op_sigma_parse(ArithOp::Divide.into(), r)?),
                OpCode::MAX => Ok(bin_op_sigma_parse(ArithOp::Max.into(), r)?),
                OpCode::MIN => Ok(bin_op_sigma_parse(ArithOp::Min.into(), r)?),
                OpCode::MODULO => Ok(bin_op_sigma_parse(ArithOp::Modulo.into(), r)?),
                OpCode::BIT_OR => Ok(bin_op_sigma_parse(BitOp::BitOr.into(), r)?),
                OpCode::BIT_AND => Ok(bin_op_sigma_parse(BitOp::BitAnd.into(), r)?),
                OpCode::BIT_XOR => Ok(bin_op_sigma_parse(BitOp::BitXor.into(), r)?),
                OpCode::BLOCK_VALUE => Ok(Expr::BlockValue(BlockValue::sigma_parse(r)?.into())),
                OpCode::FUNC_VALUE => Ok(Expr::FuncValue(FuncValue::sigma_parse(r)?)),
                OpCode::APPLY => Ok(Expr::Apply(Apply::sigma_parse(r)?)),
                OpCode::VAL_DEF => Ok(Expr::ValDef(ValDef::sigma_parse(r)?.into())),
                OpCode::VAL_USE => Ok(Expr::ValUse(ValUse::sigma_parse(r)?)),
                ExtractAmount::OP_CODE => Ok(Expr::ExtractAmount(ExtractAmount::sigma_parse(r)?)),
                OpCode::SELECT_FIELD => Ok(Expr::SelectField(SelectField::sigma_parse(r)?)),
                OpCode::CALC_BLAKE2B256 => Ok(CalcBlake2b256::sigma_parse(r)?.into()),
                CalcSha256::OP_CODE => Ok(CalcSha256::sigma_parse(r)?.into()),
                And::OP_CODE => Ok(And::sigma_parse(r)?.into()),
                Or::OP_CODE => Ok(Or::sigma_parse(r)?.into()),
                Xor::OP_CODE => Ok(Xor::sigma_parse(r)?.into()),
                Atleast::OP_CODE => Ok(Atleast::sigma_parse(r)?.into()),
                OpCode::COLL => Ok(coll_sigma_parse(r)?.into()),
                OpCode::COLL_OF_BOOL_CONST => Ok(bool_const_coll_sigma_parse(r)?.into()),
                Map::OP_CODE => Ok(Map::sigma_parse(r)?.into()),
                Filter::OP_CODE => Ok(Filter::sigma_parse(r)?.into()),
                Exists::OP_CODE => Ok(Exists::sigma_parse(r)?.into()),
                ForAll::OP_CODE => Ok(ForAll::sigma_parse(r)?.into()),
                BoolToSigmaProp::OP_CODE => Ok(BoolToSigmaProp::sigma_parse(r)?.into()),
                Upcast::OP_CODE => Ok(Upcast::sigma_parse(r)?.into()),
                Downcast::OP_CODE => Ok(Downcast::sigma_parse(r)?.into()),
                If::OP_CODE => Ok(If::sigma_parse(r)?.into()),
                ByIndex::OP_CODE => Ok(ByIndex::sigma_parse(r)?.into()),
                SizeOf::OP_CODE => Ok(SizeOf::sigma_parse(r)?.into()),
                Slice::OP_CODE => Ok(Slice::sigma_parse(r)?.into()),
                CreateProveDlog::OP_CODE => Ok(CreateProveDlog::sigma_parse(r)?.into()),
                CreateProveDhTuple::OP_CODE => Ok(CreateProveDhTuple::sigma_parse(r)?.into()),
                SigmaPropBytes::OP_CODE => Ok(SigmaPropBytes::sigma_parse(r)?.into()),
                Tuple::OP_CODE => Ok(Tuple::sigma_parse(r)?.into()),
                DecodePoint::OP_CODE => Ok(DecodePoint::sigma_parse(r)?.into()),
                SubstConstants::OP_CODE => Ok(SubstConstants::sigma_parse(r)?.into()),
                ByteArrayToLong::OP_CODE => Ok(ByteArrayToLong::sigma_parse(r)?.into()),
                ByteArrayToBigInt::OP_CODE => Ok(ByteArrayToBigInt::sigma_parse(r)?.into()),
                LongToByteArray::OP_CODE => Ok(LongToByteArray::sigma_parse(r)?.into()),
                SigmaAnd::OP_CODE => Ok(SigmaAnd::sigma_parse(r)?.into()),
                SigmaOr::OP_CODE => Ok(SigmaOr::sigma_parse(r)?.into()),
                GetVar::OP_CODE => Ok(GetVar::sigma_parse(r)?.into()),
                DeserializeRegister::OP_CODE => Ok(DeserializeRegister::sigma_parse(r)?.into()),
                DeserializeContext::OP_CODE => Ok(DeserializeContext::sigma_parse(r)?.into()),
                MultiplyGroup::OP_CODE => Ok(MultiplyGroup::sigma_parse(r)?.into()),
                Exponentiate::OP_CODE => Ok(Exponentiate::sigma_parse(r)?.into()),
                XorOf::OP_CODE => Ok(XorOf::sigma_parse(r)?.into()),
                TreeLookup::OP_CODE => Ok(TreeLookup::sigma_parse(r)?.into()),
                CreateAvlTree::OP_CODE => Ok(CreateAvlTree::sigma_parse(r)?.into()),
                o => Err(SigmaParsingError::NotImplementedOpCode(format!(
                    "{0}(shift {1})",
                    o.value(),
                    o.shift()
                ))),
            }
        };
        res
    }
}

trait SigmaSerializableWithOpCode: SigmaSerializable + HasOpCode {
    fn sigma_serialize_w_opcode<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.op_code().sigma_serialize(w)?;
        self.sigma_serialize(w)
    }
}

impl<T: SigmaSerializable + HasOpCode> SigmaSerializableWithOpCode for T {}

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        match self {
            Expr::Const(c) => match w.constant_store_mut_ref() {
                Some(cs) => {
                    let ph = (*cs).put(c.clone());
                    ph.op_code().sigma_serialize(w)?;
                    ph.sigma_serialize(w)
                }
                None => c.sigma_serialize(w),
            },
            Expr::Append(ap) => ap.expr().sigma_serialize_w_opcode(w),
            Expr::Fold(op) => op.sigma_serialize_w_opcode(w),
            Expr::ConstPlaceholder(cp) => cp.sigma_serialize_w_opcode(w),
            Expr::SubstConstants(s) => s.expr().sigma_serialize_w_opcode(w),
            Expr::ByteArrayToLong(s) => s.expr().sigma_serialize_w_opcode(w),
            Expr::ByteArrayToBigInt(s) => s.expr().sigma_serialize_w_opcode(w),
            Expr::LongToByteArray(s) => s.sigma_serialize_w_opcode(w),
            Expr::GlobalVars(op) => op.op_code().sigma_serialize(w),
            Expr::MethodCall(mc) => mc.expr().sigma_serialize_w_opcode(w),
            Expr::PropertyCall(pc) => pc.expr().sigma_serialize_w_opcode(w),
            Expr::Global => OpCode::GLOBAL.sigma_serialize(w),
            Expr::Context => OpCode::CONTEXT.sigma_serialize(w),
            Expr::OptionGet(v) => v.sigma_serialize_w_opcode(w),
            Expr::ExtractRegisterAs(v) => v.sigma_serialize_w_opcode(w),
            Expr::BinOp(op) => {
                op.expr().op_code().sigma_serialize(w)?;
                bin_op_sigma_serialize(op.expr(), w)
            }
            Expr::BlockValue(op) => op.expr().sigma_serialize_w_opcode(w),
            Expr::ValUse(op) => op.sigma_serialize_w_opcode(w),
            Expr::ValDef(op) => op.expr().sigma_serialize_w_opcode(w),
            Expr::FuncValue(op) => op.sigma_serialize_w_opcode(w),
            Expr::Apply(op) => op.sigma_serialize_w_opcode(w),
            Expr::ExtractAmount(op) => op.sigma_serialize_w_opcode(w),
            Expr::SelectField(op) => op.sigma_serialize_w_opcode(w),
            Expr::CalcBlake2b256(op) => op.sigma_serialize_w_opcode(w),
            Expr::CalcSha256(op) => op.sigma_serialize_w_opcode(w),
            Expr::Collection(op) => {
                op.op_code().sigma_serialize(w)?;
                coll_sigma_serialize(op, w)
            }
            Expr::And(op) => op.sigma_serialize_w_opcode(w),
            Expr::Or(op) => op.sigma_serialize_w_opcode(w),
            Expr::Xor(op) => op.sigma_serialize_w_opcode(w),
            Expr::Atleast(op) => op.sigma_serialize_w_opcode(w),
            Expr::LogicalNot(op) => op.sigma_serialize_w_opcode(w),
            Expr::Map(op) => op.sigma_serialize_w_opcode(w),
            Expr::Filter(op) => op.sigma_serialize_w_opcode(w),
            Expr::BoolToSigmaProp(op) => op.sigma_serialize_w_opcode(w),
            Expr::Upcast(op) => op.sigma_serialize_w_opcode(w),
            Expr::Downcast(op) => op.sigma_serialize_w_opcode(w),
            Expr::If(op) => op.sigma_serialize_w_opcode(w),
            Expr::ByIndex(op) => op.expr().sigma_serialize_w_opcode(w),
            Expr::ExtractScriptBytes(op) => op.sigma_serialize_w_opcode(w),
            Expr::SizeOf(op) => op.sigma_serialize_w_opcode(w),
            Expr::Slice(op) => op.sigma_serialize_w_opcode(w),
            Expr::CreateProveDlog(op) => op.sigma_serialize_w_opcode(w),
            Expr::CreateProveDhTuple(op) => op.sigma_serialize_w_opcode(w),
            Expr::ExtractCreationInfo(op) => op.sigma_serialize_w_opcode(w),
            Expr::Exists(op) => op.sigma_serialize_w_opcode(w),
            Expr::ExtractId(op) => op.sigma_serialize_w_opcode(w),
            Expr::SigmaPropBytes(op) => op.sigma_serialize_w_opcode(w),
            Expr::OptionIsDefined(op) => op.sigma_serialize_w_opcode(w),
            Expr::OptionGetOrElse(op) => op.sigma_serialize_w_opcode(w),
            Expr::Negation(op) => op.expr().sigma_serialize_w_opcode(w),
            Expr::BitInversion(op) => op.sigma_serialize_w_opcode(w),
            Expr::ForAll(op) => op.sigma_serialize_w_opcode(w),
            Expr::Tuple(op) => op.sigma_serialize_w_opcode(w),
            Expr::DecodePoint(op) => op.sigma_serialize_w_opcode(w),
            Expr::SigmaAnd(op) => op.sigma_serialize_w_opcode(w),
            Expr::SigmaOr(op) => op.sigma_serialize_w_opcode(w),
            Expr::GetVar(op) => op.sigma_serialize_w_opcode(w),
            Expr::DeserializeRegister(op) => op.sigma_serialize_w_opcode(w),
            Expr::DeserializeContext(op) => op.sigma_serialize_w_opcode(w),
            Expr::MultiplyGroup(op) => op.sigma_serialize_w_opcode(w),
            Expr::Exponentiate(op) => op.sigma_serialize_w_opcode(w),
            Expr::XorOf(op) => op.sigma_serialize_w_opcode(w),
            Expr::ExtractBytes(op) => op.sigma_serialize_w_opcode(w),
            Expr::ExtractBytesWithNoRef(op) => op.sigma_serialize_w_opcode(w),
            Expr::TreeLookup(op) => op.sigma_serialize_w_opcode(w),
            Expr::CreateAvlTree(op) => op.sigma_serialize_w_opcode(w),
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let tag = r.get_u8()?;
        Self::parse_with_tag(r, tag)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use crate::chain::address::AddressEncoder;
    use crate::chain::address::NetworkPrefix;

    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    #[test]
    fn ser_global() {
        let e = Expr::Global;
        assert_eq!(sigma_serialize_roundtrip(&e), e);
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Expr>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }

    #[test]
    fn full_age_usd_bank_contract() {
        // almost full version of
        // https://github.com/Emurgo/age-usd/tree/main/ageusd-smart-contracts/v0.4
        /*
            {

              val rcDefaultPrice = 1000000L

              val minStorageRent = 10000000L

              val feePercent = 1

              val HEIGHT = 377771

              val coolingOffHeight: Int = 377770
              val INF = 1000000000L

              val longMax = 9223372036854775807L

              val minReserveRatioPercent = 400L // percent
              val defaultMaxReserveRatioPercent = 800L // percent

              val isExchange = if (CONTEXT.dataInputs.size > 0) {
                val dataInput = CONTEXT.dataInputs(0)
                //val validDataInput = dataInput.tokens(0)._1 == oraclePoolNFT
                val validDataInput = true

                val bankBoxIn = SELF
                val bankBoxOut = OUTPUTS(0)

                val rateBox = dataInput
                val receiptBox = OUTPUTS(1)

                val rate = rateBox.R4[Long].get / 100
                // val rate = 100000000 / 100

                val scCircIn = bankBoxIn.R4[Long].get
                // val scCircIn = 100L
                val rcCircIn = bankBoxIn.R5[Long].get
                // val rcCircIn = 100L
                val bcReserveIn = bankBoxIn.value
                // val bcReserveIn = 100000000L

                val scTokensIn = bankBoxIn.tokens(0)._2
                // val scTokensIn = 100
                val rcTokensIn = bankBoxIn.tokens(1)._2
                // val rcTokensIn = 100

                val scCircOut = bankBoxOut.R4[Long].get
                //val scCircOut = 100
                val rcCircOut = bankBoxOut.R5[Long].get
                //val rcCircOut = 101

                val scTokensOut = bankBoxOut.tokens(0)._2
                //val scTokensOut = 100
                val rcTokensOut = bankBoxOut.tokens(1)._2
                //val rcTokensOut = 99

                val totalScIn = scTokensIn + scCircIn
                val totalScOut = scTokensOut + scCircOut

                val totalRcIn = rcTokensIn + rcCircIn
                val totalRcOut = rcTokensOut + rcCircOut

                val rcExchange = rcTokensIn != rcTokensOut
                val scExchange = scTokensIn != scTokensOut

                val rcExchangeXorScExchange = (rcExchange || scExchange) && !(rcExchange && scExchange)

                val circDelta = receiptBox.R4[Long].get
                //val circDelta = 1L
                val bcReserveDelta = receiptBox.R5[Long].get
                //val bcReserveDelta = 1010000L

                val bcReserveOut = bankBoxOut.value
                //val bcReserveOut = 100000000L + 1010000L

                val rcCircDelta = if (rcExchange) circDelta else 0L
                val scCircDelta = if (rcExchange) 0L else circDelta

                val validDeltas = (scCircIn + scCircDelta == scCircOut) &&
                                  (rcCircIn + rcCircDelta == rcCircOut) &&
                                  (bcReserveIn + bcReserveDelta == bcReserveOut)

                val coinsConserved = totalRcIn == totalRcOut && totalScIn == totalScOut

                val tokenIdsConserved = bankBoxOut.tokens(0)._1 == bankBoxIn.tokens(0)._1 && // also ensures that at least one token exists
                                         bankBoxOut.tokens(1)._1 == bankBoxIn.tokens(1)._1 && // also ensures that at least one token exists
                                         bankBoxOut.tokens(2)._1 == bankBoxIn.tokens(2)._1    // also ensures that at least one token exists

                //val tokenIdsConserved = true

                //val mandatoryRateConditions = rateBox.tokens(0)._1 == oraclePoolNFT
                val mandatoryRateConditions = true
                val mandatoryBankConditions = bankBoxOut.value >= minStorageRent &&
                                              rcExchangeXorScExchange &&
                                              coinsConserved &&
                                              validDeltas &&
                                              tokenIdsConserved

                // exchange equations
                val bcReserveNeededOut = scCircOut * rate
                val bcReserveNeededIn = scCircIn * rate
                val liabilitiesIn = max(min(bcReserveIn, bcReserveNeededIn), 0)


                val maxReserveRatioPercent = if (HEIGHT > coolingOffHeight) defaultMaxReserveRatioPercent else INF

                val reserveRatioPercentOut =
                    if (bcReserveNeededOut == 0) maxReserveRatioPercent else bcReserveOut * 100 / bcReserveNeededOut

                val validReserveRatio = if (scExchange) {
                  if (scCircDelta > 0) {
                    reserveRatioPercentOut >= minReserveRatioPercent
                  } else true
                } else {
                  if (rcCircDelta > 0) {
                    reserveRatioPercentOut <= maxReserveRatioPercent
                  } else {
                    reserveRatioPercentOut >= minReserveRatioPercent
                  }
                }

                val brDeltaExpected = if (scExchange) { // sc
                  val liableRate = if (scCircIn == 0) longMax else liabilitiesIn / scCircIn
                  val scNominalPrice = min(rate, liableRate)
                  scNominalPrice * scCircDelta
                } else { // rc
                  val equityIn = bcReserveIn - liabilitiesIn
                  val equityRate = if (rcCircIn == 0) rcDefaultPrice else equityIn / rcCircIn
                  val rcNominalPrice = if (equityIn == 0) rcDefaultPrice else equityRate
                  rcNominalPrice * rcCircDelta
                }

                val fee = brDeltaExpected * feePercent / 100

                val actualFee = if (fee < 0) {fee * -1} else fee
                // actualFee is always positive, irrespective of brDeltaExpected

                val brDeltaExpectedWithFee = brDeltaExpected + actualFee

                mandatoryRateConditions &&
                mandatoryBankConditions &&
                bcReserveDelta == brDeltaExpectedWithFee &&
                validReserveRatio &&
                validDataInput
            } else false

            sigmaProp(isExchange || // INPUTS(0).tokens(0)._1 == updateNFT &&
                CONTEXT.dataInputs.size == 0)
        }
        */
        let p2s_addr_str = "HfdbQC2Zwr5vfAUxdmjmX6b3TxQbq5w764pwsz9LLKyZVhv7SpifLB22PieCgvzSaFLomv8HNr9dxxQSSYaQg6ZyFL37nPfuVib3hVL8h42jajp754NXGqv1s4eKcbPsKkBMeTmYVSSGrpnZHzjqvcT4oN8rqKGUtLVXHs4QKyBwwNQKS5KNC8DLkdvHUQRNv5r8pCJ6ehTi31h1rfLVTsaMhAeDcYCs1uS7YMXk3msfH36krAskv8TgApoFJ1DarszwiacTuE1o4N6o4PJJifAgJ1WH4XuGRieYE1k3fo631benRDQw9nQ49p4oqAda5aXTNmabAsfCgAR8jbmUzzi3UCyYJgRUtXp7ijaGfr6o3hXd5VHDZe4gM6Vw4Ly3s881WZX2WWNedrXNqKKMVXKk55jbgn3ZmFpZiLtvPHSBCG7ULyARrTz2rAUC16StdYBqPuhHpRKEx3QYeFTYJGcMbsMGompAkCxG37X7ZVs7m7xCpPuP3AqxWtWdxkTzw5FCHALsu6ZD334n8mFgn9kiif4tbShpBo1AJu6dP22XvPU3S93q5LuNaXx6d7u5VFrpQKSN6WnhkU4LUfh3t8YU1ZBATrQDGRkaji59pqoNDuwVSfn7g1UhcMWdMnwzrCNNq1jsX2KrkX7o81aS7LEmz6xAySdyvubGh51oXNd2cmgbJ9at2Tp3hNi9FwWG5iEk882AZ7gby6QktknAwyaw9CL5qdodeh4t659H42SoqK2ATtfrZgjU5b5pYAzNp9EjFHCKkYxTo7t5G1vHHZUXjTbkzc22ggJdH3BvZYEcdQtUCLbEFJSCiMp2RjxEmyh";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let script = addr.script().unwrap().proposition().unwrap();
        dbg!(&script);
    }
}
