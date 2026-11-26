use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::NodeResponse;

/// Node extended information from /info REST API endpoint
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[repr(C)]
pub struct NodeInfo {
    /// Node name
    pub name: String,
    /// Ergo node app version
    #[serde(rename = "appVersion")]
    pub app_version: String,
}

impl NodeInfo {
    /// Returns true iff the ergo node is at least v4.0.100. This is due to the EIP-37 hard-fork.
    pub fn is_at_least_version_4_0_100(&self) -> bool {
        let trimmed = self.app_version.trim();
        let without_build = match trimmed.split_once('+') {
            Some((prefix, meta)) if !meta.is_empty() => prefix,
            Some(_) => return false,
            None => trimmed,
        };
        let (core, has_prerelease) = match without_build.split_once('-') {
            Some((prefix, pre)) if !pre.is_empty() => (prefix, true),
            Some(_) => return false,
            None => (without_build, false),
        };
        let mut parts = core.split('.');
        let (Some(maj_s), Some(min_s), Some(patch_s)) = (parts.next(), parts.next(), parts.next())
        else {
            return false;
        };
        if parts.next().is_some() {
            return false;
        }
        let parse_u32 = |s: &str| {
            (!s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
                .then(|| s.parse::<u32>().ok())
                .flatten()
        };
        let (Some(major), Some(minor), Some(patch)) =
            (parse_u32(maj_s), parse_u32(min_s), parse_u32(patch_s))
        else {
            return false;
        };
        match (major, minor, patch).cmp(&(4, 0, 100)) {
            Ordering::Greater => true,
            Ordering::Equal => !has_prerelease,
            Ordering::Less => false,
        }
    }
}

impl NodeResponse for NodeInfo {}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(ver: &str) -> NodeInfo {
        NodeInfo {
            name: "test".to_string(),
            app_version: ver.to_string(),
        }
    }

    #[test]
    fn test_is_at_least_version_4_0_100() {
        assert!(!info("4.0.9").is_at_least_version_4_0_100());
        assert!(!info("4.0.12").is_at_least_version_4_0_100());
        assert!(!info("4.0.20").is_at_least_version_4_0_100());
        assert!(!info("4.0.99").is_at_least_version_4_0_100());
        assert!(!info("4.0.100-RC1").is_at_least_version_4_0_100());
        assert!(!info("4.0.100-rc1+build").is_at_least_version_4_0_100());
        assert!(!info("4.0.100-rc.1+build.2").is_at_least_version_4_0_100());
        assert!(!info("3.9.999").is_at_least_version_4_0_100());
        assert!(!info("").is_at_least_version_4_0_100());
        assert!(!info("4.0").is_at_least_version_4_0_100());
        assert!(!info("4.0.100.1").is_at_least_version_4_0_100());
        assert!(!info("4.0.100-").is_at_least_version_4_0_100());
        assert!(!info("4.0.100+").is_at_least_version_4_0_100());
        assert!(!info("4.0.100-+build").is_at_least_version_4_0_100());
        assert!(!info("+4.0.100").is_at_least_version_4_0_100());
        assert!(!info("4.+0.100").is_at_least_version_4_0_100());
        assert!(!info("4.0.+100").is_at_least_version_4_0_100());
        assert!(!info("not-a-version").is_at_least_version_4_0_100());

        assert!(info("4.0.100").is_at_least_version_4_0_100());
        assert!(info("4.0.100+build1").is_at_least_version_4_0_100());
        assert!(info("4.0.100+build.1").is_at_least_version_4_0_100());
        assert!(info("4.0.100+build-1").is_at_least_version_4_0_100());
        assert!(info("4.0.101").is_at_least_version_4_0_100());
        assert!(info("4.0.101-rc.1+build.2").is_at_least_version_4_0_100());
        assert!(info("4.1.0").is_at_least_version_4_0_100());
        assert!(info("5.0.0").is_at_least_version_4_0_100());
        assert!(info("5.0.0-alpha.1").is_at_least_version_4_0_100());
        assert!(info("5.0.14-SNAPSHOT").is_at_least_version_4_0_100());
        assert!(info("5.0.14-rc.1").is_at_least_version_4_0_100());
        assert!(info("6.0.11").is_at_least_version_4_0_100());
        assert!(info("10.0.0").is_at_least_version_4_0_100());
    }
}
