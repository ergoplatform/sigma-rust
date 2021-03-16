use std::collections::HashMap;
use std::rc::Rc;

use ergotree_ir::ir_ergo_box::IrBoxId;
use ergotree_ir::ir_ergo_box::IrErgoBox;
use ergotree_ir::ir_ergo_box::IrErgoBoxArena;
use ergotree_ir::ir_ergo_box::IrErgoBoxArenaError;
use ergotree_ir::mir::constant::Constant;
use sigma_util::DIGEST32_SIZE;

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct IrErgoBoxDummyArena(pub HashMap<IrBoxId, IrErgoBoxDummy>);

impl IrErgoBoxArena for IrErgoBoxDummyArena {
    fn get(&self, id: &IrBoxId) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError> {
        self.0
            .get(id)
            .cloned()
            .ok_or_else(|| IrErgoBoxArenaError(format!("IrErgoBox with id {0:?} not found", id)))
            .map(|b| Rc::new(b) as Rc<dyn IrErgoBox>)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct IrErgoBoxDummy {
    pub(crate) id: IrBoxId,
    pub(crate) value: i64,
    pub(crate) tokens: Vec<(Vec<i8>, i64)>,
    pub(crate) additional_registers: Vec<Constant>,
    pub(crate) creation_height: i32,
    pub(crate) script_bytes: Vec<u8>,
    pub(crate) creation_info: (i32, Vec<u8>),
}

impl IrErgoBox for IrErgoBoxDummy {
    fn id(&self) -> &[u8; DIGEST32_SIZE] {
        &self.id.0
    }

    fn value(&self) -> i64 {
        self.value
    }

    fn tokens(&self) -> Vec<(Vec<i8>, i64)> {
        self.tokens.clone()
    }

    fn additional_registers(&self) -> &[Constant] {
        self.additional_registers.as_slice()
    }

    fn get_register(&self, id: i8) -> Option<Constant> {
        match id {
            0 => Some(self.value.into()),
            3 => Some(self.creation_info.clone().into()),
            _ => self.additional_registers.get(id as usize).cloned(),
        }
    }

    fn creation_height(&self) -> i32 {
        self.creation_height
    }

    fn script_bytes(&self) -> Vec<u8> {
        self.script_bytes.clone()
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
pub(crate) mod arbitrary {

    use super::*;
    use ergotree_ir::util::AsVecI8;
    use proptest::collection::vec;
    use proptest::prelude::*;

    use num::abs;

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
                vec(any::<u8>(), DIGEST32_SIZE + 2..=DIGEST32_SIZE + 2),
            )
                .prop_map(
                    |(
                        id,
                        value,
                        tokens,
                        creation_height,
                        additional_registers,
                        script_bytes,
                        tx_id_box_index,
                    )| {
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
                            creation_info: (creation_height, tx_id_box_index),
                        }
                    },
                )
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }
}
