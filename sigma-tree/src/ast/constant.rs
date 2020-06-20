use crate::{chain::ErgoBox, sigma_protocol::SigmaProp, types::SType};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CollPrim {
    CollBoolean(Vec<bool>),
    CollByte(Vec<i8>),
    CollShort(Vec<i16>),
    CollInt(Vec<i32>),
    CollLong(Vec<i64>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ConstantVal {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    BigInt,
    GroupElement,
    SigmaProp(Box<SigmaProp>),
    CBox(Box<ErgoBox>),
    AvlTree,
    CollPrim(CollPrim),
    Coll(Vec<ConstantVal>),
    Tup(Vec<ConstantVal>),
}

impl ConstantVal {
    pub fn sigma_prop(prop: SigmaProp) -> ConstantVal {
        ConstantVal::SigmaProp(Box::new(prop))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Constant {
    pub tpe: SType,
    pub v: ConstantVal,
}

impl Constant {
    pub fn bool(v: bool) -> Constant {
        Constant {
            tpe: SType::SBoolean,
            v: ConstantVal::Boolean(v),
        }
    }

    pub fn byte(v: i8) -> Constant {
        Constant {
            tpe: SType::SByte,
            v: ConstantVal::Byte(v),
        }
    }

    pub fn short(v: i16) -> Constant {
        Constant {
            tpe: SType::SShort,
            v: ConstantVal::Short(v),
        }
    }

    pub fn int(v: i32) -> Constant {
        Constant {
            tpe: SType::SInt,
            v: ConstantVal::Int(v),
        }
    }

    pub fn long(v: i64) -> Constant {
        Constant {
            tpe: SType::SLong,
            v: ConstantVal::Long(v),
        }
    }

    pub fn sigma_prop(prop: SigmaProp) -> Constant {
        Constant {
            tpe: SType::SSigmaProp,
            v: ConstantVal::sigma_prop(prop),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for Constant {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                any::<bool>().prop_map(|v| Constant::bool(v)),
                any::<i8>().prop_map(|v| Constant::byte(v)),
                any::<i16>().prop_map(|v| Constant::short(v)),
                any::<i32>().prop_map(|v| Constant::int(v)),
                any::<i64>().prop_map(|v| Constant::long(v)),
                // TODO: byte array
            ]
            .boxed()
        }
    }
}
