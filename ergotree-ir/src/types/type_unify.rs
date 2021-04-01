use std::collections::HashMap;

use super::stype::SType;
use super::stype_param::STypeVar;
use SType::*;

#[allow(clippy::unnecessary_wraps)]
fn unified_without_subst() -> Result<HashMap<STypeVar, SType>, TypeUnificationError> {
    Ok(HashMap::new())
}

/// Performs pairwise type unification making sure each type variable is equally
/// substituted in all items.
pub fn unify_many(
    items1: Vec<SType>,
    items2: Vec<SType>,
) -> Result<HashMap<STypeVar, SType>, TypeUnificationError> {
    if items1.len() != items2.len() {
        return Err(TypeUnificationError(format!(
            "items lists are different sizes {:?} vs. {:?}",
            items1, items2
        )));
    }
    let list_of_substitutions: Result<Vec<HashMap<STypeVar, SType>>, _> = items1
        .iter()
        .zip(items2)
        .map(|(t1, t2)| unify_one(t1, &t2))
        .collect();
    let mut res = HashMap::new();
    for substitutions in list_of_substitutions? {
        for (type_var, tpe) in substitutions.iter() {
            match res.insert(type_var.clone(), tpe.clone()) {
                Some(previous_val) if previous_val != *tpe => {
                    return Err(TypeUnificationError(format!(
                        "cannot merge new substitution {:?} for {:?} already exist substitution {:?}",
                        tpe, type_var, previous_val
                    )))
                }
                _ => (),
            };
        }
    }
    Ok(res)
}

/// Error on type unification
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TypeUnificationError(pub String);

/// Finds a substitution `subst` of type variables
/// such that unify_types(t1.with_subst(subst), t2) == Ok(emptySubst)
pub fn unify_one(t1: &SType, t2: &SType) -> Result<HashMap<STypeVar, SType>, TypeUnificationError> {
    match (t1, t2) {
        // (STypeVar(tv1), STypeVar(tv2)) => {
        //     if tv1 == tv2 {
        //         unified_without_subst()
        //     } else {
        //         None
        //     }
        // }
        (t1, t2) if t1.is_prim() && t2.is_prim() && t1 == t2 => unified_without_subst(),
        (STypeVar(id1), _) => Ok([(id1.clone(), t2.clone())].iter().cloned().collect()),
        (SColl(elem_type1), SColl(elem_type2)) => unify_one(elem_type1, elem_type2),
        (SColl(elem_type1), STuple(_)) => unify_one(elem_type1, &SAny),
        (STuple(tuple1), STuple(tuple2)) if tuple1.items.len() == tuple2.items.len() => {
            unify_many(tuple1.items.clone().into(), tuple2.items.clone().into())
        }
        (SOption(elem_type1), SOption(elem_type2)) => unify_one(elem_type1, elem_type2),
        (SFunc(sfunc1), SFunc(sfunc2)) => {
            unify_many(sfunc1.t_dom_plus_range(), sfunc2.t_dom_plus_range())
        }
        (SAny, _) => unified_without_subst(),
        (t1, t2) => Err(TypeUnificationError(format!(
            "Cannot unify {:?} and {:?}",
            t1, t2
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::super::stype::tests::primitive_type;
    use super::*;
    use crate::types::sfunc::SFunc;
    use crate::types::stuple::STuple;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn prim_types(t in primitive_type()) {
            prop_assert_eq!(unify_one(&t, &t), unified_without_subst());
            prop_assert_eq!(unify_one(&SAny, &t), unified_without_subst());
            prop_assert_eq!(unify_one(&SAny, &SColl(t.clone().into())), unified_without_subst());
            prop_assert_eq!(unify_one(&SColl(SAny.into()), &SColl(t.clone().into())), unified_without_subst());
            prop_assert_eq!(unify_one(
                &SColl(SAny.into()),
                &STuple(STuple::pair(t.clone(), t.clone()))), unified_without_subst()
            );
            prop_assert_eq!(unify_one(
                &SColl(SAny.into()),
                &STuple(STuple::pair(t.clone(), STuple(STuple::pair(t.clone(), t))))), unified_without_subst()
            );
        }

    }

    fn check_error(t1: SType, t2: SType) {
        assert!(unify_one(&t1, &t2).is_err());
    }

    fn check_subst_map(t1: SType, t2: SType, expect: HashMap<STypeVar, SType>) {
        assert_eq!(unify_one(&t1, &t2).unwrap(), expect);
        assert_eq!(
            unify_one(&t1.with_subst(&expect), &t2),
            unified_without_subst()
        )
    }

    fn check_empty_subst(t1: SType, t2: SType) {
        assert_eq!(
            unify_one(&t1, &t2).unwrap(),
            unified_without_subst().unwrap()
        );
    }

    fn check_subst(t1: SType, t2: SType, subst: (STypeVar, SType)) {
        check_subst_map(t1, t2, [subst].iter().cloned().collect())
    }

    #[test]
    fn unify_positive() {
        check_empty_subst(SInt, SInt);
        check_empty_subst(
            STuple::pair(SInt, SInt).into(),
            STuple::pair(SInt, SInt).into(),
        );
        // tuple as array
        check_empty_subst(SColl(SAny.into()), STuple::pair(SInt, SInt).into());

        check_empty_subst(
            SColl(SColl(SInt.into()).into()),
            SColl(SColl(SInt.into()).into()),
        );

        check_empty_subst(SOption(SInt.into()), SOption(SInt.into()));

        check_empty_subst(
            SFunc::new(vec![SBox], SLong, vec![]).into(),
            SFunc::new(vec![SBox], SLong, vec![]).into(),
        );

        check_subst(STypeVar::t().into(), SInt, (STypeVar::t(), SInt));
    }

    #[test]
    fn unify_negative() {
        check_error(SInt, SLong);
        check_error(SInt, STuple::pair(SInt, SBoolean).into());
        check_error(
            STuple::pair(SBoolean, SInt).into(),
            STuple::pair(SBoolean, SBoolean).into(),
        );
        check_error(SColl(SColl(SInt.into()).into()), SColl(SInt.into()));
        check_error(SOption(SBoolean.into()), SOption(SInt.into()));
        check_error(SOption(SBoolean.into()), SColl(SInt.into()));
        check_error(
            SFunc::new(vec![SBox], SLong, vec![]).into(),
            SFunc::new(vec![SBox], SInt, vec![]).into(),
        );
    }
}
