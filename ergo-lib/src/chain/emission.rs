//! Emission rules for the Ergo blockchain.
//!
//! Ports [`EmissionRules`] and [`MonetarySettings`] from the JVM's
//! `sigmastate-interpreter` to Rust. These compute the total coin supply,
//! per-block rewards, and foundation allocations from chain parameters.

use core::cmp;

/// Number of nanoERG in one ERG.
pub const COINS_IN_ONE_ERGO: i64 = 1_000_000_000;

/// Monetary parameters controlling the Ergo emission schedule.
///
/// Default values match the Ergo mainnet / testnet reference configuration
/// from the JVM `MonetarySettings` case class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonetarySettings {
    /// Number of blocks during which the total emission rate is fixed.
    pub fixed_rate_period: i32,
    /// Number of blocks in one epoch after the fixed-rate period.
    pub epoch_length: i32,
    /// Total emission per block during the fixed-rate period (nanoERG).
    pub fixed_rate: i64,
    /// Reduction in total emission per epoch (nanoERG).
    pub one_epoch_reduction: i64,
    /// Number of blocks a miner reward must mature before spending.
    pub miner_reward_delay: i32,
    /// Foundation's share of each block reward during the first epochs (nanoERG).
    pub founders_initial_reward: i64,
}

impl Default for MonetarySettings {
    fn default() -> Self {
        Self {
            fixed_rate_period: 30 * 2 * 24 * 365,
            epoch_length: 90 * 24 * 30,
            fixed_rate: 75 * COINS_IN_ONE_ERGO,
            one_epoch_reduction: 3 * COINS_IN_ONE_ERGO,
            miner_reward_delay: 720,
            founders_initial_reward: 7_500_000_000,
        }
    }
}

/// Computed emission schedule derived from [`MonetarySettings`].
///
/// Eagerly computes total coin supply and allocation by iterating all
/// block heights at construction time.
#[derive(Debug, Clone)]
pub struct EmissionRules {
    /// The monetary parameters this schedule was computed from.
    pub settings: MonetarySettings,
    /// Total coins that will ever be emitted (nanoERG).
    coins_total: i64,
    /// Last block height with positive emission.
    blocks_total: i32,
    /// Total coins allocated to the foundation (nanoERG).
    founders_coins_total: i64,
    /// Total coins allocated to miners (nanoERG).
    miners_coins_total: i64,
}

impl EmissionRules {
    /// Compute emission rules from the given monetary settings.
    ///
    /// Iterates through all block heights to compute total coin supply.
    pub fn new(settings: MonetarySettings) -> Self {
        let (coins_total, blocks_total) = {
            let mut height = 1i32;
            let mut acc = 0i64;
            loop {
                let current_rate = emission_at_height_impl(&settings, i64::from(height));
                if current_rate > 0 {
                    acc += current_rate;
                    height += 1;
                } else {
                    break (acc, height - 1);
                }
            }
        };
        let founders_coins_total = remaining_foundation_reward_impl(&settings, 0);
        let miners_coins_total = coins_total - founders_coins_total;
        EmissionRules {
            settings,
            coins_total,
            blocks_total,
            founders_coins_total,
            miners_coins_total,
        }
    }

    /// Total coins that will ever be emitted (nanoERG).
    pub fn coins_total(&self) -> i64 {
        self.coins_total
    }

    /// Last block height with positive emission.
    pub fn blocks_total(&self) -> i32 {
        self.blocks_total
    }

    /// Total coins allocated to the foundation (nanoERG).
    pub fn founders_coins_total(&self) -> i64 {
        self.founders_coins_total
    }

    /// Total coins allocated to miners (nanoERG).
    pub fn miners_coins_total(&self) -> i64 {
        self.miners_coins_total
    }

    /// Total emission (miners + foundation) at the given height (nanoERG).
    ///
    /// Returns the full block reward before splitting between miners and
    /// foundation.
    pub fn emission_at_height(&self, h: i64) -> i64 {
        emission_at_height_impl(&self.settings, h)
    }

    /// Remaining foundation reward available at the given height (nanoERG).
    ///
    /// This is the value the foundation box should hold at height `h`.
    pub fn remaining_foundation_reward_at_height(&self, h: i64) -> i64 {
        remaining_foundation_reward_impl(&self.settings, h)
    }

    /// Miner's share of the block reward at the given height (nanoERG).
    pub fn miners_reward_at_height(&self, h: i64) -> i64 {
        miners_reward_at_height_impl(&self.settings, h)
    }
}

/// Total emission at height `h` (pure function on settings).
fn emission_at_height_impl(s: &MonetarySettings, h: i64) -> i64 {
    if h < i64::from(s.fixed_rate_period) {
        s.fixed_rate
    } else {
        let epoch = 1 + (h - i64::from(s.fixed_rate_period)) / i64::from(s.epoch_length);
        cmp::max(s.fixed_rate - s.one_epoch_reduction * epoch, 0)
    }
}

/// Remaining foundation reward at height `h` (pure function on settings).
fn remaining_foundation_reward_impl(s: &MonetarySettings, h: i64) -> i64 {
    let fir = s.founders_initial_reward;
    let oer = s.one_epoch_reduction;
    let el = i64::from(s.epoch_length);
    let frp = i64::from(s.fixed_rate_period);
    let full15 = (fir - 2 * oer) * el;
    let full45 = (fir - oer) * el;

    if h < frp {
        full15 + full45 + (frp - h - 1) * fir
    } else if h < frp + el {
        full15 + (fir - oer) * (frp + el - h - 1)
    } else if h < frp + 2 * el {
        (fir - 2 * oer) * (frp + 2 * el - h - 1)
    } else {
        0
    }
}

/// Miner's reward at height `h` (pure function on settings).
fn miners_reward_at_height_impl(s: &MonetarySettings, h: i64) -> i64 {
    if h < i64::from(s.fixed_rate_period) + 2 * i64::from(s.epoch_length) {
        s.fixed_rate - s.founders_initial_reward
    } else {
        let epoch = 1 + (h - i64::from(s.fixed_rate_period)) / i64::from(s.epoch_length);
        cmp::max(s.fixed_rate - s.one_epoch_reduction * epoch, 0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    fn default_rules() -> EmissionRules {
        EmissionRules::new(MonetarySettings::default())
    }

    #[test]
    fn test_default_settings_constants() {
        let s = MonetarySettings::default();
        assert_eq!(s.fixed_rate_period, 525_600);
        assert_eq!(s.epoch_length, 64_800);
        assert_eq!(s.fixed_rate, 75_000_000_000);
        assert_eq!(s.one_epoch_reduction, 3_000_000_000);
        assert_eq!(s.miner_reward_delay, 720);
        assert_eq!(s.founders_initial_reward, 7_500_000_000);
    }

    #[test]
    fn test_coins_total_identity() {
        let rules = default_rules();
        assert_eq!(
            rules.coins_total(),
            rules.miners_coins_total() + rules.founders_coins_total()
        );
    }

    #[test]
    fn test_miners_coins_total() {
        let rules = default_rules();
        assert_eq!(rules.miners_coins_total(), 93_409_132_500_000_000);
    }

    #[test]
    fn test_emission_at_height_fixed_rate() {
        let rules = default_rules();
        assert_eq!(rules.emission_at_height(0), 75_000_000_000);
        assert_eq!(rules.emission_at_height(1), 75_000_000_000);
        assert_eq!(rules.emission_at_height(525_599), 75_000_000_000);
    }

    #[test]
    fn test_emission_at_height_epoch_transitions() {
        let rules = default_rules();
        // First epoch after fixed rate: rate drops by oneEpochReduction
        assert_eq!(rules.emission_at_height(525_600), 72_000_000_000);
        assert_eq!(rules.emission_at_height(590_399), 72_000_000_000);
        // Second epoch
        assert_eq!(rules.emission_at_height(590_400), 69_000_000_000);
    }

    #[test]
    fn test_emission_at_height_last_epoch() {
        let rules = default_rules();
        // Last epoch with positive emission (epoch 24, rate = 3e9)
        let last_height = rules.blocks_total() as i64;
        assert!(rules.emission_at_height(last_height) > 0);
        assert_eq!(rules.emission_at_height(last_height + 1), 0);
    }

    #[test]
    fn test_remaining_foundation_reward_boundaries() {
        let rules = default_rules();
        // At height 0, full foundation allocation
        assert_eq!(
            rules.remaining_foundation_reward_at_height(0),
            rules.founders_coins_total()
        );
        // After foundation period ends (fixedRatePeriod + 2*epochLength)
        let after = i64::from(rules.settings.fixed_rate_period)
            + 2 * i64::from(rules.settings.epoch_length);
        assert_eq!(rules.remaining_foundation_reward_at_height(after), 0);
    }

    #[test]
    fn test_miners_reward_during_fixed_period() {
        let rules = default_rules();
        // During fixed rate + 2 epochs: fixedRate - foundersInitialReward
        assert_eq!(rules.miners_reward_at_height(0), 67_500_000_000);
        assert_eq!(rules.miners_reward_at_height(1), 67_500_000_000);
    }

    #[test]
    fn test_miners_reward_after_foundation() {
        let rules = default_rules();
        // After foundation period, miners get full emission
        let after = i64::from(rules.settings.fixed_rate_period)
            + 2 * i64::from(rules.settings.epoch_length);
        assert_eq!(
            rules.miners_reward_at_height(after),
            rules.emission_at_height(after)
        );
    }
}
