//! JIT costing types and per-operation cost constants.
//!
//! Cost values are taken from the Scala reference implementation (sigmastate-interpreter).
//! JitCost uses a 10x scale relative to block costs: `block_cost = jit_cost / 10` (floor).

/// JIT cost value. Uses 10x scale vs block costs.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct JitCost(pub u32);

impl JitCost {
    /// Convert to block cost (divide by 10, floor).
    /// Matches Scala: `block_cost = jit_cost / 10`.
    #[allow(dead_code)]
    pub fn to_block_cost(self) -> u64 {
        (self.0 as u64) / 10
    }
}

impl core::ops::Add for JitCost {
    type Output = JitCost;
    fn add(self, rhs: JitCost) -> JitCost {
        JitCost(self.0 + rhs.0)
    }
}

impl core::ops::Mul<u32> for JitCost {
    type Output = JitCost;
    fn mul(self, rhs: u32) -> JitCost {
        JitCost(self.0 * rhs)
    }
}

/// Fixed cost for an operation (constant regardless of input).
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct FixedCost(pub JitCost);

/// Per-item cost for collection/sequence operations.
/// Total cost = base_cost + per_chunk_cost * ceil(n_items / chunk_size)
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct PerItemCost {
    pub base_cost: JitCost,
    pub per_chunk_cost: JitCost,
    pub chunk_size: u32,
}

impl PerItemCost {
    pub const fn new(base: u32, per_chunk: u32, chunk_size: u32) -> Self {
        PerItemCost {
            base_cost: JitCost(base),
            per_chunk_cost: JitCost(per_chunk),
            chunk_size,
        }
    }

    /// Calculate cost for the given number of items.
    pub fn total_cost(&self, n_items: u32) -> JitCost {
        let chunks = if n_items == 0 {
            0
        } else {
            n_items.div_ceil(self.chunk_size)
        };
        JitCost(self.base_cost.0 + self.per_chunk_cost.0 * chunks)
    }
}

/// Cost that depends on the type of the operand.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct TypeBasedCost {
    /// Cost for BigInt/UnsignedBigInt types
    pub bigint_cost: JitCost,
    /// Cost for all other numeric types
    pub default_cost: JitCost,
}

// =============================================================================
// Per-operation cost constants (from Scala sigmastate-interpreter reference)
// =============================================================================

// --- Values / Core ---
pub const CONST_COST: FixedCost = FixedCost(JitCost(5));
pub const CONST_PLACEHOLDER_COST: FixedCost = FixedCost(JitCost(1));
pub const VAL_USE_COST: FixedCost = FixedCost(JitCost(5));
pub const FUNC_VALUE_COST: FixedCost = FixedCost(JitCost(5));
pub const ADD_TO_ENV_COST: FixedCost = FixedCost(JitCost(5));
pub const APPLY_COST: FixedCost = FixedCost(JitCost(30));
pub const METHOD_CALL_COST: FixedCost = FixedCost(JitCost(4));
pub const PROPERTY_CALL_COST: FixedCost = FixedCost(JitCost(4));
pub const TUPLE_COST: FixedCost = FixedCost(JitCost(15));
pub const CONCRETE_COLLECTION_COST: FixedCost = FixedCost(JitCost(20));
pub const BLOCK_VALUE_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(1),
    per_chunk_cost: JitCost(1),
    chunk_size: 10,
};
pub const SELECT_FIELD_COST: FixedCost = FixedCost(JitCost(10));

// --- Global variables ---
pub const HEIGHT_COST: FixedCost = FixedCost(JitCost(26));
pub const INPUTS_COST: FixedCost = FixedCost(JitCost(10));
pub const OUTPUTS_COST: FixedCost = FixedCost(JitCost(10));
pub const LAST_BLOCK_UTXO_ROOT_HASH_COST: FixedCost = FixedCost(JitCost(15));
pub const SELF_COST: FixedCost = FixedCost(JitCost(10));
pub const CONTEXT_COST: FixedCost = FixedCost(JitCost(1));
pub const GLOBAL_COST: FixedCost = FixedCost(JitCost(5));

// --- Collection operations ---
pub const BY_INDEX_COST: FixedCost = FixedCost(JitCost(30));
pub const SIZE_OF_COST: FixedCost = FixedCost(JitCost(14));
pub const MAP_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(20),
    per_chunk_cost: JitCost(1),
    chunk_size: 10,
};
pub const FILTER_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(20),
    per_chunk_cost: JitCost(1),
    chunk_size: 10,
};
pub const EXISTS_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(3),
    per_chunk_cost: JitCost(1),
    chunk_size: 10,
};
pub const FOR_ALL_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(3),
    per_chunk_cost: JitCost(1),
    chunk_size: 10,
};
pub const FOLD_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(3),
    per_chunk_cost: JitCost(1),
    chunk_size: 10,
};
pub const APPEND_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(20),
    per_chunk_cost: JitCost(2),
    chunk_size: 100,
};
pub const SLICE_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(10),
    per_chunk_cost: JitCost(2),
    chunk_size: 100,
};

// --- Box operations ---
pub const EXTRACT_AMOUNT_COST: FixedCost = FixedCost(JitCost(8));
pub const EXTRACT_SCRIPT_BYTES_COST: FixedCost = FixedCost(JitCost(10));
pub const EXTRACT_BYTES_COST: FixedCost = FixedCost(JitCost(12));
pub const EXTRACT_BYTES_WITH_NO_REF_COST: FixedCost = FixedCost(JitCost(12));
pub const EXTRACT_ID_COST: FixedCost = FixedCost(JitCost(12));
pub const EXTRACT_REGISTER_AS_COST: FixedCost = FixedCost(JitCost(50));
pub const EXTRACT_CREATION_INFO_COST: FixedCost = FixedCost(JitCost(16));

// --- Option operations ---
pub const GET_VAR_COST: FixedCost = FixedCost(JitCost(10));
pub const OPTION_GET_COST: FixedCost = FixedCost(JitCost(15));
pub const OPTION_GET_OR_ELSE_COST: FixedCost = FixedCost(JitCost(20));
pub const OPTION_IS_DEFINED_COST: FixedCost = FixedCost(JitCost(10));

// --- Sigma protocol ---
pub const BOOL_TO_SIGMA_PROP_COST: FixedCost = FixedCost(JitCost(15));
pub const CREATE_PROVE_DLOG_COST: FixedCost = FixedCost(JitCost(10));
pub const CREATE_PROVE_DH_TUPLE_COST: FixedCost = FixedCost(JitCost(20));
pub const SIGMA_AND_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(10),
    per_chunk_cost: JitCost(2),
    chunk_size: 1,
};
pub const SIGMA_OR_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(10),
    per_chunk_cost: JitCost(2),
    chunk_size: 1,
};
pub const SIGMA_PROP_BYTES_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(35),
    per_chunk_cost: JitCost(6),
    chunk_size: 1,
};

// --- Logical operations ---
pub const AND_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(10),
    per_chunk_cost: JitCost(5),
    chunk_size: 32,
};
pub const OR_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(5),
    per_chunk_cost: JitCost(5),
    chunk_size: 64,
};
pub const XOR_OF_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(20),
    per_chunk_cost: JitCost(5),
    chunk_size: 32,
};
pub const LOGICAL_NOT_COST: FixedCost = FixedCost(JitCost(15));
pub const BIN_OR_COST: FixedCost = FixedCost(JitCost(20));
pub const BIN_AND_COST: FixedCost = FixedCost(JitCost(20));
pub const BIN_XOR_COST: FixedCost = FixedCost(JitCost(20));

// --- Arithmetic / BinOp ---
// Type-based: Plus/Minus
pub const ARITH_PLUS_MINUS_COST: TypeBasedCost = TypeBasedCost {
    bigint_cost: JitCost(20),
    default_cost: JitCost(15),
};
// Type-based: Multiply/Divide/Modulo
pub const ARITH_MUL_DIV_MOD_COST: TypeBasedCost = TypeBasedCost {
    bigint_cost: JitCost(25),
    default_cost: JitCost(15),
};
// Type-based: Min/Max
pub const ARITH_MIN_MAX_COST: TypeBasedCost = TypeBasedCost {
    bigint_cost: JitCost(10),
    default_cost: JitCost(5),
};
// Type-based: Gt/Lt/Ge/Le (same cost for all types in Scala)
pub const RELATION_CMP_COST: TypeBasedCost = TypeBasedCost {
    bigint_cost: JitCost(20),
    default_cost: JitCost(20),
};
pub const BIT_OP_COST: FixedCost = FixedCost(JitCost(1));

// --- Numeric conversions ---
pub const LONG_TO_BYTE_ARRAY_COST: FixedCost = FixedCost(JitCost(17));
pub const BYTE_ARRAY_TO_LONG_COST: FixedCost = FixedCost(JitCost(16));
pub const BYTE_ARRAY_TO_BIGINT_COST: FixedCost = FixedCost(JitCost(30));
pub const NEGATION_COST: FixedCost = FixedCost(JitCost(30));
pub const BIT_INVERSION_COST: FixedCost = FixedCost(JitCost(1));

// --- Casting ---
pub const NUMERIC_CAST_COST: TypeBasedCost = TypeBasedCost {
    bigint_cost: JitCost(30),
    default_cost: JitCost(10),
};

// --- Cryptographic ---
pub const CALC_BLAKE2B256_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(20),
    per_chunk_cost: JitCost(7),
    chunk_size: 128,
};
pub const CALC_SHA256_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(80),
    per_chunk_cost: JitCost(8),
    chunk_size: 64,
};
pub const DECODE_POINT_COST: FixedCost = FixedCost(JitCost(300));
pub const EXPONENTIATE_COST: FixedCost = FixedCost(JitCost(900));
pub const MULTIPLY_GROUP_COST: FixedCost = FixedCost(JitCost(40));

// --- Control flow ---
pub const IF_COST: FixedCost = FixedCost(JitCost(10));

// --- Miscellaneous ---
pub const SUBST_CONSTANTS_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(100),
    per_chunk_cost: JitCost(100),
    chunk_size: 1,
};
pub const XOR_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(10),
    per_chunk_cost: JitCost(2),
    chunk_size: 128,
};
pub const ATLEAST_COST: PerItemCost = PerItemCost {
    base_cost: JitCost(20),
    per_chunk_cost: JitCost(3),
    chunk_size: 5,
};

// --- DataValueComparer: per-type equality costs ---
pub const EQ_PRIM_COST: FixedCost = FixedCost(JitCost(3));
pub const EQ_BIGINT_COST: FixedCost = FixedCost(JitCost(5));
pub const EQ_GROUP_ELEMENT_COST: FixedCost = FixedCost(JitCost(172));
pub const EQ_TUPLE_COST: FixedCost = FixedCost(JitCost(4));
pub const EQ_OPTION_COST: FixedCost = FixedCost(JitCost(4));
pub const EQ_AVL_TREE_COST: FixedCost = FixedCost(JitCost(6));
pub const EQ_BOX_COST: FixedCost = FixedCost(JitCost(6));
pub const EQ_PREHEADER_COST: FixedCost = FixedCost(JitCost(4));
pub const EQ_HEADER_COST: FixedCost = FixedCost(JitCost(6));
pub const EQ_COLL_BYTE_PER_ITEM: PerItemCost = PerItemCost::new(15, 2, 128);
// Per-element-type collection equality costs (from Scala DataValueComparer)
pub const EQ_COLL_SHORT_PER_ITEM: PerItemCost = PerItemCost::new(15, 2, 96);
pub const EQ_COLL_INT_PER_ITEM: PerItemCost = PerItemCost::new(15, 2, 64);
pub const EQ_COLL_LONG_PER_ITEM: PerItemCost = PerItemCost::new(15, 2, 48);
pub const EQ_COLL_BOOLEAN_PER_ITEM: PerItemCost = PerItemCost::new(15, 2, 128);
pub const EQ_COLL_BIGINT_PER_ITEM: PerItemCost = PerItemCost::new(15, 7, 5);
pub const EQ_COLL_GROUP_ELEMENT_PER_ITEM: PerItemCost = PerItemCost::new(15, 5, 1);
pub const EQ_COLL_AVL_TREE_PER_ITEM: PerItemCost = PerItemCost::new(15, 5, 2);
pub const EQ_COLL_BOX_PER_ITEM: PerItemCost = PerItemCost::new(15, 5, 1);
pub const EQ_COLL_PREHEADER_PER_ITEM: PerItemCost = PerItemCost::new(15, 3, 1);
pub const EQ_COLL_HEADER_PER_ITEM: PerItemCost = PerItemCost::new(15, 5, 1);
pub const EQ_COLL_DEFAULT_PER_ITEM: PerItemCost = PerItemCost::new(10, 2, 1);

// --- Tree operations ---
// TreeLookup now uses DynamicCost (CreateAvlVerifier + LookupAvlTree), charged in tree_lookup.rs
#[allow(dead_code)]
pub const TREE_LOOKUP_COST: FixedCost = FixedCost(JitCost(30));
pub const CREATE_AVL_TREE_COST: FixedCost = FixedCost(JitCost(20));

// =============================================================================
// Method-level cost constants (from Scala methods.scala)
// =============================================================================

// --- SBox method costs ---
pub const SBOX_TOKENS_COST: FixedCost = FixedCost(JitCost(15));

// --- SHeader property costs ---
pub const SHEADER_PROP_COST: FixedCost = FixedCost(JitCost(10));
pub const SHEADER_CHECK_POW_COST: FixedCost = FixedCost(JitCost(700));

// --- SPreHeader property costs ---
pub const SPREHEADER_PROP_COST: FixedCost = FixedCost(JitCost(10));

// --- SContext method costs ---
pub const SCONTEXT_DATA_INPUTS_COST: FixedCost = FixedCost(JitCost(15));
pub const SCONTEXT_HEADERS_COST: FixedCost = FixedCost(JitCost(15));
pub const SCONTEXT_PRE_HEADER_COST: FixedCost = FixedCost(JitCost(15));
pub const SCONTEXT_SELF_BOX_INDEX_COST: FixedCost = FixedCost(JitCost(20));
pub const SCONTEXT_MINER_PUBKEY_COST: FixedCost = FixedCost(JitCost(20));
pub const SCONTEXT_GET_VAR_FROM_INPUT_COST: FixedCost = FixedCost(JitCost(10));

// --- SGroupElement method costs ---
pub const SGROUP_GET_ENCODED_COST: FixedCost = FixedCost(JitCost(250));
pub const SGROUP_NEGATE_COST: FixedCost = FixedCost(JitCost(45));

// --- SGlobal method costs ---
pub const SGLOBAL_GROUP_GENERATOR_COST: FixedCost = FixedCost(JitCost(10));
pub const SGLOBAL_XOR_COST: PerItemCost = PerItemCost::new(10, 2, 128);
// Serialize now uses instrumented SigmaByteWriter with per-operation costs (see sigma_byte_writer.rs)
#[allow(dead_code)]
pub const SGLOBAL_SERIALIZE_COST: PerItemCost = PerItemCost::new(10, 1, 1);
pub const SGLOBAL_DESERIALIZE_COST: PerItemCost = PerItemCost::new(100, 32, 32);
pub const SGLOBAL_SOME_COST: FixedCost = FixedCost(JitCost(5));
pub const SGLOBAL_NONE_COST: FixedCost = FixedCost(JitCost(5));
pub const SGLOBAL_FROM_BIGENDIAN_BYTES_COST: FixedCost = FixedCost(JitCost(10));
pub const SGLOBAL_ENCODE_NBITS_COST: FixedCost = FixedCost(JitCost(25));
pub const SGLOBAL_DECODE_NBITS_COST: FixedCost = FixedCost(JitCost(50));
// PowHit uses DynamicCost in Scala: 500 + CalcBlake2b256.cost(totalLen) * (k+1)
pub const SGLOBAL_POW_HIT_BASE_COST: u32 = 500;

// --- SColl method costs ---
pub const SCOLL_INDEX_OF_COST: PerItemCost = PerItemCost::new(20, 10, 2);
pub const SCOLL_FLATMAP_COST: PerItemCost = PerItemCost::new(60, 10, 8);
pub const SCOLL_ZIP_COST: PerItemCost = PerItemCost::new(10, 1, 10);
pub const SCOLL_INDICES_COST: PerItemCost = PerItemCost::new(20, 2, 16);
pub const SCOLL_PATCH_COST: PerItemCost = PerItemCost::new(30, 2, 10);
pub const SCOLL_UPDATED_COST: PerItemCost = PerItemCost::new(20, 1, 10);
pub const SCOLL_UPDATE_MANY_COST: PerItemCost = PerItemCost::new(20, 2, 10);
pub const SCOLL_REVERSE_COST: PerItemCost = PerItemCost::new(20, 2, 100);
pub const SCOLL_STARTS_WITH_COST: PerItemCost = PerItemCost::new(10, 1, 10);
pub const SCOLL_ENDS_WITH_COST: PerItemCost = PerItemCost::new(10, 1, 10);
pub const SCOLL_GET_COST: FixedCost = FixedCost(JitCost(30));

// --- SOption method costs ---
pub const SOPTION_MAP_COST: FixedCost = FixedCost(JitCost(20));
pub const SOPTION_FILTER_COST: FixedCost = FixedCost(JitCost(20));

// --- SNumeric method costs ---
pub const SNUMERIC_TO_BYTES_COST: FixedCost = FixedCost(JitCost(5));
pub const SNUMERIC_TO_BITS_COST: FixedCost = FixedCost(JitCost(5));
pub const SNUMERIC_BITWISE_INVERSE_COST: FixedCost = FixedCost(JitCost(5));
pub const SNUMERIC_BITWISE_OP_COST: FixedCost = FixedCost(JitCost(5));
pub const SNUMERIC_SHIFT_COST: FixedCost = FixedCost(JitCost(5));
#[allow(dead_code)]
pub const SNUMERIC_BIG_ENDIAN_BYTES_COST: FixedCost = FixedCost(JitCost(10));
pub const SNUMERIC_TO_SIGNED_COST: FixedCost = FixedCost(JitCost(10));
pub const SNUMERIC_TO_UNSIGNED_COST: FixedCost = FixedCost(JitCost(5));
pub const SNUMERIC_TO_UNSIGNED_MOD_COST: FixedCost = FixedCost(JitCost(15));
pub const SNUMERIC_MOD_INVERSE_COST: FixedCost = FixedCost(JitCost(150));
pub const SNUMERIC_PLUS_MOD_COST: FixedCost = FixedCost(JitCost(30));
pub const SNUMERIC_SUBTRACT_MOD_COST: FixedCost = FixedCost(JitCost(30));
pub const SNUMERIC_MULTIPLY_MOD_COST: FixedCost = FixedCost(JitCost(40));
pub const SNUMERIC_MOD_COST: FixedCost = FixedCost(JitCost(20));

// --- SAvlTree method costs ---
pub const SAVL_PROP_COST: FixedCost = FixedCost(JitCost(15));
pub const SAVL_UPDATE_OPERATIONS_COST: FixedCost = FixedCost(JitCost(45));
pub const SAVL_UPDATE_DIGEST_COST: FixedCost = FixedCost(JitCost(40));
// DynamicCost sub-operations (from Scala DataValueComparer / SAvlTreeMethods):
pub const CREATE_AVL_VERIFIER_COST: PerItemCost = PerItemCost::new(110, 20, 64);
pub const SAVL_GET_COST: PerItemCost = PerItemCost::new(40, 10, 1);
pub const SAVL_CONTAINS_COST: PerItemCost = PerItemCost::new(40, 10, 1);
pub const SAVL_GET_MANY_COST: PerItemCost = PerItemCost::new(40, 10, 1);
pub const SAVL_INSERT_COST: PerItemCost = PerItemCost::new(40, 10, 1);
pub const SAVL_INSERT_OR_UPDATE_COST: PerItemCost = PerItemCost::new(120, 20, 1);
pub const SAVL_UPDATE_COST: PerItemCost = PerItemCost::new(120, 20, 1);
pub const SAVL_REMOVE_COST: PerItemCost = PerItemCost::new(100, 15, 1);

// =============================================================================
// Protocol-level cost limit
// =============================================================================

/// Maximum block cost in the Ergo protocol (standard value: 1,000,000)
#[allow(dead_code)]
pub const MAX_BLOCK_COST: u64 = 1_000_000;
/// JIT cost limit (10x scale): used to cap evaluation costs
#[allow(dead_code)]
pub const MAX_BLOCK_COST_JIT: u64 = MAX_BLOCK_COST * 10;
