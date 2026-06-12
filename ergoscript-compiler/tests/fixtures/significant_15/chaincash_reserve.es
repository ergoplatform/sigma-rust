{
    // Contract for reserve (in ERG only)

    // Data:
    //  - token #0 - identifying singleton token
    //  - R4 - signing key (as a group element)
    //  - R5 - tree of all the note tokens issued TODO: check preservation in actions
    //
    // Actions:
    //  - redeem note (#0)
    //  - top up      (#1)
    //  - mint note (#2)

    val v = getVar[Byte](0).get
    val action = v / 10
    val index = v % 10

    val ownerKey = SELF.R4[GroupElement].get // reserve owner's key, used in notes and unlock/lock/refund actions
    val selfOut = OUTPUTS(index)

    // common checks for all the paths (not incl. ERG value check)
    val selfPreserved =
            selfOut.propositionBytes == SELF.propositionBytes &&
            selfOut.tokens == SELF.tokens &&
            selfOut.R4[GroupElement].get == SELF.R4[GroupElement].get

    if (action == 0) {
      // redemption path

      // OUTPUTS:
      // #1 - receipt
      // #2 - buyback

      val g: GroupElement = groupGenerator

      // if set, re-redemption against receipt data is done, otherwise, a note is redeemed
      val receiptMode = getVar[Boolean](4).get

      // read note data if receiptMode == false, receipt data otherwise
      val noteInput = INPUTS(index)
      val noteTokenId = noteInput.tokens(0)._1
      val noteValue = noteInput.tokens(0)._2 // 1 token == 1 mg of gold
      val history = noteInput.R4[AvlTree].get
      val reserveId = SELF.tokens(0)._1

      // oracle provides gold price in nanoErg per kg in its R4 register
      val goldOracle = CONTEXT.dataInputs(0)
      // todo: externalize oracle NFT id
      // the ID below is from the mainnet
      val properOracle = goldOracle.tokens(0)._1 == fromBase16("3c45f29a5165b030fdb5eaf5d81f8108f9d8f507b31487dd51f4ae08fe07cf4a")
      val oracleRate = goldOracle.R4[Long].get / 1000000 // normalize to nanoerg per mg of gold

      // 2% redemption fee
      val maxToRedeem = noteValue * oracleRate * 98 / 100
      val redeemed = SELF.value - selfOut.value

      // 0.2% going to buyback contract to support oracles network
      val buyBackCorrect = if (redeemed > 0) {
        val toOracle = redeemed * 2 / 1000
        // todo: externalize buyback NFT id
        // the ID below is from the mainnet
        val buyBackNFTId = fromBase16("bf24ed4af7eb5a7839c43aa6b240697d81b196120c837e1a941832c266d3755c")
        val buyBackInput = INPUTS(2)
        val buyBackOutput = OUTPUTS(2)

        buyBackInput.tokens(0)._1 == buyBackNFTId &&
            buyBackOutput.tokens(0)._1 == buyBackNFTId &&
            (buyBackOutput.value - buyBackInput.value) >= toOracle
      } else {
        true
      }
      val redeemCorrect = (redeemed <= maxToRedeem) && buyBackCorrect

      val position = getVar[Long](3).get
      val positionBytes = longToByteArray(position)

      val proof = getVar[Coll[Byte]](1).get
      val key = positionBytes ++ reserveId
      val value = history.get(key, proof).get

      val aBytes = value.slice(0, 33)
      val zBytes = value.slice(33, value.size)
      val a = decodePoint(aBytes)
      val z = byteArrayToBigInt(zBytes)

      val maxValueBytes = getVar[Coll[Byte]](2).get

      val message = positionBytes ++ maxValueBytes ++ noteTokenId
      val maxValue = byteArrayToLong(maxValueBytes)

      // Computing challenge
      val e: Coll[Byte] = blake2b256(aBytes ++ message ++ ownerKey.getEncoded) // strong Fiat-Shamir
      val eInt = byteArrayToBigInt(e) // challenge as big integer

      // Signature is valid if g^z = a * x^e
      val properSignature = (g.exp(z) == a.multiply(ownerKey.exp(eInt))) &&
                             noteValue <= maxValue

      // we check that receipt is properly formed, but we do not check receipt's contract here,
      // to avoid circular dependency as receipt contract depends on (hash of) our contract,
      // thus we are checking receipt contract in note and receipt contracts
      val receiptOutIndex = if (redeemed == 0) {
         getVar[Int](5).get
      } else {
         1
      }
      val receiptOut = OUTPUTS(receiptOutIndex)
      val properReceipt =
        receiptOut.tokens(0) == noteInput.tokens(0) &&
        receiptOut.R4[AvlTree].get == history  &&
        receiptOut.R5[Long].get == position    &&
        receiptOut.R6[Int].get >= HEIGHT - 20  &&  // 20 blocks for inclusion
        receiptOut.R6[Int].get <= HEIGHT &&
        receiptOut.R7[GroupElement].get == ownerKey

      // todo: could this be checked all the time ? if so then receiptMode can be eliminated
      val positionCorrect = if (receiptMode) {
        position < noteInput.R5[Long].get
      } else {
        true
      }

      sigmaProp(selfPreserved && properOracle && redeemCorrect && properSignature && properReceipt && positionCorrect)
    } else if (action == 1) {
      // top up
      // todo: check R5 preservation
      sigmaProp(selfPreserved && (selfOut.value - SELF.value >= 1000000000)) // at least 1 ERG added
    } else if (action == 2) {
      // issue a note
      // todo: check R5 preservation
      sigmaProp(selfPreserved)
    } else {
      sigmaProp(false)
    }

}