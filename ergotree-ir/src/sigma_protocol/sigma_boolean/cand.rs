//! AND conjunction for sigma proposition
use super::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaConjecture;

/// AND conjunction for sigma proposition
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Cand {
    /// Items of the conjunctions
    pub items: Vec<SigmaBoolean>,
}

impl Cand {
    /// Connects the given sigma propositions into CAND proposition performing
    /// partial evaluation when some of them are trivial propositioins.
    pub fn normalized(items: Vec<SigmaBoolean>) -> SigmaBoolean {
        assert!(!items.is_empty());
        let mut res = Vec::new();
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
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(Cand { items: res }))
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
        let cand = Cand::normalized(vec![true.into()]);
        assert!(matches!(cand, SigmaBoolean::TrivialProp(true)));
    }

    #[test]
    fn trivial_false() {
        let cand = Cand::normalized(vec![false.into()]);
        assert!(matches!(cand, SigmaBoolean::TrivialProp(false)));
    }

    #[test]
    fn pk() {
        let pk = force_any_val::<ProveDlog>();
        let cand = Cand::normalized(vec![pk.clone().into()]);
        let res: ProveDlog = cand.try_into().unwrap();
        assert_eq!(res, pk);
    }

    #[test]
    fn pk_triv_true() {
        let pk = force_any_val::<ProveDlog>();
        let cand = Cand::normalized(vec![pk.clone().into(), true.into()]);
        let res: ProveDlog = cand.try_into().unwrap();
        assert_eq!(res, pk);
    }

    #[test]
    fn pk_triv_false() {
        let pk = force_any_val::<ProveDlog>();
        let cand = Cand::normalized(vec![pk.into(), false.into()]);
        assert!(matches!(cand, SigmaBoolean::TrivialProp(false)));
    }

    #[test]
    fn pk_pk() {
        let pk1 = force_any_val::<ProveDlog>();
        let pk2 = force_any_val::<ProveDlog>();
        let pks = vec![pk1.into(), pk2.into()];
        let cand = Cand::normalized(pks.clone());
        assert!(matches!(
            cand,
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(Cand {items})) if items == pks
        ));
    }
}
