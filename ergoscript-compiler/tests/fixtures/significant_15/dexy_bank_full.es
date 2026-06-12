{
  // Bank script
  //
  // This box: (Bank box)
  //
  // TOKENS
  //   tokens(0): bankNFT identifying the box
  //   tokens(1): dexyUSD tokens to be emitted
  // REGISTERS
  //   None
  //
  // TRANSACTIONS
  //
  // [1] Arbitrage Mint
  //   Input         |  Output         |   Data-Input
  // ------------------------------------------------
  // 0 ArbitrageMint |  ArbitrageMint  |   Oracle
  // 1 Bank          |  Bank           |   LP
  // 2 Buyback       |  Buyback        |
  //
  // [2] Free Mint
  //   Input    |  Output   |   Data-Input
  // -------------------------------------
  // 0 FreeMint |  FreeMint |   Oracle
  // 1 Bank     |  Bank     |   LP
  // 2 Buyback  |  Buyback  |
  //
  // [3] Intervention
  //   Input         |  Output        |   Data-Input
  // -----------------------------------------------
  // 0 LP            |  LP            |   Oracle
  // 1 Bank          |  Bank          |   Tracking (98%)
  // 2 Intervention  |  Intervention  |
  //
  // [4] Payout
  //   Input         |  Output        |   Data-Input
  // -----------------------------------------------
  // 0 Payout        |  Payout        |   Oracle
  // 1 Bank          |  Bank          |
  // 2 Buyback       |  Buyback       |

  // [5] Update
  //   Input         |  Output        |   Data-Input
  // -----------------------------------------------
  // 0 Update        |  Update        |
  // 1 Bank          |  Bank          |

  // This box emits DexyUSD. The contract only enforces some basic rules (such as the contract and token Ids) are preserved.
  // It does not does not encode the emission logic. It just requires certain boxes in the inputs to contain certain NFTs.
  // Those boxes in turn encode the emission logic (and logic for other auxiliary flows, such as intervention).
  // The minting logic (that emits Dexy tokens) is encoded in the FreeMint and ArbitrageMint boxes

  // Oracle data:
  // R4 of the oracle contains the rate "nanoErgs per USD" in Long format

  // inputs indices
  val mintInIndex = 0         // 1st input is mint (or LP box in case of intervention, which we ensure in intervention box)
  val interventionInIndex = 2 // 3rd input is intervention box
  val payoutInIndex = 0       // 1st input is payout box

  // outputs indices
  val selfOutIndex = 1        // 2nd output is self copy

  val freeMintNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val arbitrageMintNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val interventionNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val payoutNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000004")

  val successor = OUTPUTS(selfOutIndex)

  val validSuccessor = successor.tokens(0) == SELF.tokens(0)                && // NFT preserved
                       successor.propositionBytes == SELF.propositionBytes  && // script preserved
                       successor.tokens(1)._1 == SELF.tokens(1)._1             // dexyUSD token Id preserved (but amount will change)

  val validMint = INPUTS(mintInIndex).tokens(0)._1 == freeMintNFT        ||
                  INPUTS(mintInIndex).tokens(0)._1 == arbitrageMintNFT

  val validIntervention = INPUTS(interventionInIndex).tokens(0)._1 == interventionNFT

  val validPayout = INPUTS(payoutInIndex).tokens(0)._1 == payoutNFT

  val updateNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000005")
  val validUpdate = INPUTS(0).tokens(0)._1 == updateNFT

  sigmaProp((validSuccessor && (validMint || validIntervention || validPayout)) || validUpdate)
}