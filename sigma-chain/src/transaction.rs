use crate::data_input::DataInput;
use crate::ergo_box::ErgoBoxCandidate;
use crate::input::Input;

pub struct Transaction {
    pub inputs: Vec<Input>,
    pub data_inputs: Vec<DataInput>,
    pub outputs: Vec<ErgoBoxCandidate>,
}
