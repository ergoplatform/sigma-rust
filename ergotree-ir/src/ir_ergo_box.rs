use crate::mir::constant::Constant;
use sigma_util::DIGEST32_SIZE;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use thiserror::Error;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct IrBoxId([u8; DIGEST32_SIZE]);

impl IrBoxId {
    pub fn new(id: [u8; DIGEST32_SIZE]) -> Self {
        IrBoxId(id)
    }

    pub fn get_box(
        &self,
        arena: &Rc<dyn IrErgoBoxArena>,
    ) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError> {
        arena.get(self)
    }
}

pub trait IrErgoBoxArena: Debug {
    fn get(&self, id: &IrBoxId) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError>;
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IrErgoBoxDummyArena(pub HashMap<IrBoxId, IrErgoBoxDummy>);

impl IrErgoBoxArena for IrErgoBoxDummyArena {
    fn get(&self, id: &IrBoxId) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError> {
        self.0
            .get(id)
            .cloned()
            .ok_or_else(|| IrErgoBoxArenaError(format!("IrErgoBox with id {0:?} not found", id)))
            .map(|b| Rc::new(b) as Rc<dyn IrErgoBox>)
    }
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("IrErgoBoxArenaError: {0}")]
pub struct IrErgoBoxArenaError(pub String);

pub trait IrErgoBox: Debug {
    fn id(&self) -> &[u8; DIGEST32_SIZE];
    fn value(&self) -> i64;
    fn tokens(&self) -> Vec<(Vec<i8>, i64)>;
    /// R4-R9 optional registere, where element with index 0 is R4, etc.
    fn additional_registers(&self) -> &[Constant];
    fn get_register(&self, id: i8) -> Option<Constant>;
    fn creation_height(&self) -> i32;
    fn script_bytes(&self) -> Vec<u8>;
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IrErgoBoxDummy {
    pub id: IrBoxId,
    pub value: i64,
    pub tokens: Vec<(Vec<i8>, i64)>,
    pub additional_registers: Vec<Constant>,
    pub creation_height: i32,
    pub script_bytes: Vec<u8>,
}

impl IrErgoBox for IrErgoBoxDummy {
    fn id(&self) -> &[u8; DIGEST32_SIZE] {
        todo!()
    }

    fn value(&self) -> i64 {
        todo!()
    }

    fn tokens(&self) -> Vec<(Vec<i8>, i64)> {
        todo!()
    }

    fn additional_registers(&self) -> &[Constant] {
        todo!()
    }

    fn get_register(&self, id: i8) -> Option<Constant> {
        todo!()
    }

    fn creation_height(&self) -> i32 {
        todo!()
    }

    fn script_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use crate::util::AsVecI8;

    use super::*;
    use num::abs;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for IrErgoBoxDummy {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<[u8; DIGEST32_SIZE]>(),
                100000i64..999999999,
                vec(any::<([u8; DIGEST32_SIZE], u64)>(), 0..3),
                1i32..1000,
                vec(any::<Constant>(), 0..5),
                vec(any::<u8>(), 100..1000),
            )
                .prop_map(
                    |(id, value, tokens, creation_height, additional_registers, script_bytes)| {
                        Self {
                            id: IrBoxId(id),
                            value,
                            tokens: tokens
                                .into_iter()
                                .map(|(id, amount)| (id.to_vec().as_vec_i8(), abs(amount as i64)))
                                .collect(),
                            additional_registers,
                            creation_height,
                            script_bytes,
                        }
                    },
                )
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }
}
