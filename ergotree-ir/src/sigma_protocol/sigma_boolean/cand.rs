//! AND conjunction for sigma proposition
use std::convert::TryInto;

use super::SigmaBoolean;
use super::SigmaConjectureItems;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::{SigmaParsingError, SigmaSerializable, SigmaSerializeResult};
use crate::sigma_protocol::sigma_boolean::SigmaConjecture;

/// AND conjunction for sigma proposition
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Cand {
    /// Items of the conjunctions
    pub items: SigmaConjectureItems<SigmaBoolean>,
}

impl HasStaticOpCode for Cand {
    const OP_CODE: OpCode = OpCode::AND;
}

impl Cand {
    /// Connects the given sigma propositions into CAND proposition performing
    /// partial evaluation when some of them are trivial propositioins.
    pub fn normalized(items: SigmaConjectureItems<SigmaBoolean>) -> SigmaBoolean {
        let mut res: Vec<SigmaBoolean> = Vec::new();
        for it in items {
            match it {
                SigmaBoolean::TrivialProp(false) => return it,
                SigmaBoolean::TrivialProp(true) => (),
                _ => res.push(it),
            }
        }
        if res.is_empty() {
            true.into()
        } else if res.len() == 1 {
            #[allow(clippy::unwrap_used)]
            res.first().unwrap().clone()
        } else {
            #[allow(clippy::unwrap_used)]
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(Cand {
                // should be 2 or more so unwrap is safe here
                items: res.try_into().unwrap(),
            }))
        }
    }
}

impl SigmaSerializable for Cand {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u16(self.items.len() as u16)?;
        self.items.iter().try_for_each(|i| i.sigma_serialize(w))
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let items_count = r.get_u16()?;
        let mut items = Vec::with_capacity(items_count as usize);
        for _ in 0..items_count {
            items.push(SigmaBoolean::sigma_parse(r)?);
        }
        Ok(Cand {
            items: items.try_into()?,
        })
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Cand {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any::<SigmaBoolean>(), 2..=4)
                .prop_map(|items| Cand {
                    items: items.try_into().unwrap(),
                })
                .boxed()
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::sigma_protocol::sigma_boolean::ProveDlog;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;
    use std::convert::TryInto;

    #[test]
    fn trivial_true() {
        let cand = Cand::normalized(vec![true.into(), true.into()].try_into().unwrap());
        assert!(matches!(cand, SigmaBoolean::TrivialProp(true)));
    }

    #[test]
    fn trivial_false() {
        let cand = Cand::normalized(vec![false.into(), true.into()].try_into().unwrap());
        assert!(matches!(cand, SigmaBoolean::TrivialProp(false)));
    }

    #[test]
    fn pk_triv_true() {
        let pk = force_any_val::<ProveDlog>();
        let cand = Cand::normalized(vec![pk.clone().into(), true.into()].try_into().unwrap());
        let res: ProveDlog = cand.try_into().unwrap();
        assert_eq!(res, pk);
    }

    #[test]
    fn pk_triv_false() {
        let pk = force_any_val::<ProveDlog>();
        let cand = Cand::normalized(vec![pk.into(), false.into()].try_into().unwrap());
        assert!(matches!(cand, SigmaBoolean::TrivialProp(false)));
    }

    #[test]
    fn pk_pk() {
        let pk1 = force_any_val::<ProveDlog>();
        let pk2 = force_any_val::<ProveDlog>();
        let pks: SigmaConjectureItems<SigmaBoolean> =
            vec![pk1.into(), pk2.into()].try_into().unwrap();
        let cand = Cand::normalized(pks.clone());
        assert!(matches!(
            cand,
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(Cand {items})) if items == pks
        ));
    }

    proptest! {

        #[test]
        fn sigma_proposition_ser_roundtrip(
            v in any_with::<Cand>(())) {
                prop_assert_eq![sigma_serialize_roundtrip(&v), v]
        }
    }
}
