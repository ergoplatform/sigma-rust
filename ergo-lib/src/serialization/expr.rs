use super::bin_op::bin_op_sigma_parse;
use super::bin_op::bin_op_sigma_serialize;
use super::{op_code::OpCode, sigma_byte_writer::SigmaByteWrite};
use crate::ast::and::And;
use crate::ast::apply::Apply;
use crate::ast::bin_op::ArithOp;
use crate::ast::bin_op::RelationOp;
use crate::ast::block::BlockValue;
use crate::ast::bool_to_sigma::BoolToSigmaProp;
use crate::ast::calc_blake2b256::CalcBlake2b256;
use crate::ast::coll_filter::Filter;
use crate::ast::coll_fold::Fold;
use crate::ast::coll_map::Map;
use crate::ast::collection::bool_const_coll_sigma_parse;
use crate::ast::collection::coll_sigma_parse;
use crate::ast::collection::coll_sigma_serialize;
use crate::ast::constant::Constant;
use crate::ast::constant::ConstantPlaceholder;
use crate::ast::expr::Expr;
use crate::ast::extract_amount::ExtractAmount;
use crate::ast::extract_reg_as::ExtractRegisterAs;
use crate::ast::func_value::FuncValue;
use crate::ast::global_vars::GlobalVars;
use crate::ast::logical_not::LogicalNot;
use crate::ast::method_call::MethodCall;
use crate::ast::option_get::OptionGet;
use crate::ast::or::Or;
use crate::ast::property_call::PropertyCall;
use crate::ast::select_field::SelectField;
use crate::ast::val_def::ValDef;
use crate::ast::val_use::ValUse;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};

use std::io;

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        match self {
            Expr::Const(c) => match w.constant_store() {
                Some(cs) => {
                    let ph = cs.put(c.clone());
                    ph.op_code().sigma_serialize(w)?;
                    ph.sigma_serialize(w)
                }
                None => c.sigma_serialize(w),
            },
            expr => {
                let op_code = self.op_code();
                op_code.sigma_serialize(w)?;
                match expr {
                    Expr::Const(_) => panic!("unexpected constant"), // handled in the code above (external match)
                    Expr::Fold(op) => op.sigma_serialize(w),
                    Expr::ConstPlaceholder(cp) => cp.sigma_serialize(w),
                    Expr::GlobalVars(_) => Ok(()),
                    Expr::MethodCall(mc) => mc.sigma_serialize(w),
                    Expr::ProperyCall(pc) => pc.sigma_serialize(w),
                    Expr::Context => Ok(()),
                    Expr::OptionGet(v) => v.sigma_serialize(w),
                    Expr::ExtractRegisterAs(v) => v.sigma_serialize(w),
                    Expr::BinOp(op) => bin_op_sigma_serialize(op, w),
                    Expr::BlockValue(op) => op.sigma_serialize(w),
                    Expr::ValUse(op) => op.sigma_serialize(w),
                    Expr::ValDef(op) => op.sigma_serialize(w),
                    Expr::FuncValue(op) => op.sigma_serialize(w),
                    Expr::Apply(op) => op.sigma_serialize(w),
                    Expr::ExtractAmount(op) => op.sigma_serialize(w),
                    Expr::SelectField(op) => op.sigma_serialize(w),
                    Expr::CalcBlake2b256(op) => op.sigma_serialize(w),
                    Expr::Collection(op) => coll_sigma_serialize(op, w),
                    Expr::And(op) => op.sigma_serialize(w),
                    Expr::Or(op) => op.sigma_serialize(w),
                    Expr::LogicalNot(op) => op.sigma_serialize(w),
                    Expr::Map(op) => op.sigma_serialize(w),
                    Expr::Filter(op) => op.sigma_serialize(w),
                    Expr::BoolToSigmaProp(op) => op.sigma_serialize(w),
                }
            }
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let first_byte = match r.peek_u8() {
            Ok(b) => Ok(b),
            Err(_) => {
                let res = r.get_u8(); // get(consume) the error
                assert!(res.is_err());
                res
            }
        }?;
        if first_byte <= OpCode::LAST_CONSTANT_CODE.value() {
            let constant = Constant::sigma_parse(r)?;
            Ok(Expr::Const(constant))
        } else {
            let op_code = OpCode::sigma_parse(r)?;
            match op_code {
                OpCode::FOLD => Ok(Fold::sigma_parse(r)?.into()),
                ConstantPlaceholder::OP_CODE => {
                    let cp = ConstantPlaceholder::sigma_parse(r)?;
                    if r.substitute_placeholders() {
                        // ConstantPlaceholder itself can be created only if a corresponding
                        // constant is in the constant_store, thus unwrap() is safe here
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
                OpCode::PROPERTY_CALL => Ok(Expr::ProperyCall(PropertyCall::sigma_parse(r)?)),
                OpCode::METHOD_CALL => Ok(Expr::MethodCall(MethodCall::sigma_parse(r)?)),
                OpCode::CONTEXT => Ok(Expr::Context),
                OpCode::OPTION_GET => Ok(OptionGet::sigma_parse(r)?.into()),
                OpCode::EXTRACT_REGISTER_AS => Ok(ExtractRegisterAs::sigma_parse(r)?.into()),
                OpCode::EQ => Ok(bin_op_sigma_parse(RelationOp::Eq.into(), r)?),
                OpCode::NEQ => Ok(bin_op_sigma_parse(RelationOp::NEq.into(), r)?),
                OpCode::LOGICAL_NOT => Ok(LogicalNot::sigma_parse(r)?.into()),
                OpCode::BIN_AND => Ok(bin_op_sigma_parse(RelationOp::And.into(), r)?),
                OpCode::BIN_OR => Ok(bin_op_sigma_parse(RelationOp::Or.into(), r)?),
                OpCode::GT => Ok(bin_op_sigma_parse(RelationOp::GT.into(), r)?),
                OpCode::LT => Ok(bin_op_sigma_parse(RelationOp::LT.into(), r)?),
                OpCode::GE => Ok(bin_op_sigma_parse(RelationOp::GE.into(), r)?),
                OpCode::LE => Ok(bin_op_sigma_parse(RelationOp::LE.into(), r)?),
                OpCode::PLUS => Ok(bin_op_sigma_parse(ArithOp::Plus.into(), r)?),
                OpCode::MINUS => Ok(bin_op_sigma_parse(ArithOp::Minus.into(), r)?),
                OpCode::MULTIPLY => Ok(bin_op_sigma_parse(ArithOp::Multiply.into(), r)?),
                OpCode::DIVISION => Ok(bin_op_sigma_parse(ArithOp::Divide.into(), r)?),
                OpCode::BLOCK_VALUE => Ok(Expr::BlockValue(BlockValue::sigma_parse(r)?)),
                OpCode::FUNC_VALUE => Ok(Expr::FuncValue(FuncValue::sigma_parse(r)?)),
                OpCode::APPLY => Ok(Expr::Apply(Apply::sigma_parse(r)?)),
                OpCode::VAL_DEF => Ok(Expr::ValDef(ValDef::sigma_parse(r)?)),
                OpCode::VAL_USE => Ok(Expr::ValUse(ValUse::sigma_parse(r)?)),
                OpCode::EXTRACT_AMOUNT => Ok(Expr::ExtractAmount(ExtractAmount::sigma_parse(r)?)),
                OpCode::SELECT_FIELD => Ok(Expr::SelectField(SelectField::sigma_parse(r)?)),
                OpCode::CALC_BLAKE2B256 => Ok(CalcBlake2b256::sigma_parse(r)?.into()),
                And::OP_CODE => Ok(And::sigma_parse(r)?.into()),
                Or::OP_CODE => Ok(Or::sigma_parse(r)?.into()),
                OpCode::COLL => Ok(coll_sigma_parse(r)?.into()),
                OpCode::COLL_OF_BOOL_CONST => Ok(bool_const_coll_sigma_parse(r)?.into()),
                Map::OP_CODE => Ok(Map::sigma_parse(r)?.into()),
                Filter::OP_CODE => Ok(Filter::sigma_parse(r)?.into()),
                BoolToSigmaProp::OP_CODE => Ok(BoolToSigmaProp::sigma_parse(r)?.into()),
                o => Err(SerializationError::NotImplementedOpCode(format!(
                    "{0}(shift {1})",
                    o.value(),
                    o.shift()
                ))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::address::AddressEncoder;
    use crate::chain::address::NetworkPrefix;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Expr>()) {
            dbg!(&v);
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }

    #[test]
    fn age_usd_bank_contract() {
        // simplified version of
        // https://github.com/Emurgo/age-usd/tree/main/ageusd-smart-contracts/v0.4
        /*

          val rcDefaultPrice = 1000000L

          val minStorageRent = 10000000L

          val feePercent = 1

          val HEIGHT = 377771

          val coolingOffHeight: Int = 377770
          val INF = 1000000000L

          val longMax = 9223372036854775807L

          val minReserveRatioPercent = 400L // percent
          val defaultMaxReserveRatioPercent = 800L // percent

            // val dataInput = CONTEXT.dataInputs(0)
            // val validDataInput = dataInput.tokens(0)._1 == oraclePoolNFT
            val validDataInput = true

            // val bankBoxIn = SELF
            // val bankBoxOut = OUTPUTS(0)

            // val rateBox = dataInput
            // val receiptBox = OUTPUTS(1)

            // val rate = rateBox.R4[Long].get / 100
            val rate = 100000000 / 100

            // val scCircIn = bankBoxIn.R4[Long].get
            val scCircIn = 100L
            // val rcCircIn = bankBoxIn.R5[Long].get
            val rcCircIn = 100L
            // val bcReserveIn = bankBoxIn.value
            val bcReserveIn = 100000000L

            // val scTokensIn = bankBoxIn.tokens(0)._2
            val scTokensIn = 100
            // val rcTokensIn = bankBoxIn.tokens(1)._2
            val rcTokensIn = 100

            // val scCircOut = bankBoxOut.R4[Long].get
            val scCircOut = 100
            // val rcCircOut = bankBoxOut.R5[Long].get
            val rcCircOut = 101

            // val scTokensOut = bankBoxOut.tokens(0)._2
            val scTokensOut = 100
            // val rcTokensOut = bankBoxOut.tokens(1)._2
            val rcTokensOut = 99

            val totalScIn = scTokensIn + scCircIn
            val totalScOut = scTokensOut + scCircOut

            val totalRcIn = rcTokensIn + rcCircIn
            val totalRcOut = rcTokensOut + rcCircOut

            val rcExchange = rcTokensIn != rcTokensOut
            val scExchange = scTokensIn != scTokensOut

            val rcExchangeXorScExchange = (rcExchange || scExchange) && !(rcExchange && scExchange)

            // val circDelta = receiptBox.R4[Long].get
            val circDelta = 1L
            // val bcReserveDelta = receiptBox.R5[Long].get
            val bcReserveDelta = 1010000L

            // val bcReserveOut = bankBoxOut.value
            val bcReserveOut = 100000000L + 1010000L

            val rcCircDelta = if (rcExchange) circDelta else 0L
            val scCircDelta = if (rcExchange) 0L else circDelta

            val validDeltas = (scCircIn + scCircDelta == scCircOut) &&
                              (rcCircIn + rcCircDelta == rcCircOut) &&
                              (bcReserveIn + bcReserveDelta == bcReserveOut)

            val coinsConserved = totalRcIn == totalRcOut && totalScIn == totalScOut

            // val tokenIdsConserved = bankBoxOut.tokens(0)._1 == bankBoxIn.tokens(0)._1 && // also ensures that at least one token exists
            //                         bankBoxOut.tokens(1)._1 == bankBoxIn.tokens(1)._1 && // also ensures that at least one token exists
            //                         bankBoxOut.tokens(2)._1 == bankBoxIn.tokens(2)._1    // also ensures that at least one token exists

            val tokenIdsConserved = true

            // val mandatoryRateConditions = rateBox.tokens(0)._1 == oraclePoolNFT
            val mandatoryRateConditions = true
            val mandatoryBankConditions = //bankBoxOut.value >= $minStorageRent &&
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
        }
                 */
        let p2s_addr_str = "7Nq5tKsVYCgneNgEfA2BJKwGsWozezNLhCNsRBihcHVFkDTuTThd4Qt1bi7NfCK1HuuVfjksMrEftV6MEFajjuyp1TMD2PX7SYWvkg9zH4CtgpdoBjekCNXs5XawxXnW6FT7GCqXTpJUP2TkkuqBh1df99PTigehys36uZz9wQnkrJXrv3mw3Yy4CM622qe5wdqLtpEonjazEmsw8weqEYegDyfJnswDvDkLPXtcCB86i19jik4fnSTtCcYj3jpWCQ7WL5dZn1ivs5JGRsR2ioNCRiZd3Gu1zJBgbHkMg41Z6VeCRWXjGY99BUtgtQiepSHGHajFCVcFAHhVxccdVUPCxGeEL6c2dNx6qzEkVfTfHs5qBgJewR8KCZTCVTurNBHeqCSVdxnfFvhW3f72cNrae5E1UhTAXU2iX4LZMHQsKyefY24Aq1b1srTyRWLpixjbcezFqA2TKjGSn1p1ruxbR7AQpW24ByPKT9sFE9ii4qNeXDnLcGtAAGS9FC5SD1s516a4NCu6v9zZfTvRKGkCwt78J8DEVnhTbttjcsvqFsUXQrvAv7TGVsaT4mL6B7F5BhRoZwFkgRXqFUVCWvgqJrwwjFRtbc5aZz";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let script = addr.script().unwrap().proposition().unwrap();
        dbg!(script);
    }
}
