use proptest::strategy::ValueTree;
use proptest::test_runner::TestRunner;
use proptest::{arbitrary::Arbitrary, prelude::*};

pub fn force_any_val<T: Arbitrary>() -> T {
    let mut runner = TestRunner::default();
    any::<T>().new_tree(&mut runner).unwrap().current()
}

pub fn force_any_val_with<T: Arbitrary>(args: T::Parameters) -> T {
    let mut runner = TestRunner::default();
    any_with::<T>(args).new_tree(&mut runner).unwrap().current()
}
