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
        (STypeVar(tv1), STypeVar(tv2)) if tv1 == tv2 => unified_without_subst(),
        (STypeVar(id1), t2) if !matches!(t2, STypeVar(_)) => {
            Ok([(id1.clone(), t2.clone())].iter().cloned().collect())
        }
        (t1, t2) if t1.is_prim() && t2.is_prim() && t1 == t2 => unified_without_subst(),
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
        // it is necessary for implicit conversion in Coll(bool, prop, bool)
        (SBoolean, SSigmaProp) => unified_without_subst(),
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
        assert!(
            unify_one(&t1, &t2).is_err(),
            "unification of {:?} and {:?} should fail",
            t1,
            t2
        );
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

    fn check_subst_many(t1: SType, t2: SType, subst: &[(STypeVar, SType)]) {
        check_subst_map(t1, t2, subst.iter().cloned().collect())
    }

    #[test]
    fn unify_empty_subst() {
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
            SFunc::new(vec![SBox], SLong).into(),
            SFunc::new(vec![SBox], SLong).into(),
        );

        check_empty_subst(STypeVar::t().into(), STypeVar::t().into());
        check_empty_subst(
            STuple::pair(SBoolean, STypeVar::t().into()).into(),
            STuple::pair(SBoolean, STypeVar::t().into()).into(),
        );

        check_empty_subst(
            SFunc::new(vec![SBoolean], SInt).into(),
            SFunc::new(vec![SBoolean], SInt).into(),
        );

        check_empty_subst(SBoolean, SSigmaProp);
    }

    #[test]
    fn unify_with_subst() {
        check_subst(STypeVar::t().into(), SInt, (STypeVar::t(), SInt));

        // Coll
        check_subst(
            STypeVar::t().into(),
            SColl(SInt.into()),
            (STypeVar::t(), SColl(SInt.into())),
        );
        check_subst(
            SColl(Box::new(STypeVar::t().into())),
            SColl(SInt.into()),
            (STypeVar::t(), SInt),
        );
        check_subst(
            SColl(Box::new(STypeVar::t().into())),
            SColl(Box::new(STuple::pair(SInt, SBoolean).into())),
            (STypeVar::t(), STuple::pair(SInt, SBoolean).into()),
        );
        check_subst(
            SColl(Box::new(
                STuple::pair(STypeVar::t().into(), SBoolean).into(),
            )),
            SColl(Box::new(STuple::pair(SInt, SBoolean).into())),
            (STypeVar::t(), SInt),
        );

        // Option
        check_subst(
            STypeVar::t().into(),
            SOption(SInt.into()),
            (STypeVar::t(), SOption(SInt.into())),
        );
        check_subst(
            SOption(Box::new(STypeVar::t().into())),
            SOption(SInt.into()),
            (STypeVar::t(), SInt),
        );
        check_subst(
            SOption(Box::new(STypeVar::t().into())),
            SOption(Box::new(STuple::pair(SInt, SBoolean).into())),
            (STypeVar::t(), STuple::pair(SInt, SBoolean).into()),
        );
        check_subst(
            SOption(Box::new(
                STuple::pair(STypeVar::t().into(), SBoolean).into(),
            )),
            SOption(Box::new(STuple::pair(SInt, SBoolean).into())),
            (STypeVar::t(), SInt),
        );

        // SFunc
        check_subst(
            SFunc::new(vec![STypeVar::t().into()], SInt).into(),
            SFunc::new(vec![SBoolean], SInt).into(),
            (STypeVar::t(), SBoolean),
        );
        check_subst(
            SFunc::new(vec![SInt], STypeVar::t().into()).into(),
            SFunc::new(vec![SInt], SBoolean).into(),
            (STypeVar::t(), SBoolean),
        );
        check_subst_many(
            SFunc::new(
                vec![STuple::pair(SInt, STypeVar::iv().into()).into()],
                STypeVar::t().into(),
            )
            .into(),
            SFunc::new(vec![STuple::pair(SInt, SBoolean).into()], SBox).into(),
            &[(STypeVar::t(), SBox), (STypeVar::iv(), SBoolean)],
        );
        check_subst(
            SFunc::new(
                vec![STuple::pair(SInt, STypeVar::t().into()).into()],
                STypeVar::t().into(),
            )
            .into(),
            SFunc::new(vec![STuple::pair(SInt, SBox).into()], SBox).into(),
            (STypeVar::t(), SBox),
        );
        check_subst_many(
            SFunc::new(
                vec![
                    SColl(STypeVar(STypeVar::iv()).into()),
                    SFunc::new(vec![STypeVar::iv().into()], STypeVar::ov().into()).into(),
                ],
                SColl(STypeVar(STypeVar::ov()).into()),
            )
            .into(),
            SFunc::new(
                vec![SColl(SBox.into()), SFunc::new(vec![SBox], SLong).into()],
                SColl(SLong.into()),
            )
            .into(),
            &[(STypeVar::iv(), SBox), (STypeVar::ov(), SLong)],
        )
    }

    #[test]
    fn unify_negative() {
        check_error(SInt, SLong);

        // Tuple
        check_error(SInt, STuple::pair(SInt, SBoolean).into());
        check_error(
            STuple::pair(SBoolean, SInt).into(),
            STuple::pair(SBoolean, SBoolean).into(),
        );
        check_error(STuple::pair(SInt, STypeVar::t().into()).into(), SInt);
        check_error(
            STuple::pair(STypeVar::t().into(), SInt).into(),
            STuple::pair(SBoolean, STypeVar::iv().into()).into(),
        );

        // Coll
        check_error(SColl(SColl(SInt.into()).into()), SColl(SInt.into()));
        check_error(
            SColl(Box::new(STypeVar::t().into())),
            SColl(Box::new(STypeVar::iv().into())),
        );

        // Option
        check_error(SOption(SBoolean.into()), SOption(SInt.into()));
        check_error(SOption(SBoolean.into()), SColl(SInt.into()));

        // SFunc
        check_error(
            SFunc::new(vec![SBox], SLong).into(),
            SFunc::new(vec![SBox], SInt).into(),
        );
        check_error(
            SFunc::new(vec![STypeVar::t().into()], SInt).into(),
            SFunc::new(vec![SInt], SBoolean).into(),
        );
        check_error(
            SFunc::new(vec![SInt], STypeVar::t().into()).into(),
            SFunc::new(vec![SBoolean], SInt).into(),
        );
        check_error(
            SFunc::new(
                vec![STuple::pair(SInt, STypeVar::t().into()).into()],
                STypeVar::t().into(),
            )
            .into(),
            SFunc::new(vec![STuple::pair(SInt, SBoolean).into()], SBox).into(),
        );

        check_error(SSigmaProp, SBoolean);
    }

    #[test]
    fn unify_diff_size() {
        assert!(unify_many(vec![SInt, SLong], vec![SLong]).is_err());
    }
}
