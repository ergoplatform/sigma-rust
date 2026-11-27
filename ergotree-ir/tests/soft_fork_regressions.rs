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

#[test]
fn unknown_primitive_codes_use_primitive_validation_rules() {
    use ergotree_ir::ergo_tree::ErgoTreeVersion;
    use ergotree_ir::soft_fork::{RuleStatus, SigmaValidationSettings};
    use ergotree_ir::types::stype::SType;

    // All compact containers, including pairs, must retain the primitive ID.
    for container in 0..8 {
        for code in [10u8, 11] {
            let tag = container * 12 + code;
            let error = SType::sigma_parse_bytes(&[tag]).unwrap_err();
            assert!(
                matches!(
                    error,
                    SigmaParsingError::SoftForkError(SoftForkError::InvalidPrimitiveType(id)) if id == code
                ),
                "tag {tag}: {error:?}"
            );
            for (version, primitive_rule, type_rule) in [
                (ErgoTreeVersion::V0, 1007, 1008),
                (ErgoTreeVersion::V3, 1017, 1018),
            ] {
                let settings = SigmaValidationSettings::default()
                    .updated(primitive_rule, RuleStatus::Changed(vec![code]));
                assert!(settings.is_soft_fork(&error, version));
                let wrong_code = SigmaValidationSettings::default()
                    .updated(primitive_rule, RuleStatus::Changed(vec![code + 1]));
                assert!(!wrong_code.is_soft_fork(&error, version));
                let wrong_rule = SigmaValidationSettings::default()
                    .updated(type_rule, RuleStatus::Changed(vec![code]));
                assert!(!wrong_rule.is_soft_fork(&error, version));
                let replaced = SigmaValidationSettings::default()
                    .updated(primitive_rule, RuleStatus::Replaced(primitive_rule + 100));
                assert!(replaced.is_soft_fork(&error, version));
            }
        }
    }
}
