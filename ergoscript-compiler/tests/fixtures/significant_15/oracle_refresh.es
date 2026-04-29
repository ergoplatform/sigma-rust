{ // This box (refresh box)
  //   
  //   tokens(0) refresh token (NFT)
  
  // Placeholder TODOs in the EIP-0023 PR were fromBase64 — substituted to fromBase16 since
  // our local compiler doesn't support fromBase64 yet. Hex values are the b64-decoded bytes.
  val oracleTokenId = fromBase16("2a472d4a614e645267556b58703273357638792f423f4528482b4d6250655368")
  val poolNFT       = fromBase16("472b4b6250655368566d597133743677397a24432646294a404d635166546a57")
  val epochLength = 30 // TODO replace with actual
  val minDataPoints = 4 // TODO replace with actual
  val buffer = 4 // TODO replace with actual
  val maxDeviationPercent = 5 // percent // TODO replace with actual

  val minStartHeight = HEIGHT - epochLength
  val spenderIndex = getVar[Int](0).get // the index of the data-point box (NOT input!) belonging to spender    
    
  val poolIn = INPUTS(0)
  val poolOut = OUTPUTS(0)
  val selfOut = OUTPUTS(1)

  def isValidDataPoint(b: Box) = if (b.R6[Long].isDefined) {
    b.creationInfo._1    >= minStartHeight &&  // data point must not be too old
    b.tokens(0)._1       == oracleTokenId  && // first token id must be of oracle token
    b.R5[Int].get        == poolIn.R5[Int].get // it must correspond to this epoch
  } else false 
          
  val dataPoints = INPUTS.filter(isValidDataPoint)    
  val pubKey = dataPoints(spenderIndex).R4[GroupElement].get

  val enoughDataPoints = dataPoints.size >= minDataPoints    
  val rewardEmitted = dataPoints.size * 2 // one extra token for each collected box as reward to collector   
  val epochOver = poolIn.creationInfo._1 < minStartHeight
       
  val startData = 1L // we don't allow 0 data points
  val startSum = 0L 
  // we expect data-points to be sorted in INCREASING order
  
  val lastSortedSum = dataPoints.fold((startData, (true, startSum)), {
        (t: (Long, (Boolean, Long)), b: Box) => {
           val currData = b.R6[Long].get
           val prevData = t._1
           val wasSorted = t._2._1
           val oldSum = t._2._2
           val newSum = oldSum + currData  // we don't have to worry about overflow, as it causes script to fail

           val isSorted = wasSorted && prevData <= currData

           (currData, (isSorted, newSum))
        }
    }
  )
 
  val lastData = lastSortedSum._1
  val isSorted = lastSortedSum._2._1
  val sum = lastSortedSum._2._2
  val average = sum / dataPoints.size 

  val maxDelta = lastData * maxDeviationPercent / 100          
  val firstData = dataPoints(0).R6[Long].get

  proveDlog(pubKey)                                               &&
  epochOver                                                       && 
  enoughDataPoints                                                &&    
  isSorted                                                        &&
  lastData - firstData     <= maxDelta                            && 
  poolIn.tokens(0)._1      == poolNFT                             &&
  poolOut.tokens(0)        == poolIn.tokens(0)                    && // preserve pool NFT
  poolOut.tokens(1)._1     == poolIn.tokens(1)._1                 && // reward token id preserved
  poolOut.tokens(1)._2     >= poolIn.tokens(1)._2 - rewardEmitted && // reward token amount correctly reduced
  poolOut.tokens.size      == poolIn.tokens.size                  && // cannot inject more tokens to pool box
  poolOut.R4[Long].get     == average                             && // rate
  poolOut.R5[Int].get      == poolIn.R5[Int].get + 1              && // counter
  poolOut.propositionBytes == poolIn.propositionBytes             && // preserve pool script
  poolOut.value            >= poolIn.value                        &&
  poolOut.creationInfo._1  >= HEIGHT - buffer                     && // ensure that new box has correct start epoch height
  selfOut.tokens           == SELF.tokens                         && // refresh NFT preserved
  selfOut.propositionBytes == SELF.propositionBytes               && // script preserved
  selfOut.value            >= SELF.value                       
}
