use std::convert::TryFrom;

use serde::Deserialize;
use serde::Serialize;

use crate::sigma_protocol::sigma_boolean::SigmaBoolean;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SigmaBooleanJson {}

impl From<SigmaBoolean> for SigmaBooleanJson {
    fn from(_: SigmaBoolean) -> Self {
        todo!()
    }
}

impl TryFrom<SigmaBooleanJson> for SigmaBoolean {
    type Error = String;

    fn try_from(_value: SigmaBooleanJson) -> Result<Self, Self::Error> {
        todo!()
    }
}
