{
  val charityCoef = SELF.R4[Coll[Long]].get(0)
  val serviceFee = SELF.R4[Coll[Long]].get(1)
  val ticketPrice = SELF.R4[Coll[Long]].get(2)
  val goal = SELF.R4[Coll[Long]].get(3)
  val deadlineHeight = SELF.R4[Coll[Long]].get(4)
  val totalSoldTicket = SELF.R4[Coll[Long]].get(5)
  val totalSoldTicketBI: BigInt = totalSoldTicket.toBigInt
  val winnerCoef = 100L - charityCoef - serviceFee
  val charityAddress = SELF.R5[Coll[Byte]].get
  val serviceAddress = SELF.R7[Coll[Byte]].get
  val totalRaised = totalSoldTicket * ticketPrice
  val outCharityCoef = OUTPUTS(0).R4[Coll[Long]].get(0)
  val outServiceFee = OUTPUTS(0).R4[Coll[Long]].get(1)
  val outTicketPrice = OUTPUTS(0).R4[Coll[Long]].get(2)
  val outGoal = OUTPUTS(0).R4[Coll[Long]].get(3)
  val outDeadlineHeight = OUTPUTS(0).R4[Coll[Long]].get(4)
  val outTotalSoldTicket = OUTPUTS(0).R4[Coll[Long]].get(5)
  if (HEIGHT < deadlineHeight) {
    // user can donate
    val currentSoldTicket = OUTPUTS(1).tokens(0)._2
    sigmaProp(
      allOf(
        Coll(
          // validate app.raffle box
          OUTPUTS(0).tokens(0)._1 == SELF.tokens(0)._1,
          OUTPUTS(0).propositionBytes == SELF.propositionBytes,
          OUTPUTS(0).R5[Coll[Byte]].get == charityAddress,
          OUTPUTS(0).R6[Coll[Coll[Byte]]].get == SELF.R6[Coll[Coll[Byte]]].get,
          OUTPUTS(0).R7[Coll[Byte]].get == serviceAddress,
          outCharityCoef == charityCoef,
          outServiceFee == serviceFee,
          outTicketPrice == ticketPrice,
          outGoal == goal,
          outDeadlineHeight == deadlineHeight,
          outTotalSoldTicket == totalSoldTicket + currentSoldTicket,
          // check ticket script
          blake2b256(OUTPUTS(1).propositionBytes) == ticketScriptHash,
          OUTPUTS(1).tokens(0)._1 == SELF.tokens(1)._1,
          // protect token from burning
          SELF.tokens(1)._2 == OUTPUTS(0).tokens(1)._2 + currentSoldTicket,
          // check ergs
          OUTPUTS(1).value >= fee,
          OUTPUTS(0).value == SELF.value + (currentSoldTicket * ticketPrice),
          // Winner Address or redeem
          // TODO check R4 to be valid address
          OUTPUTS(1).R4[Coll[Byte]].isDefined,
          // check ticket parameters [rangeStart, rangeEnd, deadlineHeight, ticketPrice]
          OUTPUTS(1).R5[Coll[Long]].get(0) == totalSoldTicket,
          OUTPUTS(1).R5[Coll[Long]].get(1) == outTotalSoldTicket,
          OUTPUTS(1).R5[Coll[Long]].get(2) == deadlineHeight,
          OUTPUTS(1).R5[Coll[Long]].get(3) == ticketPrice
        )
      )
    )
  } else {
      if(totalRaised >= goal) {
        // charge charity address and service fee. then change status to completed
        val charityAmount = totalRaised * charityCoef / 100L
        val serviceFeeAmount = totalRaised * serviceFee / 100L
        val winnerAmount = totalRaised - charityAmount - serviceFeeAmount
        val winNumber = (((byteArrayToBigInt(CONTEXT.dataInputs(0).id.slice(0, 15)).toBigInt % totalSoldTicketBI) + totalSoldTicketBI) % totalSoldTicketBI).toBigInt
        sigmaProp(
          allOf(
            Coll(
              // check winner box remain on output box
              blake2b256(OUTPUTS(0).propositionBytes) == winnerScriptHash,
              OUTPUTS(0).R4[Coll[Long]].get == SELF.R4[Coll[Long]].get,
              OUTPUTS(0).R5[Coll[Byte]].get == charityAddress,
              OUTPUTS(0).R6[Coll[Coll[Byte]]].get == SELF.R6[Coll[Coll[Byte]]].get,
              OUTPUTS(0).R7[Coll[Byte]].get == serviceAddress,

              OUTPUTS(0).R8[Long].get == winNumber,
              OUTPUTS(0).tokens(0)._1 == SELF.tokens(0)._1,
              OUTPUTS(0).tokens(1)._1 == SELF.tokens(1)._1,
              OUTPUTS(0).tokens(1)._2 == SELF.tokens(1)._2,
              OUTPUTS(0).value >= winnerAmount,
              // check charity to passed to charity address
              OUTPUTS(1).propositionBytes == charityAddress,
              OUTPUTS(1).value >= charityAmount,
              // check service fee
              OUTPUTS(2).propositionBytes == serviceAddress,
              OUTPUTS(2).value >= serviceFeeAmount,
              // check datainput to be oracle box
              CONTEXT.dataInputs(0).tokens(0)._1 == randomBoxToken,
              // and datainput must created after deadline
              CONTEXT.dataInputs(0).creationInfo._1 > deadlineHeight
            )
          )
        )
      } else {
      // begin refund
        sigmaProp(
          allOf(
            Coll(
              // check winner box remain on output box
              blake2b256(OUTPUTS(0).propositionBytes) == redeemScriptHash,
              OUTPUTS(0).R4[Coll[Long]].get == SELF.R4[Coll[Long]].get,
              // box must move to refund state
              OUTPUTS(0).R5[Coll[Byte]].get == charityAddress,
              OUTPUTS(0).R6[Coll[Coll[Byte]]].get == SELF.R6[Coll[Coll[Byte]]].get,
              OUTPUTS(0).tokens(0)._1 == SELF.tokens(0)._1,
              OUTPUTS(0).tokens(1)._1 == SELF.tokens(1)._1,
              OUTPUTS(0).tokens(1)._2 == SELF.tokens(1)._2,
              OUTPUTS(0).value == SELF.value - fee
            )
          )
        )
      }
    }
}
