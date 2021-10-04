//! OR conjunction for sigma proposition
use std::convert::TryInto;

use super::SigmaBoolean;
use super::SigmaConjectureItems;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::{SigmaParsingError, SigmaSerializable, SigmaSerializeResult};
use crate::sigma_protocol::sigma_boolean::SigmaConjecture;

/// OR conjunction for sigma proposition
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Cor {
    /// Items of the conjunctions
    pub items: SigmaConjectureItems<SigmaBoolean>,
}

impl HasStaticOpCode for Cor {
    const OP_CODE: OpCode = OpCode::OR;
}

impl Cor {
    /// Connects the given sigma propositions into COR proposition performing
    /// partial evaluation when some of them are trivial propositioins.
    pub fn normalized(items: SigmaConjectureItems<SigmaBoolean>) -> SigmaBoolean {
        assert!(!items.is_empty());
        let mut res = Vec::new();
        for it in items {
            match it {
                SigmaBoolean::TrivialProp(true) => return it,
                SigmaBoolean::TrivialProp(false) => (),
                _ => res.push(it),
            }
        }
        if res.is_empty() {
            false.into()
        } else if res.len() == 1 {
            #[allow(clippy::unwrap_used)]
            res.first().unwrap().clone()
        } else {
            #[allow(clippy::unwrap_used)]
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(Cor {
                // should be 2 or more so unwrap is safe here
                items: res.try_into().unwrap(),
            }))
        }
    }
}

impl SigmaSerializable for Cor {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.items.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let items = SigmaConjectureItems::<_>::sigma_parse(r)?;
        Ok(Cor { items })
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Cor {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any::<SigmaBoolean>(), 2..=4)
                .prop_map(|items| Cor {
                    items: items.try_into().unwrap(),
                })
                .boxed()
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
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
        let cor = Cor::normalized(vec![true.into(), false.into()].try_into().unwrap());
        assert!(matches!(cor, SigmaBoolean::TrivialProp(true)));
    }

    #[test]
    fn trivial_false() {
        let cor = Cor::normalized(vec![false.into(), false.into()].try_into().unwrap());
        assert!(matches!(cor, SigmaBoolean::TrivialProp(false)));
    }

    #[test]
    fn pk_triv_true() {
        let pk = force_any_val::<ProveDlog>();
        let cor = Cor::normalized(vec![pk.into(), true.into()].try_into().unwrap());
        assert!(matches!(cor, SigmaBoolean::TrivialProp(true)));
    }

    #[test]
    fn pk_triv_false() {
        let pk = force_any_val::<ProveDlog>();
        let cor = Cor::normalized(vec![pk.clone().into(), false.into()].try_into().unwrap());
        let res: ProveDlog = cor.try_into().unwrap();
        assert_eq!(res, pk);
    }

    #[test]
    fn pk_pk() {
        let pk1 = force_any_val::<ProveDlog>();
        let pk2 = force_any_val::<ProveDlog>();
        let pks: SigmaConjectureItems<SigmaBoolean> =
            vec![pk1.into(), pk2.into()].try_into().unwrap();
        let cor = Cor::normalized(pks.clone());
        assert!(matches!(
            cor,
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(Cor {items})) if items == pks
        ));
    }

    proptest! {

        #[test]
        fn sigma_proposition_ser_roundtrip(
            v in any_with::<Cor>(())) {
                prop_assert_eq![sigma_serialize_roundtrip(&v), v]
        }
    }
}
