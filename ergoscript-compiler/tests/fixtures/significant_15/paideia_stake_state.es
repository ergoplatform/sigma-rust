{

    // ===== Contract Information ===== //
    // Name: stakeState
    // Description: Contract that governs the stake (initial stake and add stake), emit, and unstake (full unstake and partial unstake) txs for the stake state box.
    // Version: 1.0
    // Authors: Lui, Luca

    // ===== Stake State Box (SELF) ===== //
    // Registers:
    //   R4[Coll[Long]]:
    //     0: Total Amount Staked
    //     1: Checkpoint
    //     2: Stakers
    //     3: Last Checkpoint Timestamp
    //     4: Cycle Duration 
    // Tokens:
    //   0: 
    //     _1: Stake State NFT  // Identifier for the stake state box.
    //     _2: Amount: 1  
    //   1: 
    //     _1: Stake Token  // Token proving that the stake box was created properly.
    //     _2: Amount: <= 1 Billion

    // ===== Stake Pool Box ===== //
    // Registers:
    //   R4[Long]: Emission Amount
    //   R5[Coll[Byte]]: Stake Pool Key
    // Tokens:
    //   0:
    //     _1: Stake Pool NFT  // Identifier for the stake pool box.
    //     _2: Amount: 1
    //   1:
    //     _1: DAO Token ID  // Token issued by the DAO for distribution
    //     _2: Amount: <= Total DAO Tokens Amount

    // ===== Emission Box ===== //
    // Registers:
    //   R4[Coll[Long]]:
    //     0: Total Amount Staked
    //     1: Checkpoint
    //     2: Stakers
    //     3: Emission Amount
    // Tokens:
    //   0: 
    //     _1: Emission NFT  // Identifier for the emission box.
    //     _2: Amount: 1
    //   1: 
    //     _1: DAO Token ID  // Tokens to be emitted by the DAO.
    //     _2: Amount: <= DAO Token Emission Amount

    // ===== Stake Box ===== //
    // Registers:
    //   R4[Coll[Long]]:
    //     0: Checkpoint
    //     1: Staking Time
    //   R5[Coll[Byte]]: Stake Key ID // ID of the stake key used for unstaking.
    // Tokens:
    //   0:
    //     _1: Stake Token  // Token proving that the stake box was created properly.
    //     _2: Amount: 1
    //   1:
    //     _1: DAO Token  // Token issued by the DAO, which the user wishes to stake.
    //     _2: Amount: > 0

    // ===== Staking Incentive Box ===== //
    // Value: ERG to pay the tx execution bot and the tx mining fee.
    // Registers: None
    // Tokens: None

    // ===== Stake Proxy Box ===== //
    // Registers:
    //   R4[Coll[Long]]:
    //     0: Staketime
    //   R5[Coll[Byte]]: User ErgoTree bytes
    // Tokens:
    //   0:
    //     _1: DAO Token  // Token issued by the DAO, which the user wishes to stake.
    //     _2: Amount: > 0 amount that the user owns and wants to stake, and sent to the proxy box.

    // ===== Unstake Proxy Box ===== //
    // Registers:
    //  R4[Coll[Long]]:
    //    0: Amount to unstake
    //  R5[Coll[Byte]]: User ErgoTree bytes (i.e. PK)
    // Tokens:
    //  0:
    //    _1: Stake Key  // Sent from the user to the unstake proxy box
    //    _2: Amount: 1

    // ===== Add Stake Proxy Box ===== //
    // Registers:
    //   R4[Coll[Long]]:
    //     0: Staketime
    //   R5[Coll[Byte]]: User ErgoTree bytes
    // Tokens:
    //  0:
    //    _1: Stake Key  // NFT used as a key for adding stake as well as unstaking
    //    _2: Amount: 1
    //  1:
    //    _1: DAO Token  // Token issued by the DAO, which the user wishes to stake.
    //    _2: Amount: > 0 amount that the uers owns and wants to additionally stake.

    // ===== Stake Tx ===== //
    // Description: User sends DAO tokens to the stake box and receive a stake key in return, proving they have staked and used for unstaking.
    // Inputs: 
    //   Stake: StakeStateBox, StakeProxyBox
    //   Add Stake: StakeStateBox, StakeBox, AddStakeProxyBox
    // DataInputs: None
    // Context Extension Variables: None
    // Outputs: 
    //   Stake: NewStakeStateBox, NewStakeBox, UserWalletBox
    //   Add Stake: NewStakeStateBox, NewStakeBox, UserWalletBox 

    // ===== Emit Tx ===== //
    // Description: Ran once per day, determining the amount from the stake pool to be withdrawn into a new emission box before being distributed to the stakers.
    // Inputs: StakeStateBox, StakePoolBox, EmissionBox, StakingIncentiveBox
    // DataInputs: None
    // Context Extension Variables: None
    // Outputs: NewStakeStateBox, NewStakePoolBox, NewEmissionBox, EmissionFeeBox, NewStakingIncentiveBox, TxOperatorOutputBox

    // ===== Unstake Tx ===== //
    // Description: User wishes to remove their staked tokens from the staking protocol, using their stake key.
    // Inputs: StakeStateBox, StakeBox, UnstakeProxyBox
    // DataInputs: None
    // Context Extension Variables: None
    // Outputs: 
    //   Full Unstake: NewStakeStateBox, UserWalletBox
    //   Partial Unstake: NewStakeStateBox, UserWalletBox, NewStakeBox

    // ===== Hard-Coded Constants ===== //
    val BlockTime: Long           = CONTEXT.preHeader.timestamp  // Timestamp from blockchain preheader
    val StakedTokenID: Coll[Byte] = _stakedTokenID               // Token identifier for the token distributed by the DAO
    val StakePoolNFT: Coll[Byte]  = _stakePoolNFT                // Identifier for the stake pool box
    val EmissionNFT: Coll[Byte]   = _emissionNFT                 // Identifier for the emission box
    val StakeContract: Coll[Byte] = _stakeContractHash           // Hash of the staking contract P2S address
    val MinimumStake: Long        = 1000L                        // Minimum amount of DAO tokens required for staking

    // Input stake state box parameters
    val totalAmountStaked: Long    = SELF.R4[Coll[Long]].get(0)
    val checkpoint: Long           = SELF.R4[Coll[Long]].get(1)
    val stakers: Long              = SELF.R4[Coll[Long]].get(2)
    val checkpointTimestamp: Long  = SELF.R4[Coll[Long]].get(3)
    val cycleDuration: Long        = SELF.R4[Coll[Long]].get(4)
    val stakeTokenID: Coll[Byte]   = SELF.tokens(1)._1

    // Output stake state box parameters
    val newStakeStateBox: Box         = OUTPUTS(0)
    val totalAmountStakedOut: Long    = newStakeStateBox.R4[Coll[Long]].get(0)
    val checkpointOut: Long           = newStakeStateBox.R4[Coll[Long]].get(1)
    val stakersOut: Long              = newStakeStateBox.R4[Coll[Long]].get(2)
    val checkpointTimestampOut: Long  = newStakeStateBox.R4[Coll[Long]].get(3)
    val cycleDurationOut: Long        = newStakeStateBox.R4[Coll[Long]].get(4)

    // Check for valid parameter replication for the new stake state box
    val validSelfReplication: Boolean = {
    
        allOf(Coll(
            
            // Check that the ERG value is preserved
            (newStakeStateBox.value == SELF.value),

            // Check that the stake state contract is the same
            (newStakeStateBox.propositionBytes == SELF.propositionBytes),    

            // Check that the stake state nft identifier token is the same 
            (newStakeStateBox.tokens(0)._1 == SELF.tokens(0)._1),
            (newStakeStateBox.tokens(0)._2 == SELF.tokens(0)._2),

            // Check that the stake token id is the same
            (newStakeStateBox.tokens(1)._1 == stakeTokenID),

            // The total tokens in the new stake state box is always 2
            (newStakeStateBox.tokens.size == 2),

            // The emission cycle duration is constant
            (cycleDurationOut == cycleDuration)

        ))

    }

    // ===== Perform Stake Tx ===== //

    // Check conditions for a valid Stake Tx
    val validStakeTx: Boolean = {

        // Check if this is a stake tx, the new stake box will have the stake token NFT ID
        val isStakeTx : Boolean = (OUTPUTS(1).tokens(0)._1 == stakeTokenID)

        if (isStakeTx) {

            // Check to see if this is the first staking tx for the user - the stake token amounts will be less for the new stake state box
            val isInitialStake: Boolean = (newStakeStateBox.tokens(1)._2 < SELF.tokens(1)._2)

            // ===== Perform Initial Stake Tx ===== //
            if (isInitialStake) {

                // Stake tx inputs
                val stakeStateBox: Box = INPUTS(0)
                val stakeProxyBox: Box = INPUTS(1)

                // Stake tx outputs
                val newStakeBox: Box = OUTPUTS(1)
                val userWalletBox: Box = OUTPUTS(2)

                // Conditions for a valid new stake state output box
                val validNewStakeStateBox: Boolean = {

                    allOf(Coll(
                            
                        // Check that the new stake state box has the same parameters
                        validSelfReplication,

                        // Check that the total staked amount increased
                        (totalAmountStakedOut == totalAmountStaked + newStakeBox.tokens(1)._2),
                        
                        // Check that the staking checkpoint has not changed
                        (checkpointOut == checkpoint),

                        // Check that the amount of stakers has increased
                        (stakersOut == stakers+1),

                        // Check that the checkpoint timestamp is the same 
                        (checkpointTimestampOut == checkpointTimestamp),

                        // Check that the amount of stake tokens has decreased
                        (newStakeStateBox.tokens(1)._2 == SELF.tokens(1)._2-1)
                            
                    ))

                }

                // Conditions for a valid new stake output box
                val validNewStakeBox: Boolean = {

                    allOf(Coll(
                        
                        // Check that the hash of the output stake box contract is the same as the hard-coded value
                        (blake2b256(newStakeBox.propositionBytes) == StakeContract),

                        // Check that the new stake box created has the current checkpoint
                        (newStakeBox.R4[Coll[Long]].get(0) == checkpoint),

                        // Give half an hour leeway for staking start
                        (newStakeBox.R4[Coll[Long]].get(1) >= BlockTime - 1800000L),

                        // Check that the stake key id (box id of the current stake state box) is stored in R5 of the new stake box
                        (newStakeBox.R5[Coll[Byte]].get == SELF.id),

                        // Check that the new stake box has 1 stake token given from the current stake state box
                        (newStakeBox.tokens(0)._1 == stakeTokenID),
                        (newStakeBox.tokens(0)._2 == 1L),

                        // Check that the new stake box has the minimum amount of DAO tokens staked in it
                        (newStakeBox.tokens(1)._1 == StakedTokenID),
                        (newStakeBox.tokens(1)._2 >= MinimumStake)

                    ))

                }

                // Conditions for a valid user wallet output box
                val validUserWalletBox: Boolean = {

                    allOf(Coll(

                        // Check that the address of the user wallet box corresponds to the stored value in R5 of the input stake proxy box
                        (userWalletBox.propositionBytes == stakeProxyBox.R5[Coll[Byte]].get),

                        // Check that the user receives the stake key token
                        (userWalletBox.tokens(0)._1 == newStakeBox.R5[Coll[Byte]].get),
                        (userWalletBox.tokens(0)._2 == 1L)

                    ))

                }

                // Conditions for the initial stake
                val validInitialStake: Boolean = {

                    allOf(Coll(
                        validNewStakeStateBox,
                        validNewStakeBox,
                        validUserWalletBox
                    ))

                }

                validInitialStake

            } else {
                    
                // ===== Perform Add Stake Tx ===== //

                // Add stake tx inputs
                val stakeStateBox: Box = INPUTS(0)
                val stakeBox: Box = INPUTS(1)
                val addStakeProxyBox: Box = INPUTS.getOrElse(2, INPUTS(0))

                // Add stake tx outputs
                val newStakeBox: Box = OUTPUTS(1)  
                val userWalletBox: Box = OUTPUTS(2)

                // Conditions for a valid stake state box
                val validNewStakeStateBox: Boolean = {

                    allOf(Coll(

                        // Check that the new stake state box has the same parameters
                        validSelfReplication,

                        // Check that the total amount staked has increased by the correct amount
                        (totalAmountStakedOut == totalAmountStaked + (newStakeBox.tokens(1)._2 - stakeBox.tokens.getOrElse(1,(Coll[Byte](),0L))._2)),
                        
                        // The checkpoint is the same value since staking occurs within the same emission cycle
                        (checkpointOut == checkpoint),
                        
                        // The amount of stakers has not increased
                        (stakersOut == stakers),

                        // The checkpoint timestamp remains the same
                        (checkpointTimestampOut == checkpointTimestamp),

                        // The amount of stake tokens does not change since the new stake box represents the stake belonging to the same user
                        (newStakeStateBox.tokens(1)._2 == SELF.tokens(1)._2)

                    ))

                }

                // Conditions for a valid stake box
                val validNewStakeBox: Boolean = {

                    allOf(Coll(

                        // Check that the new stake box value is preserved
                        (newStakeBox.value == stakeBox.value),
                        
                        // Check that the hash of the output stake box contract is the same as the hard-coded value
                        (blake2b256(newStakeBox.propositionBytes) == StakeContract),

                        // Check that the hash of the input stake box contract is the same as the hard-coded value
                        (blake2b256(stakeBox.propositionBytes) == StakeContract),

                        // Check that the checkpoint value of the output stake box is the same as the input stake state box
                        (newStakeBox.R4[Coll[Long]].get(0) == checkpoint),

                        // Check that the output and input stake boxes have the same parameters: Checkpoint, Staking Time, and Stake Key ID
                        (newStakeBox.R4[Coll[Long]].get == stakeBox.R4[Coll[Long]].get),
                        (newStakeBox.R5[Coll[Byte]].get == stakeBox.R5[Coll[Byte]].get),

                        // Check that the output stake box has the same id and amount of the stake token 
                        (newStakeBox.tokens(0)._1 == stakeTokenID),
                        (newStakeBox.tokens(0)._2 == 1L),

                        // Check that output stake box contains the same DAO token id and the added tokens to be staked from the input proxy contract
                        (newStakeBox.tokens(1)._1 == StakedTokenID),
                        (newStakeBox.tokens(1)._2 == stakeBox.tokens.getOrElse(1,(Coll[Byte](),0L))._2 + addStakeProxyBox.tokens(1)._2)

                    ))

                }

                // Conditions for a valid user wallet output box
                val validUserWalletBox: Boolean = {

                    allOf(Coll(

                        // The use obtains the stakey key token 
                        (userWalletBox.tokens(0)._1 == newStakeBox.R5[Coll[Byte]].get),
                        (userWalletBox.tokens(0)._2 == 1L)

                    ))

                }

                // Conditions for adding stake
                val validAddStake: Boolean = {

                    allOf(Coll(
                        validNewStakeStateBox,
                        validNewStakeBox,
                        validUserWalletBox
                    ))

                }

                validAddStake

            }

        } else {
            false
        }

    }

    // ===== Perform Emit Tx ===== //

    // Check conditions for valid Emit tx
    val validEmitTx: Boolean = {

        // Emit Tx Input
        val stakeStateBox: Box = INPUTS(0)
        val stakePoolBox: Box = INPUTS(1)

        // Conditions for valid emit tx inputs
        val validEmitInputs: Boolean = {

            allOf(Coll(
                (stakePoolBox.tokens(0)._1 == StakePoolNFT),  // The input stake pool box must have the stake pool NFT as a token
                (INPUTS.size >= 3)                            // The stake state box, stake pool box, emission box, and staking incentive box
            ))

        }

        if (validEmitInputs) {

            val emissionBox: Box = INPUTS(2)

            // Conditions for a valid output stake state box
            val validNewStakeStateBox: Boolean = {

                allOf(Coll(

                    // Check that the new stake state box has the same parameters
                    validSelfReplication,

                    // Check that the staking checkpoint has increased by one
                    (checkpointOut == checkpoint + 1L),

                    // Check that the amount of stakers in the protocol at the time of the emit tx has not changed
                    (stakersOut == stakers),

                    // Check that the checkpoint timestamp of the output stake state box has increased by the staking cycle duration
                    (checkpointTimestampOut == checkpointTimestamp + cycleDuration),

                    // Check that the new checkpoint timestamp occurs within the time that the block is created
                    (checkpointTimestampOut < BlockTime),

                    // Check that the new stake state box also has the same stake tokens as the input stake state box
                    (newStakeStateBox.tokens(1)._2 == SELF.tokens(1)._2)

                ))

            }

            // Conditions for a valid output emission box
            val validNewEmissionBox: Boolean = {

                allOf(Coll(

                    // Check that the input emission box contains the emission NFT
                    (emissionBox.tokens(0)._1 == EmissionNFT),

                    // Check that the input emission box contains the previous checkpoint value as a parameter (i.e. it comes from the previous staking cycle)
                    (emissionBox.R4[Coll[Long]].get(1) == checkpoint - 1L),

                    // Check that the input emission box contains no more stakers as a parameter (i.e. all stakers in the previous emission cycle have received tokens from the compount tx)
                    (emissionBox.R4[Coll[Long]].get(2) == 0L)

                ))

            }

            allOf(Coll(
                validNewStakeStateBox,
                validNewEmissionBox
            ))


        } else {
            false
        }

    }

    // ===== Perform Unstake Tx ===== //
    
    val validUnstakeTx: Boolean = {

        // Unstake Tx Inputs
        val stakeStateBox:   Box = INPUTS(0)
        val stakeBox:        Box = INPUTS(1)
        val unstakeProxyBox: Box = INPUTS(2)

        // Unstake Tx Outputs
        val userWalletBox:   Box = OUTPUTS(1)

        // Conditions for valid unstake tx inputs
        val validUnstakeInputs: Boolean = {

            allOf(Coll(
                (totalAmountStaked > totalAmountStakedOut),  // Funds are being withdrawn from the total amount staked
                (INPUTS.size >= 3),                          // The stake state box, the stake box, and the unstake proxy box
                (stakeBox.tokens.size > 1)                   // The stake box should have DAO tokens inside of it ready to be withdrawn from the staking protocol
            ))

        }

        if (validUnstakeInputs) {

            // Unstake calculation variables
            val unstakedAmount: Long = totalAmountStaked - totalAmountStakedOut
            val stakeKey: Coll[Byte] = stakeBox.R5[Coll[Byte]].get 
            val remainingStakeAmount: Long = stakeBox.tokens(1)._2 - unstakedAmount

            // Condition for a partial unstake tx           
            val isPartialUnstake: Boolean = (remainingStakeAmount > 0)  // The must still be some remaining amount of DAO tokens staked in the stake box

            // Conditions for valid inputs and outputs for the unstake tx (partial and full unstake)
            val validUnstakeInputsAndOutputs: Boolean = {

                // Conditions for a valid stake box
                val validStakeBox: Boolean = {

                    allOf(Coll(
                        (stakeBox.tokens(0)._1 == stakeTokenID),        // The stake box must still contain the stake token given from the stake state box
                        (stakeBox.R4[Coll[Long]].get(0) == checkpoint)  // The input stake box must be the same as this current box with the same emission checkpoint
                    ))

                }

                // Conditions for a valid unstake proxy input box
                val validUnstakeProxyBox: Boolean = {
                    (unstakeProxyBox.tokens(0)._1 == stakeKey)  // The unstake proxy box must contain the stake key, sent to it by the user
                }

                // Conditions for a valid new stake state output box
                val validNewStakeStateBox: Boolean = {

                    allOf(Coll(

                        // Check that the new stake state box has the same parameters
                        validSelfReplication,

                        // Check that the total stake amount is conserved properly
                        (totalAmountStakedOut == totalAmountStaked - unstakedAmount),

                        // The unstake must occure within the same emission cycle, with the same checkpoint
                        (checkpointOut == checkpoint),

                        // The amount of stakers should decrease by 1 if a full unstake occurs
                        (stakersOut == stakers - (if (!isPartialUnstake) 1L else 0L)),

                        // Check that the unstake tx occurs within the same emission cycle, with the same checkpoit timestamp
                        (checkpointTimestampOut == checkpointTimestamp),

                        // The stake token from the stake box should be given back to the stake state box after a full unstake
                        (newStakeStateBox.tokens(1)._2 == SELF.tokens(1)._2 + (if (!isPartialUnstake) 1L else 0L))

                    ))

                }

                // Conditions for a valid user wallet output box
                val validUserWalletBox: Boolean = {

                    allOf(Coll(
                        
                        // Check that the user receives their unstaked DAO tokens back
                        (userWalletBox.tokens(0)._1 == stakeBox.tokens(1)._1),
                        (userWalletBox.tokens(0)._2 == unstakedAmount)

                    ))

                }

                allOf(Coll(
                    validStakeBox,
                    validUnstakeProxyBox,
                    validNewStakeStateBox,
                    validUserWalletBox
                ))
                

            }

            if (isPartialUnstake) {

                // ===== Perform Partial Unstake Tx ===== //

                val validPartialUnstake: Boolean = {

                    // Parital Unstake Tx Outputs
                    val newStakeBox: Box = OUTPUTS(2)

                    // Conditions for valid partial unstake inputs and outputs
                    val validPartialUnstakeInputsAndOutputs: Boolean = {

                        // Conditions for the new stake output box, since a partial unstake would still require tokens to be staked in the protocol
                        val validNewStakeBox: Boolean = {

                            allOf(Coll(
                                
                                // ERG value to make the box exists is preserved
                                (newStakeBox.value == stakeBox.value),

                                // Check that the output stake box is guarded by the same contract as the input stake box
                                (newStakeBox.propositionBytes == stakeBox.propositionBytes),

                                // The stake token from the previous stake box is preserved for the new stake box 
                                (newStakeBox.tokens.getOrElse(0, (Coll[Byte](), 0L))._1 == stakeBox.tokens(0)._1),
                                (newStakeBox.tokens.getOrElse(0, (Coll[Byte](), 0L))._2 == stakeBox.tokens(0)._2),
                                
                                // The new stake box must contain the remaining staked DAO tokens after the partial unstake
                                (newStakeBox.tokens(1)._1 == stakeBox.tokens(1)._1),
                                (newStakeBox.tokens(1)._2 == remainingStakeAmount),
                                
                                // The new stake box keeps the same parameters as the previous stake box: Checkpoint and Staking Time
                                (newStakeBox.R4[Coll[Long]].get(0) == stakeBox.R4[Coll[Long]].get(0)),
                                (newStakeBox.R4[Coll[Long]].get(1) == stakeBox.R4[Coll[Long]].get(1))
                            ))

                        }

                        // Conditions for a valid user wallet output box
                        val validUserWalletBox: Boolean = {

                            // The user must get back their stake key, since it is not a full unstake
                            (userWalletBox.tokens.getOrElse(1, (Coll[Byte](), 0L))._1 == stakeKey)

                        }

                        allOf(Coll(
                            validNewStakeBox,
                            validUserWalletBox
                        ))

                    }

                    allOf(Coll(
                        validUnstakeInputsAndOutputs,
                        validPartialUnstakeInputsAndOutputs,
                        (remainingStakeAmount >= MinimumStake)
                    ))

                }

                validPartialUnstake

            } else {

                // ===== Perform Full Unstake Tx ===== //

                // Conditions for a full unstake
                val validFullUnstake: Boolean = {
                    validUnstakeInputsAndOutputs
                }

                validFullUnstake

            }

        } else {
            false
        }

    }

    sigmaProp(validStakeTx || validEmitTx || validUnstakeTx)

}