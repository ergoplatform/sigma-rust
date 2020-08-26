//! ContextExtension type
use crate::{
    ast::Constant,
    serialization::{
        sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SerializationError,
        SigmaSerializable,
    },
};
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

/// User-defined variables to be put into context
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ContextExtension {
    /// key-value pairs of variable id and it's value
    #[cfg_attr(feature = "with-serde", serde(flatten))]
    pub values: HashMap<u8, Constant>,
}

impl ContextExtension {
    /// Returns an empty ContextExtension
    pub fn empty() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl SigmaSerializable for ContextExtension {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u8(self.values.len() as u8)?;
        let mut sorted_values: Vec<(&u8, &Constant)> = self.values.iter().collect();
        sorted_values.sort_by_key(|(k, _)| *k);
        sorted_values.iter().try_for_each(|(idx, c)| {
            w.put_u8(**idx)?;
            c.sigma_serialize(w)
        })?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let values_count = r.get_u8()?;
        let mut values: HashMap<u8, Constant> = HashMap::with_capacity(values_count as usize);
        for _ in 0..values_count {
            let idx = r.get_u8()?;
            values.insert(idx, Constant::sigma_parse(r)?);
        }
        Ok(ContextExtension { values })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for ContextExtension {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any::<Constant>(), 0..10)
                .prop_map(|constants| {
                    let pairs: Vec<(u8, Constant)> = constants
                        .into_iter()
                        .enumerate()
                        .map(|(idx, c)| (idx as u8, c))
                        .collect();
                    Self {
                        values: pairs.into_iter().collect(),
                    }
                })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ContextExtension>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
