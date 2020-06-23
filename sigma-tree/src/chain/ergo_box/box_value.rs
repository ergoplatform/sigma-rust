#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::serializer::{SerializationError, SigmaSerializable};
use sigma_ser::vlq_encode;
use std::io;

/// Box value
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct BoxValue(u64);

impl BoxValue {
    /// Create new value (with bounds check)
    pub fn new(v: u64) -> Option<BoxValue> {
        if BoxValue::within_bounds(v) {
            Some(BoxValue(v))
        } else {
            None
        }
    }

    /// Check if a value is in bounds
    pub fn within_bounds(v: u64) -> bool {
        v >= 1 && v <= i64::MAX as u64
    }
}

impl SigmaSerializable for BoxValue {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u64(self.0)
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let v = r.get_u64()?;
        Ok(BoxValue(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{arbitrary::Arbitrary, prelude::*};

    impl Arbitrary for BoxValue {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            // TODO: should be in 1 - i64.max range
            any::<u64>().prop_map(|v| BoxValue(v)).boxed()
        }
    }
}

