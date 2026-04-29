# 15 Significant Pieces of ErgoScript in the Ergo Ecosystem

> Ranked by protocol influence, with the single most important contract identified within each. Data sourced from the Ergo Knowledge Base (EKB) MCP and Ergo Transcripts MCP.

---

## How to Read This Document

Two parallel rankings are presented side-by-side:

| Column | What It Measures |
|--------|-----------------|
| **Protocol Rank** | Influence of the overall protocol on the ecosystem (TVL, usage, foundational importance, downstream dependencies) |
| **Key Contract** | The single ErgoScript contract that is the load-bearing piece of that protocol's on-chain logic |

---

## Ranked by Protocol Influence

### 1. Oracle Pool v2 (EIP-0023)
**Repo:** `ergoplatform/oracle-core`  
**Category:** Infrastructure / Price Feed  
**Key Contract:** `refresh.es` — Refresh Contract

The foundational price-feed primitive for all of Ergo DeFi. Every major stablecoin (SigmaUSD, Dexy/USE, Gluon Gold), every options protocol (SigmaO), and every lending pool (DuckPools) consumes oracle pool output boxes as read-only data inputs. The Refresh Contract is the core of the entire oracle system — it enforces epoch expiry, minimum data point quorum, sorted ordering, max-deviation bounds, TWAP-style averaging, and reward token accounting in a single contract. The collector mechanic (n+1 reward tokens for the transaction assembler) is the on-chain incentive design that keeps oracle operators competing to keep pools fresh.

**Why this contract matters most:** Every other protocol in this list is broken without it. The Refresh Contract is the most upstream dependency in the Ergo DeFi stack.

---

### 2. SigmaUSD / AgeUSD Bank
**Repo:** `DjedAlliance/Djed-Ergo`  
**Category:** Stablecoin  
**Key Contract:** `ageusd-smart-contracts/bank.es` — Bank / State Box Contract

The first stablecoin deployed on Ergo and the longest-running on-chain reserve. Implements the AgeUSD protocol: SigUSD is the stable coin, SigRSV is the reserve coin. The Bank contract guards the State NFT singleton box that holds the entire ERG reserve, tracks circulating SigUSD and SigRSV supply via token quantities, enforces reserve ratio bounds (minting SigUSD requires ≥400% reserve ratio; minting SigRSV requires ≤800%), and reads the oracle rate from a data input bearing the oracle pool NFT. No separate liquidation contract exists — stability emerges solely from the reserve ratio invariants, eliminating an entire attack surface.

**Why this contract matters most:** The State Box is the entire protocol in one contract. It mints, burns, accounts fees, and validates oracle data — all in ErgoScript.

---

### 3. Spectrum Finance DEX
**Repo:** `spectrum-finance/ergo-dex`  
**Category:** DEX / AMM  
**Key Contract:** `n2t_pool.es` / `t2t_pool.es` — Constant Product Pool Contract

The primary decentralized exchange on Ergo and the largest source of on-chain liquidity. Implements a constant function market maker (CFMM / constant product `x*y=k`) across Native-to-Token (N2T) and Token-to-Token (T2T) pool variants. The Pool Contract enforces: LP token issuance proportional to deposits, constant product invariant on swaps with protocol fee deducted, and correct output routing to buyer addresses via proxy swap contracts. The off-chain matcher aggregates user swap requests and assembles valid pool-spending transactions. Dexy/USE forks the Spectrum swap contracts directly for its internal LP.

**Why this contract matters most:** Most ERG-denominated token price discovery happens through Spectrum pools. It is also the template that subsequent AMM designs on Ergo copy or extend.

---

### 4. Rosen Bridge
**Repo:** `rosen-bridge/contract`  
**Category:** Cross-Chain Bridge  
**Key Contract:** `EventTrigger.es` — Event Trigger Contract

The primary cross-chain bridge connecting Ergo to Cardano, Ethereum, Bitcoin, and other chains. The architecture uses a permissioned watcher network: watchers post cryptographic commitments to observed cross-chain events, and when a quorum is reached the commitments are merged into an EventTrigger box. The EventTrigger contract enforces: valid quorum of watcher WIDs, correct RWT token accounting, guard multi-sig authorization, and reward distribution. If consensus fails, a Fraud contract slashes the collateral of malicious watchers. The RwtRepo contract manages the full watcher lifecycle (permit issuance, restaking, revocation).

**Why this contract matters most:** EventTrigger is the settlement surface for all cross-chain asset flows. It is the contract that either releases wrapped assets or triggers slashing.

---

### 5. Dexy / USE (DexyUSD)
**Repo:** `kushti/dexy-stable`  
**Category:** Algorithmic Stablecoin  
**Key Contract:** `bank.es` — Bank Contract

The most architecturally sophisticated stablecoin system on Ergo, currently deployed as the USE stablecoin (~$400K TVL). Unlike SigmaUSD's ratio-gated model, Dexy uses a two-sided stabilization architecture: the Bank mints USE when the LP price rises above the oracle rate (arbitrage minting), and buys USE back from the LP when price falls below peg (bank intervention). The Bank contract acts as a permissioned state machine — it does not encode minting logic directly but accepts state transitions only when authorized satellite contract NFTs (ArbMint, Intervention, FreeMint) are present in the transaction inputs. The DORT buyback loop routes minting fees to purchase DORT from an ERG/DORT LP, which flows to oracle operators — making oracle security economically self-sustaining.

**Why this contract matters most:** The Bank is the reserve vault and the authorization hub. No token can be minted, redeemed, or arbitraged without the Bank contract accepting the transaction.

---

### 6. ErgoMixer
**Repo:** `ergoMixer/ergoMixBack`  
**Category:** Privacy / Mixing  
**Key Contract:** `FullMix.es` — Full Mix Contract

The only production privacy mixer on Ergo, and the canonical demonstration of Sigma protocol ring signatures in ErgoScript. The FullMix Contract guards a completed mix output using an OR-composition of two `proveDlog` statements — a ring signature over both participants' public keys — so an external observer cannot determine which participant is spending the output. The HalfMix Contract handles the open offer phase: either the depositor cancels (reclaim path) or a second participant joins (producing the FullMix pair). The Stealth Contract extends this to one-time-use stealth addresses via non-interactive Diffie-Hellman key exchange.

**Why this contract matters most:** FullMix.es is the earliest and most direct use of Ergo's native cryptographic primitives (Sigma protocols, ring signatures) in a production application. It remains the privacy benchmark for the ecosystem.

---

### 7. SkyHarbor NFT Marketplace
**Repo:** `skyharbor-market/contracts`  
**Category:** NFT Marketplace  
**Key Contract:** `V1_ErgEditsAndOffersV1.es` — Sale Listing Contract

The dominant NFT marketplace on Ergo by volume (9,090+ historical sales, 363,000+ ERG volume). The Sale Listing Contract implements a fixed-price atomic swap: an NFT seller lists a token at a specific ERG price, and a buyer can purchase it by including the correct ERG payment to the seller in the same transaction, with optional royalty routing to the original issuer. The contract enforces payment distribution (seller + marketplace fee + royalty) or allows the seller to reclaim the listing via signature. Implements replay attack prevention and is compatible with EIP-24 NFT collection standards.

**Why this contract matters most:** The most economically active NFT contract on Ergo by transaction count. Established the de facto standard for atomic NFT sales that other marketplaces followed.

---

### 8. Phoenix HodlERG
**Repo:** `PhoenixErgo/phoenix-hodlcoin-contracts`  
**Category:** DeFi Primitive / Bonding Curve  
**Key Contract:** `phoenix_v1_hodlerg_bank.es` — HodlERG Bank Contract

Implements the hodlCoin protocol: a cryptographically enforced proof-of-hold mechanism where token price accrues monotonically through a bonding curve. Users mint hodlERG by depositing ERG; burn fees are partially retained in the reserve rather than fully extracted, ensuring the price per hodlERG can only increase over time. The Bank Contract enforces: Singleton NFT continuity, mint/burn price derived from `reserve / circulating_supply`, correct fee retention (dev fee + burn fee retained, remainder to user), and storage rent accounting for the ERG-based reserve. A parallel hodlToken variant generalizes the mechanism to any native token.

**Why this contract matters most:** Introduced a new DeFi primitive to Ergo — the monotonic bonding curve. The bank contract is a clean, minimal state machine that other protocols now reference as a pattern.

---

### 9. Paideia DAO Framework
**Repo:** `paideiadao/paideia-contracts`  
**Category:** DAO Tooling  
**Key Contract:** `stakeState.es` — Stake State Contract

The primary DAO governance and staking framework on Ergo, used by multiple DAOs including ErgoPad. The StakeState contract governs the core state machine for the staking protocol: it manages total staked amounts, checkpoint tracking (epoch-based emission schedule), staker counts, and validates three transaction types — stake (initial and add-stake), emit (daily reward distribution), and unstake (full and partial withdrawal). Individual stake positions are guarded by `stake.es`, which references the Stake State NFT for authorization. Governance proposals and votes are managed by companion contracts that read staking weight from the state box.

**Why this contract matters most:** StakeState is the single box that all individual stakers and the governance layer reference. Corrupting or exploiting it would compromise the entire DAO's token economy.

---

### 10. Gluon Gold (GluonW)
**Repo:** `StabilityNexus/Gluon-Ergo-Contracts`  
**Category:** Dual Synthetic Asset / Stablecoin  
**Key Contract:** `GluonWBoxGuardScript.es` — Reactor Box Contract

A dual synthetic asset protocol issuing Neutrons (GAU, gold-pegged stablecoin) and Protons (GAUC, leveraged reserve coin) backed by ERG collateral. The Reactor Box Contract governs four transaction types identified purely by directional token/ERG balance changes: Fission (ERG → Neutrons + Protons), Fusion (Neutrons + Protons → ERG), Beta Decay Plus (Protons → Neutrons), and Beta Decay Minus (Neutrons → Protons). Volume-weighted dynamic fees are computed entirely on-chain from rolling 14-day buckets stored in Registers R7/R8. Treasury governance is a multisig SigmaProp in R5, updateable only via treasury authorization.

**Why this contract matters most:** The entire protocol state — reserve, supplies, fees, governance — lives in a single reactor box. The guard script is simultaneously a state machine, an accounting engine, and a governance layer.

---

### 11. DuckPools Lending Protocol
**Repo:** `duckpools/lend-protocol-contracts`  
**Category:** Lending / Borrowing  
**Key Contract:** `childInterest.es` — Child Pool Interest Contract

The first lending protocol on Ergo, supporting collateralized borrowing against ERG and rsBTC/rsADA. The Child Interest Contract tracks historical interest rate calculations for a lending pool, validates that rates are computed correctly based on utilization metrics (borrowed/total ratio), and maintains an append-only interest history without exceeding storage constraints. The parent pool contract reads from child interest boxes to determine current borrow rates. DuckPools also pioneered on-chain options integration via `optionBox.es`, making it the most feature-complete lending implementation on Ergo.

**Why this contract matters most:** The child interest tracking pattern is the mechanism that enables variable-rate lending without off-chain rate oracles. It is the most novel design contribution DuckPools made to Ergo's contract library.

---

### 12. SigmaO — P2P Options Protocol
**Repo:** `ThierryM1212/SigmaO`  
**Category:** Derivatives / Options  
**Key Contract:** `Option.es` — Core Option Reserve Contract

The only production options protocol on Ergo and the template for the EtchaP2P design. The Option Reserve Contract implements a multi-stage state machine over a single persistent box governing the full option lifecycle: Mint (collateral lock), Delivery (option tokens issued to buyer), Exercise (Call or Put, with correct asset/ERG swap enforced on-chain), and Expiry/Settlement (collateral recovery). European and American exercise styles are both supported. Issuer authentication uses `proveDlog(decodePoint(issuerECPoint))` — a Sigma protocol proof rather than a hardcoded P2PK address. The companion `Option_Sell.es` contract prices options using an on-chain Black-Scholes approximation with a precomputed square-root lookup table.

**Why this contract matters most:** The first complete options implementation on an eUTXO chain. Demonstrated that complex multi-leg derivative logic can be encoded in ErgoScript without inter-contract calls.

---

### 13. ChainCash
**Repo:** `BetterMoneyLabs/chaincash`  
**Category:** Layer 2 / Digital Cash  
**Key Contract:** `reserve.es` — Reserve Contract

A kushti-authored Layer 2 digital cash system where ERG-backed notes can circulate off-chain and be redeemed on-chain. The Reserve Contract manages three core operations: redeem (burn note tokens for ERG at oracle-determined gold prices), top-up (add ERG collateral), and mint (issue new note tokens). Redemption validates cryptographic proofs of issued notes using AVL tree inclusion proofs, enabling selective redemption without scanning all outstanding notes. A companion Layer 2 variant adds a contestation period and dispute resolution mechanism. ChainCash is the most direct research-to-production pipeline for Ergo's theoretical Layer 2 capabilities.

**Why this contract matters most:** The reserve.es contract is the trust anchor for the entire note-circulation system. Its AVL-proof-based redemption mechanism is one of the most advanced cryptographic patterns deployed in Ergo production contracts.

---

### 14. ErgoRaffle
**Repo:** `ErgoRaffle/raffle-backend`  
**Category:** Community / Fundraising  
**Key Contract:** `raffle.es` — Raffle Contract

A community fundraising and lottery protocol that has facilitated numerous Ergo ecosystem fundraisers. The Raffle Contract holds ticket purchases (each represented as a native token), enforces a deadline height after which a winner is selected deterministically from the block header hash of the closing block, distributes the prize to the winning ticket holder, and routes the remaining proceeds to the project's designated charity/funding address. Implements a ticket refund path if the minimum funding threshold is not met.

**Why this contract matters most:** One of the earliest multi-participant contracts on Ergo with a provably fair randomness mechanism. Demonstrated verifiable on-chain randomness from block headers as a practical pattern before more sophisticated VRF-based approaches existed.

---

### 15. SigmaFi — Collateralized Bond Protocol
**Repo:** `K-Singh/Sigma-Finance`  
**Category:** Lending / Fixed-Rate Bonds  
**Key Contract:** `BondContractERG.ergo` — ERG Bond Contract

A peer-to-peer fixed-rate lending protocol where borrowers issue bonds collateralized by ERG and native tokens to borrow SigUSD. The Bond Contract implements two mutually exclusive spending paths: early repayment by the borrower before maturity (borrower proves ownership, repays principal + interest, recovers collateral), or liquidation by the lender after maturity (lender proves the bond is past deadline, claims the collateral). The companion `OpenOfferFixedHeightERG.ergo` handles the open order book where lenders browse and fill borrower requests, atomically creating the bond box on fill.

**Why this contract matters most:** Introduced fixed-rate, fixed-term collateralized lending to Ergo. The two-path bond mechanic (repay vs. liquidate) became the reference implementation for peer-to-peer secured debt on eUTXO.

---

## Summary Table

| Rank | Protocol | Category | Key Contract | Primary Pattern |
|------|----------|----------|-------------|-----------------|
| 1 | Oracle Pool v2 | Infrastructure | `refresh.es` | Singleton NFT + reward distribution |
| 2 | SigmaUSD | Stablecoin | `bank.es` | State box + reserve ratio guards |
| 3 | Spectrum Finance DEX | DEX / AMM | `n2t_pool.es` | Constant product CFMM |
| 4 | Rosen Bridge | Bridge | `EventTrigger.es` | Quorum commitment + guard multi-sig |
| 5 | Dexy / USE | Stablecoin | `bank.es` | State machine + satellite NFT auth |
| 6 | ErgoMixer | Privacy | `FullMix.es` | Ring signature (OR proveDlog) |
| 7 | SkyHarbor | NFT Marketplace | `ErgEditsAndOffers.es` | Atomic swap + royalty routing |
| 8 | Phoenix HodlERG | Bonding Curve | `hodlerg_bank.es` | Monotonic bonding curve |
| 9 | Paideia DAO | DAO Tooling | `stakeState.es` | Epoch state machine + checkpoints |
| 10 | Gluon Gold | Dual Synthetic | `GluonWBoxGuardScript.es` | Balance-delta state dispatch |
| 11 | DuckPools | Lending | `childInterest.es` | Append-only interest history |
| 12 | SigmaO | Options | `Option.es` | Multi-stage option lifecycle |
| 13 | ChainCash | Layer 2 Cash | `reserve.es` | AVL proof redemption |
| 14 | ErgoRaffle | Community | `raffle.es` | Block header randomness |
| 15 | SigmaFi | P2P Bonds | `BondContractERG.ergo` | Dual-path bond (repay vs. liquidate) |

---

## Cross-Cutting Patterns

Several design patterns recur across the most influential contracts:

**Singleton / State NFT** — A unique token whose presence authenticates the canonical state box. Used by Oracle Pool, SigmaUSD, Dexy, Gluon, HodlERG, Paideia, Rosen Bridge.

**Data Input (read-only oracle consumption)** — Protocols consume oracle boxes without spending them, enabling parallel oracle readers without contention. Used by SigmaUSD, Dexy, SigmaO, DuckPools.

**Proxy Contracts** — User intent boxes created off-chain and swept by bots into pool-spending transactions. Used by Spectrum DEX, Dexy LP, HodlERG, Paideia staking.

**Sigma Protocol Proofs** — `proveDlog`, `atLeast`, and OR-compositions of discrete-log proofs for authentication and ring signatures. Used by ErgoMixer, SigmaO (issuer auth), Rosen Bridge (guard multi-sig).

**AVL Tree Proofs** — Authenticated data structures for compact state proofs. Used by ChainCash (note redemption), Paideia (staking), oracle pools (multi-feed extensions).

---

*Sources: Ergo Knowledge Base (EKB) MCP — `ergo-knowledge-base.vercel.app`; Ergo Transcripts MCP. Data as of April 2026.*
