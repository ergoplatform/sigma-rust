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

    #[test]
    fn ageusd_update() {
        // from eip-15 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "VLyjpv3dse3PbatT83GnDkBQasGqY52dAEdi9XpXhuSUn1FS1Tm7XxtAgmBiqY9pJXtEAsDKwX9ygSjrFu7vnUQZudhC2sSmxhxqgD3ZxJ2VsGwmPG77F6EiEZhcq71oqEq31y9XvCCXL5nqqszdENPAVhu7xT296qZ7w1x6hmwdh9ZE89bjfgbhfNYopoqsCaNLWYHJ12TDSY93kaGqCVKSu6gEF1gLpXBfRCnAPPxYswJPmK8oWDn8PKrUGs3MjVsj6bGXiW3VTGP4VsNH8YSSkjyj1FZ9azLsyfnNJ3zah2zUHdCCqY6PjH9JfHf9joCPf6TusvXgr71XWvh5e2HPEPQr4eJMD4S96cGTiSs3J5XcRd1tCDYoiis8nxv99zFFhHgpqXHgeqjhJ5sPot9eRYTsmm4cRTVLXYAiuKPS2qW5";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = INPUTS(1)
                  val v2 = v1.tokens
                  val v3 = OUTPUTS(1)
                  val v4 = SELF.id
                  val v5 = OUTPUTS(0)
                  sigmaProp(v2.size == 3 && v2(2)._1 == "7d672d1def471720ca5782fd6473e47e796d9ac0c138d9911346f118b2f6d9d9" && v2 == v3.tokens && v1.value == v3.value && v1.getReg(4).get == v3.getReg(4).get && v1.getReg(5).get == v3.getReg(5).get && v4 == INPUTS(0).id && SELF.tokens == v5.tokens && SELF.propBytes == v5.propBytes && v5.value >= SELF.value && INPUTS.filter({
                      (v6: Box) => 
                        {
                          val v8 = v6.tokens
                          v8.size > 0 && v8(0)._1 == "f7995f212216fcf21854f56df7a9a0a9fc9b7ae4c0f1cc40f5b406371286a5e0" && v6.getReg(6).get == v4 && v6.getReg(7).get == blake2b256(v3.propBytes)
                        }

                      }
                    ).fold(0)({
                      (v6: (Long, Box)) => 
                        v6._1 + v6._2.tokens(0)._2
                      }
                    ) >= upcast(3))
                }
            "#]],
        )
    }

    #[test]
    fn ageusd_ballot() {
        // from eip-15 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "22ELWBHzyWGjPRE48ZJDfFmD24myYdG3vHz8CipSS7rgE65ABmEj9QJiy3rG2PTJeCaZw9VX56GY6uoA3hQch7i5BfFU3AprUWTABi4X1VWtRdK9yrYJkmN6fq8hGfvmWTrsyh4fXZoGETpLuXQViYo194ajej2h7dr3oqNATdMskSXzxJi83bFdAvQ";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = OUTPUTS(INPUTS.indexOf(SELF0))
                  val v2 = SELF.getReg(4).get
                  allOf(
                    sigmaProp(v1.getReg(4).get == v2 && v1.propBytes == SELF.propBytes && v1.tokens == SELF.tokens && v1.value >= SELF.value), 
                    anyOf(
                      proveDlog(v2), 
                      sigmaProp(INPUTS(0).tokens(0)._1 == "239c170b7e82f94e6b05416f14b8a2a57e0bfff0e3c93f4abbcd160b6a5b271a" && !v1.getReg(7).isDefined()), 
                    ), 
                  )
                }
            "#]],
        )
    }

    #[test]
    fn amm_simple_pool() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "k6fD5ht5e1itDejPFV2VzAoHv478KQCbDnLAL6XUVeEu8KDaboCVZAoFz2AtMoLqM3CgQfr2TZhpwz7K96AgwTXDvBVeTchJ31jjD46Di1W67H8wwFcivnY62UB6L7HWzCkbYuiZaAq2qSJta5Twt4A2Aaoy7xViWcyLUVNAyQYDJXKhVBAGwp76i2too5yWUmEU4zt9XnjJAUt1FFfurNtTNHNPDbqmTRE4crz347q6rfbvkMmg9Jtk9rSiPCQpKjdbZVzUnP4CUw6AvQH6rZXxgNMktAtjQdHhCnrCmf78FwCKqYS54asKd1MFgYNT4NzPwmdZF6JtQt1vvkjZXqpGkjy33xxDNYy8JZS8eeqVgZErPeJ1aj4aaK8gvmApUgGStMDFeFYjuQqZiZxEAHNdAXDg7hyGnmfzA6Hj9zcB7p9nKCDNhEQEMPL1kMG5aXvt2HUPXqiCkLrv596DaGmRMN3gMJaj1T1AfMYNwZozcJ9uUSK4i6Xham28HWAekTtDPhobnmjvkubwLVTtvUumWHtDWFxYSJPF7vqzgZqg6Y5unMF";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = OUTPUTS(0)
                  val v2 = v1.tokens
                  val v3 = SELF.tokens
                  val v4 = v2(0)
                  val v5 = v3(0)
                  val v6 = v2(2)
                  val v7 = v3(2)
                  val v8 = v2(3)
                  val v9 = v3(3)
                  val v10 = 1000000000000000000 - v5._2
                  val v11 = 1000000000000000000 - v4._2 - v10
                  val v12 = v7._2
                  val v13 = v6._2 - v12
                  val v14 = v13 > 0
                  val v15 = v9._2
                  val v16 = upcast(v15)
                  val v17 = upcast(v13)
                  val v18 = v8._2 - v15
                  val v19 = upcast(v12)
                  val v20 = upcast(v18)
                  val v21 = upcast(v10)
                  val v22 = upcast(v11) / v21
                  sigmaProp(v1.propBytes == SELF.propBytes && v1.value >= SELF.value && v2(1) == v3(1) && v4._1 == v5._1 && v6._1 == v7._1 && v8._1 == v9._1 && if (v11 == 0) if (v14) v16 * v17 * BigInt256(Int256(997)) >= upcast(-v18) * v19 * BigInt256(Int256(1000)) + upcast(v13 * 997) else v19 * v20 * BigInt256(Int256(997)) >= upcast(-v13) * v16 * BigInt256(Int256(1000)) + upcast(v18 * 997) else if (v14 && v18 > 0) upcast(-v11) <= v17 * v21 / v19 min v20 * v21 / v16 else v17 >= v22 * v19 && v20 >= v22 * v16)
                }
            "#]],
        )
    }

    #[test]
    fn amm_simple_swap() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "cLPHJ3MHuKAHoCUwGhcEFw5sWJqvPwFyKxTRj1aUoMwgAz78Fg3zLXRhBup9Te1WLau1gZXNmXvUmeXGCd7QLeqB7ArrT3v5cg26piEtqymM6j2SkgYVCobgoAGKeTf6nMLxv1uVrLdjt1GnPxG1MuWj7Es7Dfumotbx9YEaxwqtTUC5SKsJc9LCpAmNWRAQbU6tVVEvmfwWivrGoZ3L5C4DMisxN3U";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = INPUTS(0).tokens
                  val v2 = v1(2)
                  val v3 = SELF.tokens(0)
                  val v4 = v3._1
                  val v5 = v1(3)
                  val v6 = v3._2
                  sigmaProp(v2._1 == v4 || v5._1 == v4 && OUTPUTS.exists({
                      (v7: Box) => 
                        {
                          val v9 = v7.tokens(0)._2
                          v9 >= upcast(1000) && upcast(v5._2) * upcast(v6) * BigInt256(Int256(997)) <= upcast(v9) * upcast(v2._2) * BigInt256(Int256(1000)) + upcast(v6 * 997)
                        }

                      }
                ))
                  }
            "#]],
        )
    }

    #[test]
    fn amm_conc_pool_root() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "3STRfQWC9Xb5wAxBiEQ74uTFSemk1oHn43mwj9tMCeu2a3A4kie1bY2qsCdRaEmdQoq3B4tXQuzq9nm84A8PmBgCzgGDEZf2pgYoAUc6krZxUY3rvKWW44ZpzN3u5bFRpKDo6rxKtxX2tw99xmfyfaVBejgDaTfsib2PSVsu9hrLQ3SouECWHQMjDA3Pi8ZuCvQeW8GDkZfHPr3SgwaxY1jpY2njsmf3JBASMoVZ6Mfpg63Q6mBno7mKUSCE7vNHHUZe2V7JEikwjPkaxSWxnwy3J17faGtiEHZLKiNQ9WNtsJLbdVp56dQGfC2zaiXjhx1XJK6m4Nh2M8yEvSuBzanRBAJqrNseGS97tk2iLqqfHrqqmmDsHY3mujCURky4SLr7YLk4B";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = OUTPUTS(0)
                  val v2 = SELF.tokens(0)
                  val v3 = v2._1
                  val v4 = SELF.getReg(4).get
                  val v5 = SELF.getReg(5).get
                  val v6 = SELF.getReg(6).get
                  val v7 = OUTPUTS(1)
                  val v8 = v7.tokens
                  val v9 = v7.getReg(6).get
                  val v10 = v5._2
                  val v11 = v5._1
                  val v12 = v7.getReg(5).get
                  val v13 = v7.getReg(7).get
                  sigmaProp(v1.propBytes == SELF.propBytes && v1.value >= SELF.value && v1.tokens(0) == (, v3, v2._2 - 1) && v1.getReg(4).get == v4 && v1.getReg(5).get == v5 && v1.getReg(6).get == v6 && v7.getReg(4).get == v4 && v7.getReg(8).get == v6 && v8(1)._1 == v3 && v8(0) == (, SELF.id, 1000000000000000000) && v9._1 * v10 == v9._2 * v11 * v12 && v13._1 * v10 == v13._2 * v11 * v12 + 1)
                }
            "#]],
        )
    }

    #[test]
    fn amm_conc_pool_boot() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "6Mv73vd1MnJp6AQg5vHGP9nujFc3Y1PL5gzeUt9PzCaUiQug7ueQGU1bDkmFkCspq4LU8j3T8yY6UyJQKSfah5qEDzjx8QCJF47NBG5jxgPxmBHkM6cUgnYa5ngzn9jrpAn379UC7o5nugTg3HYWZGk3APMcRftkrC3EgroiVMEmSkDcDwaebkNWKfKe3JXgewoTrgZ2YLMafr3JfX47C1zddoWDhS8TWryQYEprkP334eisuh1Fr2iNTW9ruV6m38cRkfRfzSBHYq45mvNLH7JQo6uQZ4NFPx4t27Q5A3mSqCpk7ATThFcQmc2w3Pp2F6xL87c94gxk83G8UEqkAhmaNfoj19zji9rxqRzq9gJeTLBraHR2DchKtahH8HhFPg5DZ4SjwJ4MHqTDF";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = OUTPUTS(0)
                  val v2 = v1.tokens
                  val v3 = SELF.tokens
                  val v4 = v1.getReg(5).get
                  val v5 = v1.getReg(6).get
                  val v6 = v2(3)
                  val v7 = v6._2
                  val v8 = upcast(v7)
                  val v9 = v1.getReg(7).get
                  val v10 = upcast(v9)
                  val v11 = v2(2)
                  val v12 = v3(0)
                  sigmaProp(true && v1.value >= SELF.value && v2(0) == (, SELF.id, 1) && v2(1) == (, v3(1)._1, 1) && v1.getReg(4).get == SELF.getReg(4).get && v4 == SELF.getReg(6).get && v5 == SELF.getReg(7).get && (, v6._1, v2(4)._1) == SELF.getReg(8).get && v8 * v8 == v10 * v10 && if (v11._1 == v12._1) v11._2 else 0 >= v12._2 - v9 && v7 * upcast(v4._2) >= v7 * upcast(v4._1) && v7 * upcast(v5._2) < v7 * upcast(v5._1))
                }
            "#]],
        )
    }

    #[test]
    fn amm_conc_pool() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "AhCu1UkNT4c9q3B2Lb7gNgvZWCdXL8iYgmNxTYiy4S3wgKWFFW6kz9v7pvY8NqC7g4wgXXwzJY1fQVn2xrLkiyiQWsorq5dR7d5KnDAY43H4GvSVjaDciadXCSHCb8jgk8mFSQCwoZHweLmMJ25312wT85AySJgYUuzdUxMz4EnQpiwZR2XVZq3M81gycuqP9gUryryjN4J1cAF3yL3kZR3rREubBvJ2CY5hF74Xaj2jwajivkESkqq22ieWWG2sK7dk1A7KHr1MmiXGcUBAMMGPAu3mVCeFW9SongxP9hodnJThLknjWRBBBC6wq5jNkSdHrMbdaQM3XesXqGTk9KwWpnSL92E96muU2k8FQbo5isps1r5ciYVrFptfEAC3tWbwcVmRKtrgxtCex6bP5aBZYjaH6L9QQbkYriDAcQ1iZcpf3hHCqURjRXL7i72C3aGBwzzspQvhLof6x4f4gPxTCtF1bNUxddUL6DJ1PbQWzVH8taivjhHohis6sRn3Akvv4xaZRJdKZ8rDuiounRKNXi8VoNgVEZbSFYtfweRSdsiXJCkhtehLWdtFTk1eg7djASdBGKaguvtEBcGaAALVDUoH479VskPUQ6hrfS7KcWrATBdb8sf4W5MFpx7UNitzq2fzSKC96mQRUzy5uELe7Y7vexm5ArNEyr6ARkypZypSzJ2CEifjVxxRBEWVtbdqHrwP4gWv6cMdbqFWwuXAw2BZQnWpZFtKAGQ9m";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = OUTPUTS(0)
                  val v2 = v1.tokens
                  val v3 = SELF.tokens
                  val v4 = v2(2)
                  val v5 = v3(2)
                  val v6 = v2(3)
                  val v7 = v3(3)
                  val v8 = v2(4)
                  val v9 = v3(4)
                  val v10 = SELF.getReg(4).get
                  val v11 = SELF.getReg(5).get
                  val v12 = SELF.getReg(6).get
                  val v13 = 1000000000000000000 - v5._2
                  val v14 = 1000000000000000000 - v4._2 - v13
                  val v15 = v6._2
                  val v16 = upcast(v15)
                  val v17 = v8._2
                  val v18 = upcast(v17)
                  val v19 = v16 * upcast(v11._2) >= v18 * upcast(v11._1) && v16 * upcast(v12._2) < v18 * upcast(v12._1)
                  val v20 = v7._2
                  val v21 = v15 - v20
                  val v22 = v21 > 0
                  val v23 = v9._2
                  val v24 = upcast(v23)
                  val v25 = upcast(v21)
                  val v26 = v17 - v23
                  val v27 = upcast(v20)
                  val v28 = 1000
                  val v29 = upcast(v26)
                  val v30 = upcast(v13)
                  val v31 = upcast(v14) / v30
                  sigmaProp(v1.propBytes == SELF.propBytes && v1.value >= SELF.value && v2(0) == v3(0) && v2(1) == v3(1) && v4._1 == v5._1 && v6._1 == v7._1 && v8._1 == v9._1 && v1.getReg(4).get == v10 && v1.getReg(5).get == v11 && v1.getReg(6).get == v12 && if (v14 == 0) if (v22) v24 * v25 * upcast(v10) >= upcast(-v26) * v27 * upcast(v28) + upcast(v21 * upcast(v10)) else v27 * v29 * upcast(v10) >= upcast(-v21) * v24 * upcast(v28) + upcast(v26 * upcast(v10)) && v19 else if (v22 && v26 > 0) upcast(-v14) <= v25 * v30 / v27 min v29 * v30 / v24 && v19 else v25 >= v31 * v27 && v29 >= v31 * v24)
                }
            "#]],
        )
    }

    #[test]
    fn eip_22_auction() {
        // from https://github.com/ergoplatform/eips/blob/adbe21512cadf51a2d9af8406cfd418f95335899/eip-0022.md
        let p2s_addr_str = "GE68RH3VEgW6b4kN3GhYigrLxoXr9jMgMpmm3KnXJaYq1PzHieYhz7Uka86idxvBWLRLmpiA3HrxHPsX1jzQZEv5yaRDueiJqks1keM7iB1eYWMEVRUUq1MLFzdA1FHQwCfSakM3Uc8uBPqk2icxhoXvw1CVbUVtFCzcPrZzf8Jjf8gS5bCFpWQscHo14HTsdBxyV3dwL6wKu8LP8FuWJE7qCEgX9ToEiztH4ZLmFwBejnUFrCQqjLVLWpdgdnAXVyewiX9DxXKJKL4wNqhPUrYjmHEVvpZAezXjzfVMr7gKupTqAgx2AJYGh4winEDeYq9MVshX8xjJweGhbAm2RXN1euQpoepFaKqfrT2mQBTmr6edbbzYg6VJ7DoSCDzmcUupFAmZMjMiaUbgtyz2VEbPEKsmAFrZ6zdB5EUxhiYZMd6KdstsJwZCgKJSSCShTgpfqNLCdpR9JbZFQpA1uhUkuLMPvGi74V5EwijTEEtjmTVcWcVhJKv4GDr1Lqe2bMPq4jfEfqvemaY8FcrCsCSi2LZoQUeJ9VrBeotGTKccq8JhwnvNGhLUUrrm32v3bhU82jbtVBVFRD3FSv5hhS6pKHtTevjwuG7JWoR3LN7279A7zQGJWmkSWDoEhHjgxseqZ2G5bLB7ZVEzKM261QhwMwmXA1eWgq8zdBH1u9kFC9bMQ812q2DPZTuhzpBWJh74UGwaEgZLhnUrDKT58cEa4R3kfWyGCMoNw78q1E3a2eKDz8Va5wnixzT2SZFHU8DfHjPSz5rm8Mr3YxgRC6GzaasPDxTrZjuMJHU2exhqsoFvur7Q";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let expr = addr.script().unwrap().proposition().unwrap();
        check_pretty(
            expr,
            expect![[r#"
                {
                  val v1 = OUTPUTS(0)
                  val v2 = CONTEXT.preHeader.timestamp
                  val v3 = SELF.getReg(7).get
                  val v4 = SELF.tokens
                  val v5 = v4.size
                  val v6 = v5 == 1
                  val v7 = {
                      (v7: Box) => 
                        if (v6) v7.value else v7.tokens(1)._2
                      }

                    val v8 = v7(SELF)
                    val v9 = SELF.getReg(6).get
                    val v10 = SELF.getReg(8).get
                    val v11 = Coll[Coll[Byte]]()
                    val v12 = OUTPUTS(1)
                    val v13 = SELF.getReg(5).get
                    val v14 = SELF.getReg(4).get
                    val v15 = CONTEXT.dataInputs(0)
                    val v16 = v15.getReg(8).get
                    sigmaProp(v1.value >= SELF.value && v2 < v3 && v1.tokens(0) == v4(0) && {
                      val v17 = v7(v1)
                      v17 >= v8 + v9 || v10 != -1 && v17 >= v10
                    }
                 && v1.propBytes == SELF.propBytes && v5 == v1.tokens.size && {
                        (v17: Box) => 
                          if (v6) v11 else v17.tokens(1)._1
                        }
                (SELF) == {
                          (v17: Box) => 
                            if (v6) v11 else v17.tokens(1)._1
                          }
                (v1) && v12.propBytes == v13 && v7(v12) >= v8 && v1.getReg(4).get == v14 && v1.getReg(5).get.size > 0 && v1.getReg(6).get == v9 && v1.getReg(7).get == if (v3 - v2 <= v16(0)) v3 + v16(1) else v3 && v1.getReg(8).get == v10 && v1.getReg(9) == SELF.getReg(9) || if (OUTPUTS.size == 5) {
                          val v17 = OUTPUTS(2)
                          val v18 = v8 / upcast(v15.getReg(4).get)
                          val v19 = v4(0)
                          val v20 = v8 / upcast(v15.getReg(6).get)
                          val v21 = OUTPUTS(3)
                          val v22 = v21.getReg(4).get
                          v2 >= v3 || v8 >= v10 && v10 != -1 && v7(v17) >= v18 && v17.propBytes == v15.getReg(5).get && v1.tokens(0) == v19 && v1.propBytes == v13 && v7(v12) >= v8 - v18 - v20 - if (v6) v15.getReg(7).get * 2 else 0 && v12.propBytes == v14 && blake2b256(v22.bytes) == v19._1 && v7(v21) >= v20 && v21.propBytes == v22.propBytes
                        }
                 else false)
                      }
            "#]],
        )
    }
}
