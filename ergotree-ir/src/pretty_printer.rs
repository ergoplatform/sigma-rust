//! Pretty printer for ErgoTree IR

use std::fmt::Write;

mod print;
pub use print::Print;

// TODO: extract to a separate module
/// Printer trait with tracking of current position and indent
pub trait Printer: Write {
    /// Current position (last printed char)
    fn current_pos(&self) -> usize;
    /// Increase indent
    fn inc_ident(&mut self);
    /// Decrease indent
    fn dec_ident(&mut self);
    /// Get current indent
    fn get_indent(&self) -> usize;
    /// Print the current indent
    fn print_indent(&mut self) -> std::fmt::Result {
        write!(self, "{:indent$}", "", indent = self.get_indent())
    }
}

/// Printer implementation with tracking of current position and indent
pub struct PosTrackingWriter {
    print_buf: String,
    current_pos: usize,
    current_indent: usize,
}

impl Write for PosTrackingWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let len = s.len();
        self.current_pos += len;
        write!(self.print_buf, "{}", s)
    }
}

impl Printer for PosTrackingWriter {
    fn current_pos(&self) -> usize {
        self.current_pos
    }

    fn inc_ident(&mut self) {
        self.current_indent += Self::INDENT;
    }

    fn dec_ident(&mut self) {
        self.current_indent -= Self::INDENT;
    }

    fn get_indent(&self) -> usize {
        self.current_indent
    }
}

impl PosTrackingWriter {
    const INDENT: usize = 2;

    /// Create new printer
    pub fn new() -> Self {
        Self {
            print_buf: String::new(),
            current_pos: 0,
            current_indent: 0,
        }
    }

    /// Get printed buffer
    pub fn get_buf(&self) -> &str {
        &self.print_buf
    }

    /// Get printed buffer as String
    pub fn as_string(self) -> String {
        self.print_buf
    }
}

impl Default for PosTrackingWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {

    use expect_test::expect;

    use crate::chain::address::AddressEncoder;
    use crate::chain::address::NetworkPrefix;
    use crate::ergo_tree::ErgoTree;
    use crate::mir::bin_op::ArithOp;
    use crate::mir::bin_op::BinOp;
    use crate::mir::block::BlockValue;
    use crate::mir::expr::Expr;
    use crate::mir::val_def::ValDef;
    use crate::mir::val_use::ValUse;
    use crate::serialization::SigmaSerializable;
    use crate::types::stype::SType;

    use super::*;

    fn check_pretty(expr: Expr, expected_tree: expect_test::Expect) {
        let print_buf = String::new();
        let mut w = PosTrackingWriter {
            print_buf,
            current_pos: 0,
            current_indent: 0,
        };
        let _ = expr.print(&mut w).unwrap();
        expected_tree.assert_eq(w.get_buf());
    }

    fn check_spans(expr: Expr, expected_tree: expect_test::Expect) {
        let print_buf = String::new();
        let mut w = PosTrackingWriter {
            print_buf,
            current_pos: 0,
            current_indent: 0,
        };
        let spanned_expr = expr.print(&mut w).unwrap();
        expected_tree.assert_eq(format!("{:?}", spanned_expr).as_str());
    }

    #[test]
    fn print_block() {
        let val_id = 1.into();
        let expr = Expr::BlockValue(
            BlockValue {
                items: vec![ValDef {
                    id: val_id,
                    rhs: Box::new(Expr::Const(1i32.into())),
                }
                .into()],
                result: Box::new(
                    ValUse {
                        val_id,
                        tpe: SType::SInt,
                    }
                    .into(),
                ),
            }
            .into(),
        );
        check_pretty(
            expr,
            expect![[r#"
            {
              val v1 = 1
              v1
            }
            "#]],
        );
    }

    #[test]
    fn print_binop() {
        let val_id = 1.into();
        let expr = Expr::BlockValue(
            BlockValue {
                items: vec![ValDef {
                    id: val_id,
                    rhs: Box::new(
                        BinOp {
                            kind: ArithOp::Divide.into(),
                            left: Expr::Const(4i32.into()).into(),
                            right: Expr::Const(2i32.into()).into(),
                        }
                        .into(),
                    ),
                }
                .into()],
                result: Box::new(
                    ValUse {
                        val_id,
                        tpe: SType::SInt,
                    }
                    .into(),
                ),
            }
            .into(),
        );
        check_pretty(
            expr.clone(),
            expect![[r#"
            {
              val v1 = 4 / 2
              v1
            }
            "#]],
        );

        check_spans(
            expr,
            expect![[
                r#"BlockValue(Spanned { source_span: SourceSpan { offset: 0, length: 26 }, expr: BlockValue { items: [ValDef(Spanned { source_span: SourceSpan { offset: 4, length: 14 }, expr: ValDef { id: ValId(1), rhs: BinOp(Spanned { source_span: SourceSpan { offset: 13, length: 5 }, expr: BinOp { kind: Arith(Divide), left: Const("4: SInt"), right: Const("2: SInt") } }) } })], result: ValUse(ValUse { val_id: ValId(1), tpe: SInt }) } })"#
            ]],
        );
    }

    #[test]
    fn eip23_refresh_contract() {
        let ergo_tree_bytes = base16::decode("1016043c040004000e202a472d4a614e645267556b58703273357638792f423f4528482b4d625065536801000502010105000400040004020402040204080400040a05c8010e20472b4b6250655368566d597133743677397a24432646294a404d635166546a570400040404020408d80ed60199a37300d602b2a4730100d603b5a4d901036395e6c672030605eded928cc77203017201938cb2db6308720373020001730393e4c672030504e4c6720205047304d604b17203d605b0720386027305860273067307d901053c413d0563d803d607e4c68c7205020605d6088c720501d6098c720802860272078602ed8c720901908c72080172079a8c7209027207d6068c720502d6078c720501d608db63087202d609b27208730800d60ab2a5730900d60bdb6308720ad60cb2720b730a00d60db27208730b00d60eb2a5730c00ea02ea02ea02ea02ea02ea02ea02ea02ea02ea02ea02ea02ea02ea02ea02ea02ea02cde4c6b27203e4e30004000407d18f8cc77202017201d1927204730dd18c720601d190997207e4c6b27203730e0006059d9c72077e730f057310d1938c7209017311d193b2720b7312007209d1938c720c018c720d01d1928c720c02998c720d027e9c7204731305d193b1720bb17208d193e4c6720a04059d8c7206027e720405d193e4c6720a05049ae4c6720205047314d193c2720ac27202d192c1720ac17202d1928cc7720a0199a37315d193db6308720edb6308a7d193c2720ec2a7d192c1720ec1a7").unwrap();
        let ergo_tree = ErgoTree::sigma_parse_bytes(&ergo_tree_bytes).unwrap();
        check_pretty(
            ergo_tree.proposition().unwrap(),
            expect![[r#"
                {
                  val v1 = HEIGHT - 30
                  val v2 = INPUTS(0)
                  val v3 = INPUTS.filter({
                      (v3: Box) => 
                        if (v3.getReg(6).isDefined()) v3.creationInfo._1 >= v1 && v3.tokens(0)._1 == "2a472d4a614e645267556b58703273357638792f423f4528482b4d6250655368" && v3.getReg(5).get == v2.getReg(5).get else false
                      }
                    )
                  val v4 = v3.size
                  val v5 = v3.fold((, 1, (, true, 0)))({
                      (v5: ((Long, (Boolean, Long)), Box)) => 
                        {
                          val v7 = v5._2.getReg(6).get
                          val v8 = v5._1
                          val v9 = v8._2
                          (, v7, (, v9._1 && v8._1 <= v7, v9._2 + v7))
                        }

                      }
                    )
                  val v6 = v5._2
                  val v7 = v5._1
                  val v8 = v2.tokens
                  val v9 = v8(0)
                  val v10 = OUTPUTS(0)
                  val v11 = v10.tokens
                  val v12 = v11(1)
                  val v13 = v8(1)
                  val v14 = OUTPUTS(1)
                  allOf(
                    allOf(
                      allOf(
                        allOf(
                          allOf(
                            allOf(
                              allOf(
                                allOf(
                                  allOf(
                                    allOf(
                                      allOf(
                                        allOf(
                                          allOf(
                                            allOf(
                                              allOf(
                                                allOf(
                                                  allOf(
                                                    proveDlog(v3(getVar(0).get).getReg(4).get), 
                                                    sigmaProp(v2.creationInfo._1 < v1), 
                                                  ), 
                                                  sigmaProp(v4 >= 4), 
                                                ), 
                                                sigmaProp(v6._1), 
                                              ), 
                                              sigmaProp(v7 - v3(0).getReg(6).get <= v7 * upcast(5) / 100), 
                                            ), 
                                            sigmaProp(v9._1 == "472b4b6250655368566d597133743677397a24432646294a404d635166546a57"), 
                                          ), 
                                          sigmaProp(v11(0) == v9), 
                                        ), 
                                        sigmaProp(v12._1 == v13._1), 
                                      ), 
                                      sigmaProp(v12._2 >= v13._2 - upcast(v4 * 2)), 
                                    ), 
                                    sigmaProp(v11.size == v8.size), 
                                  ), 
                                  sigmaProp(v10.getReg(4).get == v6._2 / upcast(v4)), 
                                ), 
                                sigmaProp(v10.getReg(5).get == v2.getReg(5).get + 1), 
                              ), 
                              sigmaProp(v10.propBytes == v2.propBytes), 
                            ), 
                            sigmaProp(v10.value >= v2.value), 
                          ), 
                          sigmaProp(v10.creationInfo._1 >= HEIGHT - 4), 
                        ), 
                        sigmaProp(v14.tokens == SELF.tokens), 
                      ), 
                      sigmaProp(v14.propBytes == SELF.propBytes), 
                    ), 
                    sigmaProp(v14.value >= SELF.value), 
                  )
                }
            "#]],
        )
    }

    #[test]
    fn eip23_update_contract() {
        let ergo_tree_bytes = base16::decode("100f0400040004000402040204020e20472b4b6250655368566d597133743677397a24432646294a404d635166546a570400040004000e203f4428472d4b6150645367566b5970337336763979244226452948404d625165010005000400040cd80ad601b2a4730000d602db63087201d603b27202730100d604b2a5730200d605db63087204d606b2a5730300d607b27205730400d6088c720701d6098c720702d60ab27202730500d1ededed938c7203017306edededed937203b2720573070093c17201c1720493c672010405c67204040593c672010504c672040504efe6c672040661edededed93db63087206db6308a793c27206c2a792c17206c1a7918cc77206018cc7a701efe6c67206046192b0b5a4d9010b63d801d60ddb6308720b9591b1720d7308d801d60ec6720b070eededed938cb2720d73090001730a93e4c6720b05048cc7a70193e4c6720b060ecbc2720495ede6720ee6c6720b0805ed93e4720e720893e4c6720b08057209ed938c720a017208938c720a027209730b730cd9010b41639a8c720b018cb2db63088c720b02730d00027e730e05").unwrap();
        let ergo_tree = ErgoTree::sigma_parse_bytes(&ergo_tree_bytes).unwrap();
        check_pretty(
            ergo_tree.proposition().unwrap(),
            expect![[r#"
                {
                  val v1 = INPUTS(0)
                  val v2 = v1.tokens
                  val v3 = v2(0)
                  val v4 = OUTPUTS(0)
                  val v5 = v4.tokens
                  val v6 = OUTPUTS(1)
                  val v7 = v5(1)
                  val v8 = v7._1
                  val v9 = v7._2
                  val v10 = v2(1)
                  sigmaProp(v3._1 == "472b4b6250655368566d597133743677397a24432646294a404d635166546a57" && v3 == v5(0) && v1.value == v4.value && v1.getReg(4) == v4.getReg(4) && v1.getReg(5) == v4.getReg(5) && !v4.getReg(6).isDefined() && v6.tokens == SELF.tokens && v6.propBytes == SELF.propBytes && v6.value >= SELF.value && v6.creationInfo._1 > SELF.creationInfo._1 && !v6.getReg(4).isDefined() && INPUTS.filter({
                      (v11: Box) => 
                        {
                          val v13 = v11.tokens
                          if (v13.size > 0) {
                            val v14 = v11.getReg(7)
                            v13(0)._1 == "3f4428472d4b6150645367566b5970337336763979244226452948404d625165" && v11.getReg(5).get == SELF.creationInfo._1 && v11.getReg(6).get == blake2b256(v4.propBytes) && if (v14.isDefined() && v11.getReg(8).isDefined()) v14.get == v8 && v11.getReg(8).get == v9 else v10._1 == v8 && v10._2 == v9
                          }
                 else false
                        }

                      }
                    ).fold(0)({
                      (v11: (Long, Box)) => 
                        v11._1 + v11._2.tokens(0)._2
                      }
                    ) >= upcast(6))
                }
            "#]],
        )
    }

    #[test]
    fn eip23_ballot_contract() {
        let ergo_tree_bytes = base16::decode("10070580dac409040204020400040204000e206251655468576d5a7134743777217a25432a462d4a404e635266556a586e3272d803d601e4c6a70407d602b2a5e4e3000400d603c672020407eb02cd7201d1edededede6720393c27202c2a793db63087202db6308a792c172027300ededededed91b1a4730191b1db6308b2a47302007303938cb2db6308b2a473040073050001730693e47203720192c17202c1a7efe6c672020561").unwrap();
        let ergo_tree = ErgoTree::sigma_parse_bytes(&ergo_tree_bytes).unwrap();
        check_pretty(
            ergo_tree.proposition().unwrap(),
            expect![[r#"
                {
                  val v1 = SELF.getReg(4).get
                  val v2 = OUTPUTS(getVar(0).get)
                  val v3 = v2.getReg(4)
                  anyOf(
                    proveDlog(v1), 
                    sigmaProp(v3.isDefined() && v2.propBytes == SELF.propBytes && v2.tokens == SELF.tokens && v2.value >= 10000000 && INPUTS.size > 1 && INPUTS(1).tokens.size > 0 && INPUTS(1).tokens(0)._1 == "6251655468576d5a7134743777217a25432a462d4a404e635266556a586e3272" && v3.get == v1 && v2.value >= SELF.value && !v2.getReg(5).isDefined()), 
                  )
                }
            "#]],
        )
    }

    #[test]
    fn eip23_oracle_contract() {
        let ergo_tree_bytes = base16::decode("100a040004000580dac409040004000e20472b4b6250655368566d597133743677397a24432646294a404d635166546a570402040204020402d804d601b2a5e4e3000400d602db63087201d603db6308a7d604e4c6a70407ea02d1ededed93b27202730000b2720373010093c27201c2a7e6c67201040792c172017302eb02cd7204d1ededededed938cb2db6308b2a4730300730400017305938cb27202730600018cb2720373070001918cb27202730800028cb272037309000293e4c672010407720492c17201c1a7efe6c672010561").unwrap();
        let ergo_tree = ErgoTree::sigma_parse_bytes(&ergo_tree_bytes).unwrap();
        check_pretty(
            ergo_tree.proposition().unwrap(),
            expect![[r#"
                {
                  val v1 = OUTPUTS(getVar(0).get)
                  val v2 = v1.tokens
                  val v3 = SELF.tokens
                  val v4 = SELF.getReg(4).get
                  allOf(
                    sigmaProp(v2(0) == v3(0) && v1.propBytes == SELF.propBytes && v1.getReg(4).isDefined() && v1.value >= 10000000), 
                    anyOf(
                      proveDlog(v4), 
                      sigmaProp(INPUTS(0).tokens(0)._1 == "472b4b6250655368566d597133743677397a24432646294a404d635166546a57" && v2(1)._1 == v3(1)._1 && v2(1)._2 > v3(1)._2 && v1.getReg(4).get == v4 && v1.value >= SELF.value && !v1.getReg(5).isDefined()), 
                    ), 
                  )
                }
            "#]],
        )
    }

    #[test]
    fn ageusd_bank_full() {
        // from eip-15 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "MUbV38YgqHy7XbsoXWF5z7EZm524Ybdwe5p9WDrbhruZRtehkRPT92imXer2eTkjwPDfboa1pR3zb3deVKVq3H7Xt98qcTqLuSBSbHb7izzo5jphEpcnqyKJ2xhmpNPVvmtbdJNdvdopPrHHDBbAGGeW7XYTQwEeoRfosXzcDtiGgw97b2aqjTsNFmZk7khBEQywjYfmoDc9nUCJMZ3vbSspnYo3LarLe55mh2Np8MNJqUN9APA6XkhZCrTTDRZb1B4krgFY1sVMswg2ceqguZRvC9pqt3tUUxmSnB24N6dowfVJKhLXwHPbrkHViBv1AKAJTmEaQW2DN1fRmD9ypXxZk8GXmYtxTtrj3BiunQ4qzUCu1eGzxSREjpkFSi2ATLSSDqUwxtRz639sHM6Lav4axoJNPCHbY8pvuBKUxgnGRex8LEGM8DeEJwaJCaoy8dBw9Lz49nq5mSsXLeoC4xpTUmp47Bh7GAZtwkaNreCu74m9rcZ8Di4w1cmdsiK1NWuDh9pJ2Bv7u3EfcurHFVqCkT3P86JUbKnXeNxCypfrWsFuYNKYqmjsix82g9vWcGMmAcu5nagxD4iET86iE2tMMfZZ5vqZNvntQswJyQqv2Wc6MTh4jQx1q2qJZCQe4QdEK63meTGbZNNKMctHQbp3gRkZYNrBtxQyVtNLR8xEY8zGp85GeQKbb37vqLXxRpGiigAdMe3XZA4hhYPmAAU5hpSMYaRAjtvvMT3bNiHRACGrfjvSsEG9G2zY5in2YWz5X9zXQLGTYRsQ4uNFkYoQRCBdjNxGv6R58Xq74zCgt19TxYZ87gPWxkXpWwTaHogG1eps8WXt8QzwJ9rVx6Vu9a5GjtcGsQxHovWmYixgBU8X9fPNJ9UQhYyAWbjtRSuVBtDAmoV1gCBEPwnYVP5GCGhCocbwoYhZkZjFZy6ws4uxVLid3FxuvhWvQrVEDYp7WRvGXbNdCbcSXnbeTrPMey1WPaXX";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = CONTEXT.dataInputs
                  sigmaProp(if (v1.size > 0) {
                    val v2 = v1(0)
                    val v3 = v2.tokens(0)._1 == "011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887f"
                    val v4 = OUTPUTS(0)
                    val v5 = v4.value
                    val v6 = SELF.tokens
                    val v7 = v6(1)
                    val v8 = v7._2
                    val v9 = v4.tokens
                    val v10 = v9(1)
                    val v11 = v10._2
                    val v12 = v8 != v11
                    val v13 = v6(0)
                    val v14 = v13._2
                    val v15 = v9(0)
                    val v16 = v15._2
                    val v17 = v14 != v16
                    val v18 = SELF.getReg(5).get
                    val v19 = v4.getReg(5).get
                    val v20 = SELF.getReg(4).get
                    val v21 = v4.getReg(4).get
                    val v22 = OUTPUTS(1)
                    val v23 = v22.getReg(4).get
                    val v24 = if (v12) 0 else v23
                    val v25 = if (v12) v23 else 0
                    val v26 = SELF.value
                    val v27 = v22.getReg(5).get
                    val v28 = v2.getReg(4).get / 100
                    val v29 = v26 min v20 * v28 max 0
                    val v30 = if (v17) v28 min if (v20 == 0) 9223372036854775807 else v29 / v20 * v24 else {
                      val v30 = v26 - v29
                      if (v30 == 0) 1000000 else if (v18 == 0) 1000000 else v30 / v18 * v25
                    }

                    val v31 = v30 * upcast(2) / 100
                    val v32 = v21 * v28
                    val v33 = if (HEIGHT > 460000) 800 else 1000000000
                    val v34 = if (v32 == 0) v33 else v5 * 100 / v32
                    v3 && v5 >= 10000000 && v4.propBytes == SELF.propBytes && v12 || v17 && !v12 && v17 && v8 + v18 == v11 + v19 && v14 + v20 == v16 + v21 && v20 + v24 == v21 && v18 + v25 == v19 && v26 + v27 == v5 && v21 >= 0 && v19 >= 0 && v15._1 == v13._1 && v10._1 == v7._1 && v9(2)._1 == v6(2)._1 && v27 == v30 + if (v31 < 0) -v31 else v31 && if (v17) if (v24 > 0) v34 >= 400 else true else if (v25 > 0) v34 <= v33 else v34 >= 400 && v3
                  }
                 else false || INPUTS(0).tokens(0)._1 == "239c170b7e82f94e6b05416f14b8a2a57e0bfff0e3c93f4abbcd160b6a5b271a")
                }
            "#]],
        )
    }
}
