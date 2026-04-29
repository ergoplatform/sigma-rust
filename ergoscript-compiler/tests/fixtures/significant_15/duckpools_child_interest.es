{
    val PoolNft                = fromBase58("BJt8g3gZqEhCL9rzfDsfp7Me67DTuM8HvYBW5yHoHxs")
    val InterestParamaterBoxNft = fromBase58("E9emgoNfXgLHWXs612Cv6bf1aE1PC27Fa4aKT8SEcnQy")
    val InterestDenomination     = 100000000L
    val CoefficientDenomination = 100000000L
    val InitiallyLockedLP      = 9000000000000000L
    val MaximumBorrowTokens        = 9000000000000000L
    val MaximumHistoryHeight = 430
    val MaximumExecutionFee = 2000000
    val updateFrequency = 120

    val successor = OUTPUTS(0)
    val pool      = CONTEXT.dataInputs(0)
    val parameterBox = CONTEXT.dataInputs(1)

    val interestHistory      = SELF.R4[Coll[Long]].get
    val recordedHeight       = SELF.R5[Long].get
    val childIndex = SELF.R6[Int].get
    val finalInterestHistory = successor.R4[Coll[Long]].get
    val finalHeight          = successor.R5[Long].get
    val finalChildIndex = successor.R6[Int].get

    // get coefficients
    val coefficients = parameterBox.R4[Coll[Long]].get
    val a = coefficients(0).toBigInt
    val b = coefficients(1).toBigInt
    val c = coefficients(2).toBigInt
    val d = coefficients(3).toBigInt
    val e = coefficients(4).toBigInt
    val f = coefficients(5).toBigInt
     
    val deltaHeight      = HEIGHT - recordedHeight
    val isReadyToUpdate = deltaHeight >= updateFrequency 

    val poolAssets = pool.tokens(3)._2
    val deltaFinalHeight      = finalHeight - HEIGHT
    val validDeltaFinalHeight = (deltaFinalHeight >= 0 && deltaFinalHeight <= 5)
    val borrowed    = MaximumBorrowTokens - pool.tokens(2)._2
    val util        = (InterestDenomination.toBigInt * borrowed.toBigInt / (poolAssets.toBigInt + borrowed.toBigInt))

    val D = CoefficientDenomination.toBigInt
    val M = InterestDenomination.toBigInt
    val x = util.toBigInt

    val currentRate = (
        M + (
            a + 
            (b * x) / D + 
            (c * x) / D * x / M +
            (d * x) / D * x / M * x / M +
            (e * x) / D * x / M * x / M * x / M + 
            (f * x) / D * x / M * x / M * x / M * x / M
            )
        ) 

    val retainedERG          = successor.value >= SELF.value - MaximumExecutionFee
    val preservedInterestNFT = successor.tokens == SELF.tokens

    val validSuccessorScript = SELF.propositionBytes == successor.propositionBytes
    val validInterestUpdate = ( 
        interestHistory == finalInterestHistory.slice(0, interestHistory.size) &&
        finalInterestHistory(interestHistory.size).toBigInt == currentRate &&
        finalInterestHistory.size == interestHistory.size + 1
    )

    val validPoolBox = pool.tokens(0)._1 == PoolNft
    val validParameterBox = parameterBox.tokens(0)._1 == InterestParamaterBoxNft

    val retainIndex = finalChildIndex == childIndex
    val isUnderMaxHeight = finalInterestHistory.size <= MaximumHistoryHeight
    
    val isValidDummyRegisters = (
        successor.R7[Boolean].get &&
        successor.R8[Boolean].get &&
        successor.R9[Boolean].get
    )

    val noMoreTokens = successor.tokens.size == SELF.tokens.size

    sigmaProp(
        isReadyToUpdate &&
        isUnderMaxHeight &&
        validSuccessorScript &&
        retainedERG &&
        preservedInterestNFT &&
        validInterestUpdate &&
        validDeltaFinalHeight &&
        retainIndex &&
        validPoolBox &&
        validParameterBox &&
        isValidDummyRegisters &&
        noMoreTokens          
    )
}
