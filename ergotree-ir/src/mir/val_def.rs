use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::traversable::impl_traversable_expr;
use crate::types::stype::SType;
use crate::types::stype_param::STypeVar;

use super::expr::Expr;

extern crate derive_more;
use alloc::boxed::Box;
use alloc::vec::Vec;
use derive_more::Display;
use derive_more::From;

use crate::has_opcode::HasOpCode;
#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;

/// Variable id
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, From, Display, Ord, PartialOrd)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub struct ValId(pub u32);

impl ValId {
    pub(crate) fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> core3::io::Result<()> {
        w.put_u32(self.0)
    }

    pub(crate) fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let id = r.get_u32()?;
        Ok(ValId(id))
    }
}

/** IR node for let-bound expressions `let x = rhs` which is ValDef.
 * These nodes are used to represent ErgoTrees after common sub-expression elimination.
 * This representation is more compact in serialized form.
 * @param id unique identifier of the variable in the current scope. */
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub struct ValDef {
    /// Variable id
    pub id: ValId,
    /// Type parameters — non-empty for the polymorphic `FunDef` form
    /// (`val id[T] = ...`, opcode 0xd7), empty for a plain `ValDef` (0xd6).
    /// The JVM models both as one node (`ValDef(id, tpeArgs, rhs)`) with the
    /// opcode chosen by `tpeArgs.isEmpty` — mirrored here.
    pub tpe_args: Vec<STypeVar>,
    /// Expr, bound to the variable
    pub rhs: Box<Expr>,
}

impl ValDef {
    /// Type
    pub fn tpe(&self) -> SType {
        self.rhs.tpe()
    }

    /// Parse the `FunDef`-opcoded form (0xd7): type parameters precede the
    /// rhs (JVM `ValDefSerializer` with `FunDefCode`).
    pub(crate) fn sigma_parse_fun_def<R: SigmaByteRead>(
        r: &mut R,
    ) -> Result<Self, SigmaParsingError> {
        let id = ValId::sigma_parse(r)?;
        // The JVM reads the count as a SIGNED byte (`r.getByte()`) and fails
        // on a negative size when allocating the array — mirror by rejecting
        // counts above 127 (bytes 0x80..=0xff).
        let n_tpe_args = r.get_i8()?;
        if n_tpe_args < 0 {
            return Err(SigmaParsingError::Misc(format!(
                "FunDef: negative type parameter count {}",
                n_tpe_args
            )));
        }
        // Type parameters are written with the full type encoding
        // (JVM `w.putType(arg)` / `r.getType().asInstanceOf[STypeVar]`),
        // so each must parse as an `SType` that IS a type variable.
        let tpe_args = (0..n_tpe_args)
            .map(|_| match SType::sigma_parse(r)? {
                SType::STypeVar(tv) => Ok(tv),
                other => Err(SigmaParsingError::Misc(format!(
                    "FunDef: expected a type variable as type parameter, got {:?}",
                    other
                ))),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let rhs = Expr::sigma_parse(r)?;
        r.val_def_type_store().insert(id, rhs.tpe());
        Ok(ValDef {
            id,
            tpe_args,
            rhs: Box::new(rhs),
        })
    }
}

impl HasOpCode for ValDef {
    /// `FunDef` (0xd7) when type parameters are present, plain `ValDef`
    /// (0xd6) otherwise — the JVM serializes both shapes from the same node
    /// (`ValDefSerializer`, registered under both opcodes).
    fn op_code(&self) -> OpCode {
        if self.tpe_args.is_empty() {
            OpCode::VAL_DEF
        } else {
            OpCode::FUN_DEF
        }
    }
}

impl SigmaSerializable for ValDef {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.id.sigma_serialize(w)?;
        if !self.tpe_args.is_empty() {
            // FunDef form: type parameters precede the rhs (JVM
            // `ValDefSerializer` "type arguments" section; the count is a
            // single byte and each parameter uses the full type encoding —
            // `w.putType(arg)`).
            w.put_u8(self.tpe_args.len() as u8)?;
            for tpe_arg in &self.tpe_args {
                SType::STypeVar(tpe_arg.clone()).sigma_serialize(w)?;
            }
        }
        self.rhs.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let id = ValId::sigma_parse(r)?;
        let rhs = Expr::sigma_parse(r)?;
        r.val_def_type_store().insert(id, rhs.tpe());
        Ok(ValDef {
            id,
            tpe_args: Vec::new(),
            rhs: Box::new(rhs),
        })
    }
}

impl_traversable_expr!(ValDef, boxed rhs);

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ValDef>()) {
            let e = Expr::ValDef(v.into());
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
