//! ProverResult
use std::io;

use super::context_extension::ContextExtension;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SerializationError,
    SigmaSerializable,
};
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

/// Proof of correctness of tx spending
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ProverResult {
    /// proof that satisfies final sigma proposition
    #[cfg_attr(feature = "with-serde", serde(rename = "proofBytes"))]
    pub proof: Vec<u8>,
    /// user-defined variables to be put into context
    #[cfg_attr(feature = "with-serde", serde(rename = "extension"))]
    pub extension: ContextExtension,
}

impl SigmaSerializable for ProverResult {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u16(self.proof.len() as u16)?;
        w.write_all(&self.proof)?;
        self.extension.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let proof_len = r.get_u16()?;
        let mut proof = vec![0; proof_len as usize];
        r.read_exact(&mut proof)?;
        let extension = ContextExtension::sigma_parse(r)?;
        Ok(ProverResult { proof, extension })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::{collection::vec, prelude::*};

    impl Arbitrary for ProverResult {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (vec(any::<u8>(), 0..100), any::<ContextExtension>())
                .prop_map(|(proof, extension)| Self { proof, extension })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ProverResult>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
