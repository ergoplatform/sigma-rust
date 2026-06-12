{   
    //////////////////////////////////////////////////////////
    // SigmaO OPTION CONTRACT
    //////////////////////////////////////////////////////////
    // Smart contract allowing to create tokens that behave like an option on other Ergo tokens
    // The option token owner is granted to buy or sell (exercise the option) one or several tokens (share size) at a given strike price
    // The option can be of type:
    //      - Call: The option issuer sells tokens against ERG
    //              The option token owner can exercise the option to buy tokens at the defined strike price
    //      - Put: The option issuer buys tokens against ERG
    //             The option token owner can exercise the option to sell tokens at the defined strike price
    // The option can be of style:
    //      - European options can be exercised 24h after the maturity date
    //      - American options can be exercised up to maturity date
    // The option minimal duration is set to 24h
    // States:
    //   - not minted: Refundable SigmaO option token mint request (Option definition box)
    //                   Erg Value : 3 * TxFee + 2 * BoxMinValue + dAppUIMintFee + (PUT ? N nanoERG)
    //                   Tokens: (CALL ? N underlying token raw amount)
    //                   R4 Coll[Byte]: Option name for information purpose
    //                   R5 Coll[Byte]: Underlying token ID
    //                   R6 Coll[Byte]: Underlying token decimals encoded as a string, for better display of the option amount in the wallet, not used
    //                   R7 Box: Random small box to avoid crash of the contract
    //                   R8 Coll[Long]: [
    //                                    - Option Type (0 Call, 1 Put)
    //                                    - Option Style (0 European, 1 American)
    //                                    - Share size (number of smaller unit of token per smaller unit of option)
    //                                    - Maturity Date (unix time milleseconds)
    //                                    - Strike price (nanoerg per smallest unit of token)
    //                                    - dApp Mint Fee (nanoerg) (mint fee freely computed by the UI)
    //                                    - miner transaction fee (nanoerg)
    //                                  ]
    //                   R9 Coll[Coll[Byte]]: [
    //                                    - Issuer EC point
    //                                    - dAppUIErgoTree
    //                                  ]
    //    
    //   - minted, not delivered: Option tokens are created and needs to be delivered to the issuer
    //                   Erg Value : 2 * TxFee + 2 * BoxMinValue (PUT ? N nanoERG)
    //                   Tokens: N / Share Size + 1 Option tokens
    //                           (CALL ? N underlying token raw amount)
    //                   R4 Coll[Byte]: Option name for information purpose
    //                   R5 Coll[Byte]: Underlying token ID
    //                   R6 Coll[Byte]: Underlying token decimals encoded as a string, for better display of the option amount in the wallet, not used
    //                   R7 Box: Option Creation box (not mined state)
    //
    //   - reserve: Option reserve is now usable, the option are delivered to the issuer, one stay in the box
    //                   Erg Value : 1 * TxFee + 1 * BoxMinValue (PUT ? N nanoERG)
    //                   Tokens: 1 Option tokens
    //                           (CALL ? N underlying token raw amount)
    //                   R4 Coll[Byte]: Option name for information purpose
    //                   R5 Coll[Byte]: Underlying token ID
    //                   R6 Coll[Byte]: Underlying token decimals encoded as a string, for better display of the option amount in the wallet, not used
    //                   R7 Box: Option Creation box (not mined state)
    //
    
    // Option underlying token / ERG
    val HourInMilli = 3600000L
    val BoxMinValue = 1000000L

    val valueIn: Long = SELF.value
    val selfToken0: (Coll[Byte], Long) = SELF.tokens.getOrElse(0, (Coll[Byte](),0L))
    val selfToken1: (Coll[Byte], Long) = SELF.tokens.getOrElse(1, (Coll[Byte](),0L))
    val output0Token0: (Coll[Byte], Long) = OUTPUTS(0).tokens.getOrElse(0, (Coll[Byte](),0L))
    val output0Token1: (Coll[Byte], Long) = OUTPUTS(0).tokens.getOrElse(1, (Coll[Byte](),0L))
    val output1Token0: (Coll[Byte], Long) = OUTPUTS(1).tokens.getOrElse(0, (Coll[Byte](),0L))
    
    val isMinted: Boolean = selfToken0._1 == SELF.R7[Box].get.id                           && 
                            SELF.propositionBytes == SELF.R7[Box].get.propositionBytes
    // if the token is not minted yet take the configuration from self, else from the mint box in the register R7
    val optionCreationBox: Box = if (isMinted) {
        SELF.R7[Box].get
    } else {
        SELF
    }

    // Get option creation box info
    val optionName: Coll[Byte] = optionCreationBox.R4[Coll[Byte]].get
    val underlyingAssetTokenId: Coll[Byte] = optionCreationBox.R5[Coll[Byte]].get
    val optionDecimals: Coll[Byte] = optionCreationBox.R6[Coll[Byte]].get
    val isCall: Boolean = optionCreationBox.R8[Coll[Long]].get(0) == 0L // 0 Call, 1 Put
    val isEuropean: Boolean = optionCreationBox.R8[Coll[Long]].get(1) == 0L // 0 european, 1 american
    val shareSize: Long = optionCreationBox.R8[Coll[Long]].get(2) // Number of smallest unit of underlying token granted per option, 100 for 1 SigUSD per option
    val maturityDate: Long = optionCreationBox.R8[Coll[Long]].get(3) // Unix time
    val strikePrice: Long = optionCreationBox.R8[Coll[Long]].get(4) // nanoerg per token smallest unit of underlying token
    val dAppUIMintFee: Long = optionCreationBox.R8[Coll[Long]].get(5) // nanoerg
    val TxFee: Long = optionCreationBox.R8[Coll[Long]].get(6) // nanoerg
    val issuerECPoint: Coll[Byte] = optionCreationBox.R9[Coll[Coll[Byte]]].get(0)
    val issuerErgoTree: Coll[Byte] = proveDlog(decodePoint(issuerECPoint)).propBytes
    val dAppUIFeeErgoTree: Coll[Byte] = optionCreationBox.R9[Coll[Coll[Byte]]].get(1)
    val optionTokenIDIn: Coll[Byte] = optionCreationBox.id

    val MinOptionReserveValue: Long = TxFee + BoxMinValue

    // Compute option state
    val currentTimestamp: Long = CONTEXT.preHeader.timestamp
    val remainingDuration: Long = maturityDate - currentTimestamp
    val isOptionDelivered: Boolean = isMinted            &&
                                     selfToken0._2 == 1L
    val isExpired: Boolean = currentTimestamp > maturityDate
    val isExercible: Boolean = if (isEuropean) { // European
        // exercible during 24h after expiration
        isOptionDelivered && isExpired && currentTimestamp < maturityDate + 24 * HourInMilli
    } else { // American
        // exercible until expiration
        isOptionDelivered && !isExpired
    }
    val isEmpty: Boolean = if (isCall) {
        isOptionDelivered && selfToken1._2 == 0L
    } else {
        isOptionDelivered && valueIn == MinOptionReserveValue
    }                           


    val validBasicReplicatedOutput0: Boolean = if (OUTPUTS(0).propositionBytes == SELF.propositionBytes) {
        OUTPUTS(0).value >= MinOptionReserveValue                  &&
        OUTPUTS(0).R4[Coll[Byte]].get == optionName                && // For information purpose
        OUTPUTS(0).R5[Coll[Byte]].get == underlyingAssetTokenId    && // underlyingAssetTokenId to find the reserve boxes per underlying asset
        OUTPUTS(0).R6[Coll[Byte]].get == optionDecimals            && // Same decimal than for the underlyingAssetTokenId for better display in wallets, not used
        OUTPUTS(0).R7[Box].get == optionCreationBox
    } else {
        false
    }

    val validMintOption: Boolean = if (!isMinted && INPUTS.size == 1 && OUTPUTS.size == 3) {
        validBasicReplicatedOutput0                           &&
        OUTPUTS(0).value == valueIn - TxFee - dAppUIMintFee   &&
        OUTPUTS(0).value >=  2 * MinOptionReserveValue        && // prevent to get stuck before delivery
        output0Token0._1 == SELF.id                           &&
        (
            (  // Call
                isCall                                                                   &&
                output0Token1._1 == underlyingAssetTokenId                               &&
                output0Token1 == selfToken0                                              && // keep all underlying tokens
                output0Token0._2 == selfToken0._2 / shareSize + 1L                       && // minted option, one to stay in the box
                OUTPUTS(0).tokens.size == 2
            ) ||
            (   // Put
                !isCall                                                                  &&
                OUTPUTS(0).tokens.size == 1                                              &&
                output0Token0._2 == (valueIn - 3 * TxFee - dAppUIMintFee - 2 * BoxMinValue) / (strikePrice * shareSize) + 1L // minted option, one to stay in the box
            )
        )                                                     &&
        // dApp Fee
        OUTPUTS(1).propositionBytes == dAppUIFeeErgoTree      &&
        OUTPUTS(1).tokens.size == 0                           &&
        OUTPUTS(1).value >= dAppUIMintFee
    } else {
        false
    }

    val validDeliverOption: Boolean = if (isMinted && !isOptionDelivered && INPUTS.size == 1 && OUTPUTS.size == 3) {
        // replicate option reserve
        OUTPUTS(0).value == valueIn - TxFee - BoxMinValue     &&
        validBasicReplicatedOutput0                           &&
        output0Token0._1 == selfToken0._1                     &&
        output0Token0._2 == 1L                                &&
        output0Token1 == selfToken1                           &&
        // deliver options to the issuer
        OUTPUTS(1).propositionBytes == issuerErgoTree         &&
        OUTPUTS(1).value == BoxMinValue                       &&
        OUTPUTS(1).tokens.size == 1                           &&
        output1Token0._1 == selfToken0._1                     &&
        output1Token0._2 == selfToken0._2 - 1L
    } else {
        false
    }

    val validCloseOptionContract: Boolean = if ((isExpired && !isExercible) || isEmpty) {
        OUTPUTS.size == 2                                          &&
        OUTPUTS(0).propositionBytes == issuerErgoTree              &&
        OUTPUTS(0).value >= valueIn - TxFee                        &&
        output0Token0._1 == selfToken1._1                          && //return underlying tokens if any
        output0Token0._2 == selfToken1._2
    } else {
        false
    }

    val validExerciseOption: Boolean = if (isExercible && INPUTS.size == 2 && OUTPUTS.size == 4) {
        val output2Token0: (Coll[Byte], Long) = OUTPUTS(2).tokens.getOrElse(0, (Coll[Byte](),0L))
        val input1Token0: (Coll[Byte], Long) = INPUTS(1).tokens.getOrElse(0, (Coll[Byte](),0L))
        val exercisedAmountReserve: Long = if (isCall) {
            selfToken1._2 - output0Token1._2
        } else {
            valueIn - OUTPUTS(0).value
        }
        val numberOptionExpected: Long = if (isCall) {
            exercisedAmountReserve / shareSize
        } else {
            exercisedAmountReserve / ( strikePrice * shareSize)
        }
        val numberOptionProvided = if (input1Token0._1 == optionTokenIDIn) {
            input1Token0._2
        } else {
            0L
        }

        // replicate the option reserve
        numberOptionExpected == numberOptionProvided                                  &&
        validBasicReplicatedOutput0                                                   &&
        // buyer delivery, PK verified by the buy request script            
        (
            (  // call exercised, option issuer sell the underlying token
               // option user buy the underlying token against ERG
                isCall                                                                &&
                selfToken0 == output0Token0                                           &&
                (
                    output0Token1._1 == underlyingAssetTokenId         ||
                    output0Token1._2 == 0L // empty the reserve
                )                                                                     &&
                output1Token0._1 == underlyingAssetTokenId                            &&
                output1Token0._2 == exercisedAmountReserve                            &&
                OUTPUTS(1).tokens.size == 1                                           &&
                OUTPUTS(2).value >= numberOptionExpected * strikePrice * shareSize    && // strike price in nanoerg no need to adjust
                OUTPUTS(2).tokens.size == 0
            )
            ||
            (  // put exercised, option issuer buy the underlying token
               // option user sell the underlying token against ERG
                !isCall                                                               &&
                OUTPUTS(1).value >= exercisedAmountReserve                            &&
                OUTPUTS(1).tokens.size == 0                                           &&
                output2Token0._1 == underlyingAssetTokenId                            &&
                output2Token0._2 >= numberOptionExpected * shareSize                  &&
                OUTPUTS(2).tokens.size == 1
            )
        )                                                                             &&
        // issuer pay box            
        OUTPUTS(2).propositionBytes == issuerErgoTree                                 &&
        // ensure option are burnt
        OUTPUTS(3).tokens.size == 0
    } else {
        false
    }

    // RESULT
    (
        ( // refund the issuer
            proveDlog(decodePoint(issuerECPoint))                          && 
            sigmaProp(!isMinted                                            && 
                      OUTPUTS.size == 2                                    &&  // prevent random option minting from the issuer
                      OUTPUTS(0).propositionBytes == issuerErgoTree
                     )
        )            
                                        || 
        sigmaProp(                         // action by anyone 
            validMintOption             || 
            validExerciseOption         ||
            validDeliverOption          ||
            validCloseOptionContract
            )
    )
}   
