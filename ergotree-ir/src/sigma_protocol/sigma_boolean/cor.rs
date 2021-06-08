//! OR conjunction for sigma proposition
use std::convert::TryInto;

use super::SigmaBoolean;
use super::SigmaConjectureItems;
use crate::sigma_protocol::sigma_boolean::SigmaConjecture;

/// OR conjunction for sigma proposition
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Cor {
    /// Items of the conjunctions
    pub items: SigmaConjectureItems<SigmaBoolean>,
}

impl Cor {
    /// Connects the given sigma propositions into COR proposition performing
    /// partial evaluation when some of them are trivial propositioins.
    pub fn normalized(items: SigmaConjectureItems<SigmaBoolean>) -> SigmaBoolean {
        assert!(!items.is_empty());
        let mut res = Vec::new();
        for it in items.iter() {
            match it {
                SigmaBoolean::TrivialProp(true) => return it.clone(),
                SigmaBoolean::TrivialProp(false) => (),
                _ => res.push(it.clone()),
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

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma_protocol::sigma_boolean::ProveDlog;
    use sigma_test_util::force_any_val;
    use std::convert::TryInto;

    #[test]
    fn trivial_true() {
        let cor = Cor::normalized(vec![true.into()].try_into().unwrap());
        assert!(matches!(cor, SigmaBoolean::TrivialProp(true)));
    }

    #[test]
    fn trivial_false() {
        let cor = Cor::normalized(vec![false.into()].try_into().unwrap());
        assert!(matches!(cor, SigmaBoolean::TrivialProp(false)));
    }

    #[test]
    fn pk() {
        let pk = force_any_val::<ProveDlog>();
        let cor = Cor::normalized(vec![pk.clone().into()].try_into().unwrap());
        let res: ProveDlog = cor.try_into().unwrap();
        assert_eq!(res, pk);
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
}
