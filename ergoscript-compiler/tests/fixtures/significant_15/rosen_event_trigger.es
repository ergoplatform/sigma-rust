{
  // ----------------- REGISTERS
  // R4: Coll[Byte] = WID list digest
  // R5: Coll[Coll[Byte]] = Event data
  // R6: Coll[Byte] = Permit contract script digest
  // R7: Int = Commitment Count
  // ----------------- TOKENS
  // 0: RWT

  // In case of fraud: [TriggerEvent, CleanupToken] => [Fraud1, Fraud2, ...]
  // In case of payment: [TriggerEvent, Commitments(if exists), LockBox](dataInput: GuardNFTBox) => [Permit1, ..., changeBox]
  // Placeholder constants substituted for fixture-only canonical compile.
  // Originals are template variables `CLEANUP_NFT`, `LOCK_SCRIPT_HASH`,
  // `FRAUD_SCRIPT_HASH`, `CLEANUP_CONFIRMATION` injected at deploy time.
  val cleanupNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001");
  val cleanupConfirmation = 720;
  val LockScriptHash = fromBase16("0000000000000000000000000000000000000000000000000000000000000002");
  val FraudScriptHash = fromBase16("0000000000000000000000000000000000000000000000000000000000000003");
  val GuardLockExists = INPUTS.exists { (box: Box) => blake2b256(box.propositionBytes) == LockScriptHash}
  val fraudScriptCheck = if(blake2b256(OUTPUTS(0).propositionBytes) == FraudScriptHash) {
    allOf(
      Coll(
        INPUTS(1).tokens(0)._1 == cleanupNFT,
        HEIGHT - cleanupConfirmation >= SELF.creationInfo._1
      )
    )
  } else {
    allOf(
      Coll(
        GuardLockExists,
        blake2b256(OUTPUTS(0).propositionBytes) == SELF.R6[Coll[Byte]].get
      )
    )
  }
  val commitmentCount = SELF.R7[Int].get
  val rewards = OUTPUTS.filter{(box:Box) 
      => box.tokens.size > 0 && box.tokens(0)._1 == SELF.tokens(0)._1
    }
    .slice(0, commitmentCount)
  val WIDs = rewards.map{(box:Box) => box.R4[Coll[Byte]].get}
  val widListDigest = blake2b256(WIDs.fold(Coll[Byte](), {(a: Coll[Byte], b: Coll[Byte]) => a++b}))
  val checkAllWIDs = rewards.forall {
    (data: Box) => {
      data.propositionBytes == OUTPUTS(0).propositionBytes &&
      data.tokens(0)._1 == SELF.tokens(0)._1 &&
      data.tokens(0)._2 == SELF.tokens(0)._2 / commitmentCount
    }
  }
  sigmaProp(
    allOf(
      Coll(
        rewards.size == commitmentCount,
        SELF.R4[Coll[Byte]].get == widListDigest,
        checkAllWIDs,
        fraudScriptCheck
      )
    )
  )
}
