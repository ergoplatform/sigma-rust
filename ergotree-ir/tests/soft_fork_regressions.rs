//! Wire regressions for the soft-fork review of #982.
#![allow(clippy::unwrap_used)]
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::mir::{
    constant::Constant, expr::Expr, method_call::MethodCall, property_call::PropertyCall,
};
use ergotree_ir::serialization::{
    constant_store::ConstantStore, sigma_byte_reader::SigmaByteReader,
    sigma_byte_writer::SigmaByteWriter, SigmaParsingError, SigmaSerializable,
};
use ergotree_ir::soft_fork::SoftForkError;
use ergotree_ir::types::{sglobal, sheader};
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use std::io::Cursor;

#[test]
fn sized_tree_rejects_hard_zero_prefix_and_oversized_constants() {
    assert!(ErgoTree::sigma_parse_bytes(&[0x18, 3, 1, 0, 0]).is_err());
    assert!(ErgoTree::sigma_parse_bytes(&[0x18, 3, 0x81, 0x20, 0]).is_err());
    assert!(matches!(
        ErgoTree::sigma_parse_bytes(&[0x0c, 1, 0xfb]).unwrap(),
        ErgoTree::Unparsed { .. }
    ));
}

#[test]
fn property_version_gate_runs_before_object_specialization() {
    let mut bytes = vec![];
    let mut writer = SigmaByteWriter::new(&mut bytes, None);
    sheader::TYPE_CODE.sigma_serialize(&mut writer).unwrap();
    writer.put_u8(sheader::CHECK_POW_METHOD_ID.0).unwrap();
    // Intentionally invalid receiver: the V0 reader must first reject the V3 method.
    Expr::Const(Constant::from(1i32))
        .sigma_serialize(&mut writer)
        .unwrap();
    let mut reader = SigmaByteReader::new(Cursor::new(bytes), ConstantStore::empty());
    assert!(matches!(
        PropertyCall::sigma_parse(&mut reader),
        Err(SigmaParsingError::SoftForkError(
            SoftForkError::UnknownMethodId(_, _)
        ))
    ));
}

#[test]
fn method_version_gate_runs_before_argument_specialization() {
    let mut bytes = vec![];
    let mut writer = SigmaByteWriter::new(&mut bytes, None);
    sglobal::TYPE_CODE.sigma_serialize(&mut writer).unwrap();
    writer.put_u8(sglobal::ENCODE_NBITS_METHOD_ID.0).unwrap();
    Expr::Global.sigma_serialize(&mut writer).unwrap();
    vec![Expr::Const(Constant::from(true))]
        .sigma_serialize(&mut writer)
        .unwrap();
    let mut reader = SigmaByteReader::new(Cursor::new(bytes), ConstantStore::empty());
    assert!(matches!(
        MethodCall::sigma_parse(&mut reader),
        Err(SigmaParsingError::SoftForkError(
            SoftForkError::UnknownMethodId(_, _)
        ))
    ));
}
