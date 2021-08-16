#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::rc::Rc;

    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::address::AddressEncoder;
    use ergotree_ir::address::NetworkPrefix;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;

    #[test]
    fn simplified_age_usd_bank_contract() {
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
        let script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
            .try_into()
            .unwrap();
        assert!(res);
    }

    #[test]
    fn ageusd_bank_full() {
        // from eip-15 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "MUbV38YgqHy7XbsoXWF5z7EZm524Ybdwe5p9WDrbhruZRtehkRPT92imXer2eTkjwPDfboa1pR3zb3deVKVq3H7Xt98qcTqLuSBSbHb7izzo5jphEpcnqyKJ2xhmpNPVvmtbdJNdvdopPrHHDBbAGGeW7XYTQwEeoRfosXzcDtiGgw97b2aqjTsNFmZk7khBEQywjYfmoDc9nUCJMZ3vbSspnYo3LarLe55mh2Np8MNJqUN9APA6XkhZCrTTDRZb1B4krgFY1sVMswg2ceqguZRvC9pqt3tUUxmSnB24N6dowfVJKhLXwHPbrkHViBv1AKAJTmEaQW2DN1fRmD9ypXxZk8GXmYtxTtrj3BiunQ4qzUCu1eGzxSREjpkFSi2ATLSSDqUwxtRz639sHM6Lav4axoJNPCHbY8pvuBKUxgnGRex8LEGM8DeEJwaJCaoy8dBw9Lz49nq5mSsXLeoC4xpTUmp47Bh7GAZtwkaNreCu74m9rcZ8Di4w1cmdsiK1NWuDh9pJ2Bv7u3EfcurHFVqCkT3P86JUbKnXeNxCypfrWsFuYNKYqmjsix82g9vWcGMmAcu5nagxD4iET86iE2tMMfZZ5vqZNvntQswJyQqv2Wc6MTh4jQx1q2qJZCQe4QdEK63meTGbZNNKMctHQbp3gRkZYNrBtxQyVtNLR8xEY8zGp85GeQKbb37vqLXxRpGiigAdMe3XZA4hhYPmAAU5hpSMYaRAjtvvMT3bNiHRACGrfjvSsEG9G2zY5in2YWz5X9zXQLGTYRsQ4uNFkYoQRCBdjNxGv6R58Xq74zCgt19TxYZ87gPWxkXpWwTaHogG1eps8WXt8QzwJ9rVx6Vu9a5GjtcGsQxHovWmYixgBU8X9fPNJ9UQhYyAWbjtRSuVBtDAmoV1gCBEPwnYVP5GCGhCocbwoYhZkZjFZy6ws4uxVLid3FxuvhWvQrVEDYp7WRvGXbNdCbcSXnbeTrPMey1WPaXX";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let _script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        // let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
        //     .try_into()
        //     .unwrap();
        // assert!(!res);
    }

    #[test]
    fn ageusd_update() {
        // from eip-15 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "VLyjpv3dse3PbatT83GnDkBQasGqY52dAEdi9XpXhuSUn1FS1Tm7XxtAgmBiqY9pJXtEAsDKwX9ygSjrFu7vnUQZudhC2sSmxhxqgD3ZxJ2VsGwmPG77F6EiEZhcq71oqEq31y9XvCCXL5nqqszdENPAVhu7xT296qZ7w1x6hmwdh9ZE89bjfgbhfNYopoqsCaNLWYHJ12TDSY93kaGqCVKSu6gEF1gLpXBfRCnAPPxYswJPmK8oWDn8PKrUGs3MjVsj6bGXiW3VTGP4VsNH8YSSkjyj1FZ9azLsyfnNJ3zah2zUHdCCqY6PjH9JfHf9joCPf6TusvXgr71XWvh5e2HPEPQr4eJMD4S96cGTiSs3J5XcRd1tCDYoiis8nxv99zFFhHgpqXHgeqjhJ5sPot9eRYTsmm4cRTVLXYAiuKPS2qW5";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let _script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        // let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
        //     .try_into()
        //     .unwrap();
        // assert!(!res);
    }

    #[test]
    fn ageusd_ballot() {
        // from eip-15 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "22ELWBHzyWGjPRE48ZJDfFmD24myYdG3vHz8CipSS7rgE65ABmEj9QJiy3rG2PTJeCaZw9VX56GY6uoA3hQch7i5BfFU3AprUWTABi4X1VWtRdK9yrYJkmN6fq8hGfvmWTrsyh4fXZoGETpLuXQViYo194ajej2h7dr3oqNATdMskSXzxJi83bFdAvQ";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let _script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        // let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
        //     .try_into()
        //     .unwrap();
        // assert!(!res);
    }

    #[test]
    fn amm_simple_pool() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "k6fD5ht5e1itDejPFV2VzAoHv478KQCbDnLAL6XUVeEu8KDaboCVZAoFz2AtMoLqM3CgQfr2TZhpwz7K96AgwTXDvBVeTchJ31jjD46Di1W67H8wwFcivnY62UB6L7HWzCkbYuiZaAq2qSJta5Twt4A2Aaoy7xViWcyLUVNAyQYDJXKhVBAGwp76i2too5yWUmEU4zt9XnjJAUt1FFfurNtTNHNPDbqmTRE4crz347q6rfbvkMmg9Jtk9rSiPCQpKjdbZVzUnP4CUw6AvQH6rZXxgNMktAtjQdHhCnrCmf78FwCKqYS54asKd1MFgYNT4NzPwmdZF6JtQt1vvkjZXqpGkjy33xxDNYy8JZS8eeqVgZErPeJ1aj4aaK8gvmApUgGStMDFeFYjuQqZiZxEAHNdAXDg7hyGnmfzA6Hj9zcB7p9nKCDNhEQEMPL1kMG5aXvt2HUPXqiCkLrv596DaGmRMN3gMJaj1T1AfMYNwZozcJ9uUSK4i6Xham28HWAekTtDPhobnmjvkubwLVTtvUumWHtDWFxYSJPF7vqzgZqg6Y5unMF";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        assert!(addr.script().unwrap().proposition().is_ok());
        let _script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        // let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
        //     .try_into()
        //     .unwrap();
        // assert!(!res);
    }

    #[test]
    fn amm_simple_swap() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "cLPHJ3MHuKAHoCUwGhcEFw5sWJqvPwFyKxTRj1aUoMwgAz78Fg3zLXRhBup9Te1WLau1gZXNmXvUmeXGCd7QLeqB7ArrT3v5cg26piEtqymM6j2SkgYVCobgoAGKeTf6nMLxv1uVrLdjt1GnPxG1MuWj7Es7Dfumotbx9YEaxwqtTUC5SKsJc9LCpAmNWRAQbU6tVVEvmfwWivrGoZ3L5C4DMisxN3U";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let _script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        // let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
        //     .try_into()
        //     .unwrap();
        // assert!(!res);
    }

    #[test]
    fn amm_conc_pool_root() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "3STRfQWC9Xb5wAxBiEQ74uTFSemk1oHn43mwj9tMCeu2a3A4kie1bY2qsCdRaEmdQoq3B4tXQuzq9nm84A8PmBgCzgGDEZf2pgYoAUc6krZxUY3rvKWW44ZpzN3u5bFRpKDo6rxKtxX2tw99xmfyfaVBejgDaTfsib2PSVsu9hrLQ3SouECWHQMjDA3Pi8ZuCvQeW8GDkZfHPr3SgwaxY1jpY2njsmf3JBASMoVZ6Mfpg63Q6mBno7mKUSCE7vNHHUZe2V7JEikwjPkaxSWxnwy3J17faGtiEHZLKiNQ9WNtsJLbdVp56dQGfC2zaiXjhx1XJK6m4Nh2M8yEvSuBzanRBAJqrNseGS97tk2iLqqfHrqqmmDsHY3mujCURky4SLr7YLk4B";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let _script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        // let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
        //     .try_into()
        //     .unwrap();
        // assert!(!res);
    }

    #[test]
    fn amm_conc_pool_boot() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "6Mv73vd1MnJp6AQg5vHGP9nujFc3Y1PL5gzeUt9PzCaUiQug7ueQGU1bDkmFkCspq4LU8j3T8yY6UyJQKSfah5qEDzjx8QCJF47NBG5jxgPxmBHkM6cUgnYa5ngzn9jrpAn379UC7o5nugTg3HYWZGk3APMcRftkrC3EgroiVMEmSkDcDwaebkNWKfKe3JXgewoTrgZ2YLMafr3JfX47C1zddoWDhS8TWryQYEprkP334eisuh1Fr2iNTW9ruV6m38cRkfRfzSBHYq45mvNLH7JQo6uQZ4NFPx4t27Q5A3mSqCpk7ATThFcQmc2w3Pp2F6xL87c94gxk83G8UEqkAhmaNfoj19zji9rxqRzq9gJeTLBraHR2DchKtahH8HhFPg5DZ4SjwJ4MHqTDF";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let _script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        // let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
        //     .try_into()
        //     .unwrap();
        // assert!(!res);
    }

    #[test]
    fn amm_conc_pool() {
        // from eip-14 https://github.com/ergoplatform/eips/pull/27/files
        let p2s_addr_str = "AhCu1UkNT4c9q3B2Lb7gNgvZWCdXL8iYgmNxTYiy4S3wgKWFFW6kz9v7pvY8NqC7g4wgXXwzJY1fQVn2xrLkiyiQWsorq5dR7d5KnDAY43H4GvSVjaDciadXCSHCb8jgk8mFSQCwoZHweLmMJ25312wT85AySJgYUuzdUxMz4EnQpiwZR2XVZq3M81gycuqP9gUryryjN4J1cAF3yL3kZR3rREubBvJ2CY5hF74Xaj2jwajivkESkqq22ieWWG2sK7dk1A7KHr1MmiXGcUBAMMGPAu3mVCeFW9SongxP9hodnJThLknjWRBBBC6wq5jNkSdHrMbdaQM3XesXqGTk9KwWpnSL92E96muU2k8FQbo5isps1r5ciYVrFptfEAC3tWbwcVmRKtrgxtCex6bP5aBZYjaH6L9QQbkYriDAcQ1iZcpf3hHCqURjRXL7i72C3aGBwzzspQvhLof6x4f4gPxTCtF1bNUxddUL6DJ1PbQWzVH8taivjhHohis6sRn3Akvv4xaZRJdKZ8rDuiounRKNXi8VoNgVEZbSFYtfweRSdsiXJCkhtehLWdtFTk1eg7djASdBGKaguvtEBcGaAALVDUoH479VskPUQ6hrfS7KcWrATBdb8sf4W5MFpx7UNitzq2fzSKC96mQRUzy5uELe7Y7vexm5ArNEyr6ARkypZypSzJ2CEifjVxxRBEWVtbdqHrwP4gWv6cMdbqFWwuXAw2BZQnWpZFtKAGQ9m";
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let addr = encoder.parse_address_from_str(p2s_addr_str).unwrap();
        let _script: Rc<Expr> = addr.script().unwrap().proposition().unwrap();
        // dbg!(&script);
        // let res: bool = eval_out_wo_ctx::<SigmaProp>(script.as_ref())
        //     .try_into()
        //     .unwrap();
        // assert!(!res);
    }
}
