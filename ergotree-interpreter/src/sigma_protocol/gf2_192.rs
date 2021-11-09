// TODO: remove after all todo! are implemented
#![allow(clippy::todo)]

use super::challenge::Challenge;

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Gf2_192 {}

impl From<Gf2_192> for Challenge {
    fn from(_: Gf2_192) -> Self {
        todo!()
    }
}

impl From<Challenge> for Gf2_192 {
    fn from(_: Challenge) -> Self {
        todo!()
    }
}
