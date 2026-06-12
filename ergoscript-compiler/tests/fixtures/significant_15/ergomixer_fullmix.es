{
  val g = groupGenerator
  val c1 = SELF.R4[GroupElement].get
  val c2 = SELF.R5[GroupElement].get
  val gX = SELF.R6[GroupElement].get
  val delta = SELF.R7[Coll[Byte]].get

  val isHalf = {(b: Box) => blake2b256(b.propositionBytes) == delta}
  val noFeeTokenInBox = {(b:Box) => b.tokens.forall({(a: (Coll[Byte], Long)) => a._1 != tokenId})}
  val destroyToken = OUTPUTS.forall(noFeeTokenInBox)
  val nextBob = isHalf(INPUTS(0)) && blake2b256(INPUTS(2).propositionBytes) == feeEmissionScriptHash
  val nextAlice = isHalf(OUTPUTS(0)) && blake2b256(INPUTS(1).propositionBytes) == feeEmissionScriptHash

  (proveDlog(c2) || proveDHTuple(g, c1, gX, c2)) && {
    sigmaProp(nextAlice || nextBob || destroyToken)
  }
}
