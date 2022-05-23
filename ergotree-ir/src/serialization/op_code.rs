#![allow(dead_code)]
#![allow(missing_docs)]

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable, SigmaSerializeResult,
};

use super::sigma_byte_writer::SigmaByteWrite;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct OpCode(u8);

impl OpCode {
    // reference implementation
    // https://github.com/ScorexFoundation/sigmastate-interpreter/blob/develop/sigmastate/src/main/scala/sigmastate/serialization/OpCodes.scala

    /// Decoding of types depends on the first byte and in general is a recursive procedure
    /// consuming some number of bytes from Reader.
    /// All data types are recognized by the first byte falling in the
    /// region [FIRST_DATA_TYPE .. LAST_DATA_TYPE]
    pub const FIRST_DATA_TYPE: OpCode = OpCode(1);
    pub const LAST_DATA_TYPE: OpCode = OpCode(111);

    /// We use optimized encoding of constant values to save space in serialization.
    /// Since Box registers are stored as Constant nodes we save 1 byte for each register.
    /// This is due to convention that Value.opCode falling in [1..LastDataType] region is a constant.
    /// Thus, we can just decode an instance of SType and then decode
    /// data using [`crate::serialization::data::DataSerializer`].
    /// Decoding of constants depends on the first byte and in general is a recursive procedure
    /// consuming some number of bytes from Reader.
    pub const CONSTANT_CODE: OpCode = OpCode(0);
    /// The last constant code is equal to [`crate::serialization::types::TypeCode::FIRST_FUNC_TYPE`] which represent
    /// generic function type.
    /// We use this single code to represent all functional constants, since we don't have
    /// enough space in single byte.
    /// Subsequent bytes have to be read from Reader in order to decode the type of the function
    /// and the corresponding data.
    pub const LAST_CONSTANT_CODE: OpCode = OpCode(Self::LAST_DATA_TYPE.value() + 1);

    pub const VAL_USE: OpCode = Self::new_op_code(2);
    pub const CONSTANT_PLACEHOLDER: OpCode = Self::new_op_code(3);
    pub const SUBST_CONSTANTS: OpCode = Self::new_op_code(4); // reserved 5 - 9 (5)

    pub const LONG_TO_BYTE_ARRAY: OpCode = Self::new_op_code(10);
    pub const BYTE_ARRAY_TO_BIGINT: OpCode = Self::new_op_code(11);
    pub const BYTE_ARRAY_TO_LONG: OpCode = Self::new_op_code(12);
    pub const DOWNCAST: OpCode = Self::new_op_code(13);
    pub const UPCAST: OpCode = Self::new_op_code(14);

    pub const TRUE: OpCode = Self::new_op_code(15);
    pub const FALSE: OpCode = Self::new_op_code(16);
    pub const UNIT_CONSTANT: OpCode = Self::new_op_code(17);
    pub const GROUP_GENERATOR: OpCode = Self::new_op_code(18);
    pub const COLL: OpCode = Self::new_op_code(19); // reserved 20
    pub const COLL_OF_BOOL_CONST: OpCode = Self::new_op_code(21);

    pub const TUPLE: OpCode = Self::new_op_code(22);
    pub const SELECT_1: OpCode = Self::new_op_code(23);
    pub const SELECT_2: OpCode = Self::new_op_code(24);
    pub const SELECT_3: OpCode = Self::new_op_code(25);
    pub const SELECT_4: OpCode = Self::new_op_code(26);
    pub const SELECT_5: OpCode = Self::new_op_code(27);
    pub const SELECT_FIELD: OpCode = Self::new_op_code(28);

    // Relation ops codes
    pub const LT: OpCode = Self::new_op_code(31);
    pub const LE: OpCode = Self::new_op_code(32);
    pub const GT: OpCode = Self::new_op_code(33);
    pub const GE: OpCode = Self::new_op_code(34);
    pub const EQ: OpCode = Self::new_op_code(35);
    pub const NEQ: OpCode = Self::new_op_code(36);
    pub const IF: OpCode = Self::new_op_code(37);
    pub const AND: OpCode = Self::new_op_code(38);
    pub const OR: OpCode = Self::new_op_code(39);
    pub const ATLEAST: OpCode = Self::new_op_code(40);

    // Arithmetic codes
    pub const MINUS: OpCode = Self::new_op_code(41);
    pub const PLUS: OpCode = Self::new_op_code(42);
    pub const XOR: OpCode = Self::new_op_code(43);
    pub const MULTIPLY: OpCode = Self::new_op_code(44);
    pub const DIVISION: OpCode = Self::new_op_code(45);
    pub const MODULO: OpCode = Self::new_op_code(46);
    pub const EXPONENTIATE: OpCode = Self::new_op_code(47);
    pub const MULTIPLY_GROUP: OpCode = Self::new_op_code(48);
    pub const MIN: OpCode = Self::new_op_code(49);
    pub const MAX: OpCode = Self::new_op_code(50);

    /// Environment (context methods)
    pub const HEIGHT: OpCode = Self::new_op_code(51);
    pub const INPUTS: OpCode = Self::new_op_code(52);
    pub const OUTPUTS: OpCode = Self::new_op_code(53);
    pub const LAST_BLOCK_UTXO_ROOT_HASH: OpCode = Self::new_op_code(54);
    pub const SELF_BOX: OpCode = Self::new_op_code(55); // reserved 56 - 59 (4)

    pub const MINER_PUBKEY: OpCode = Self::new_op_code(60);

    // Collection and tree operations codes
    pub const MAP: OpCode = Self::new_op_code(61);
    pub const EXISTS: OpCode = Self::new_op_code(62);
    pub const FOR_ALL: OpCode = Self::new_op_code(63);
    pub const FOLD: OpCode = Self::new_op_code(64);
    pub const SIZE_OF: OpCode = Self::new_op_code(65);
    pub const BY_INDEX: OpCode = Self::new_op_code(66);
    pub const APPEND: OpCode = Self::new_op_code(67);
    pub const SLICE: OpCode = Self::new_op_code(68);
    pub const FILTER: OpCode = Self::new_op_code(69);
    pub const AVL_TREE: OpCode = Self::new_op_code(70);
    pub const AVT_TREE_GET: OpCode = Self::new_op_code(71);
    pub const FLAT_MAP: OpCode = Self::new_op_code(72); // reserved 73 - 80 (8)

    // Type casts codes
    pub const EXTRACT_AMOUNT: OpCode = Self::new_op_code(81);
    pub const EXTRACT_SCRIPT_BYTES: OpCode = Self::new_op_code(82);
    pub const EXTRACT_BYTES: OpCode = Self::new_op_code(83);
    pub const EXTRACT_BYTES_WITH_NO_REF: OpCode = Self::new_op_code(84);
    pub const EXTRACT_ID: OpCode = Self::new_op_code(85);
    pub const EXTRACT_REGISTER_AS: OpCode = Self::new_op_code(86);
    pub const EXTRACT_CREATION_INFO: OpCode = Self::new_op_code(87); // reserved 88 - 90 (3)

    // Cryptographic operations codes
    pub const CALC_BLAKE2B256: OpCode = Self::new_op_code(91);
    pub const CALC_SHA256: OpCode = Self::new_op_code(92);
    pub const PROVE_DLOG: OpCode = Self::new_op_code(93);
    pub const PROVE_DIFFIE_HELLMAN_TUPLE: OpCode = Self::new_op_code(94);
    pub const SIGMA_PROP_IS_PROVEN: OpCode = Self::new_op_code(95);
    pub const SIGMA_PROP_BYTES: OpCode = Self::new_op_code(96);
    pub const BOOL_TO_SIGMA_PROP: OpCode = Self::new_op_code(97);
    pub const TRIVIAL_PROP_FALSE: OpCode = Self::new_op_code(98);
    pub const TRIVIAL_PROP_TRUE: OpCode = Self::new_op_code(99);

    // Deserialization codes
    pub const DESERIALIZE_CONTEXT: OpCode = Self::new_op_code(100);
    pub const DESERIALIZE_REGISTER: OpCode = Self::new_op_code(101); // Block codes
    pub const VAL_DEF: OpCode = Self::new_op_code(102);
    pub const FUN_DEF: OpCode = Self::new_op_code(103);
    pub const BLOCK_VALUE: OpCode = Self::new_op_code(104);
    pub const FUNC_VALUE: OpCode = Self::new_op_code(105);
    pub const APPLY: OpCode = Self::new_op_code(106);
    pub const PROPERTY_CALL: OpCode = Self::new_op_code(107);
    pub const METHOD_CALL: OpCode = Self::new_op_code(108);
    pub const GLOBAL: OpCode = Self::new_op_code(109);

    pub const SOME_VALUE: OpCode = Self::new_op_code(110);
    pub const NONE_VALUE: OpCode = Self::new_op_code(111);

    pub const GET_VAR: OpCode = Self::new_op_code(115);
    pub const OPTION_GET: OpCode = Self::new_op_code(116);
    pub const OPTION_GET_OR_ELSE: OpCode = Self::new_op_code(117);
    pub const OPTION_IS_DEFINED: OpCode = Self::new_op_code(118);

    // Modular arithmetic operations codes
    pub const MOD_Q: OpCode = Self::new_op_code(119);
    pub const PLUS_MOD_Q: OpCode = Self::new_op_code(120);
    pub const MINUS_MOD_Q: OpCode = Self::new_op_code(121);

    pub const SIGMA_AND: OpCode = Self::new_op_code(122);
    pub const SIGMA_OR: OpCode = Self::new_op_code(123);
    pub const BIN_OR: OpCode = Self::new_op_code(124);
    pub const BIN_AND: OpCode = Self::new_op_code(125);

    pub const DECODE_POINT: OpCode = Self::new_op_code(126);

    pub const LOGICAL_NOT: OpCode = Self::new_op_code(127);
    pub const NEGATION: OpCode = Self::new_op_code(128);
    pub const BIT_INVERSION: OpCode = Self::new_op_code(129);
    pub const BIT_OR: OpCode = Self::new_op_code(130);
    pub const BIT_AND: OpCode = Self::new_op_code(131);

    pub const BIN_XOR: OpCode = Self::new_op_code(132);

    pub const BIT_XOR: OpCode = Self::new_op_code(133);
    pub const BIT_SHIFT_RIGHT: OpCode = Self::new_op_code(134);
    pub const BIT_SHIFT_LEFT: OpCode = Self::new_op_code(135);
    pub const BIT_SHIFT_RIGHT_ZEROED: OpCode = Self::new_op_code(136);

    pub const COLL_SHIFT_RIGHT: OpCode = Self::new_op_code(137);
    pub const COLL_SHIFT_LEFT: OpCode = Self::new_op_code(138);
    pub const COLL_SHIFT_RIGHT_ZEROED: OpCode = Self::new_op_code(139);

    pub const COLL_ROTATE_LEFT: OpCode = Self::new_op_code(140);
    pub const COLL_ROTATE_RIGHT: OpCode = Self::new_op_code(141);

    pub const CONTEXT: OpCode = Self::new_op_code(142);
    pub const XOR_OF: OpCode = Self::new_op_code(143); // equals to 255

    const fn new_op_code(shift: u8) -> OpCode {
        OpCode(Self::LAST_CONSTANT_CODE.value() + shift)
    }

    pub fn parse(b: u8) -> OpCode {
        OpCode(b)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub const fn shift(self) -> u8 {
        self.0 - Self::LAST_CONSTANT_CODE.value()
    }
}

impl SigmaSerializable for OpCode {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.0)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let code = r.get_u8()?;
        Ok(OpCode::parse(code))
    }
}
