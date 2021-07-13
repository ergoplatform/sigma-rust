//! AND conjunction for sigma proposition
use std::convert::TryInto;

use super::SigmaBoolean;
use super::SigmaConjectureItems;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
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

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma_protocol::sigma_boolean::ProveDlog;
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
}
