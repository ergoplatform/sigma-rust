use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stuple::STuple;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use lazy_static::lazy_static;

/// SAvlTree type code
pub const TYPE_CODE: TypeCode = TypeCode::SAVL_TREE;
/// SAvlTree type name
pub static TYPE_NAME: &str = "AvlTree";
/// AvlTree.digest property
pub const DIGEST_METHOD_ID: MethodId = MethodId(1);
/// AvlTree.insert property
pub const INSERT_METHOD_ID: MethodId = MethodId(12);

lazy_static! {
    /// AvlTree method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> =
        vec![
            &DIGEST_METHOD_DESC,
            &INSERT_METHOD_DESC,
        ]
    ;
}

lazy_static! {
    static ref DIGEST_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: DIGEST_METHOD_ID,
        name: "digest",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree],
            t_range: SType::SColl(Box::new(SType::SByte)).into(),
            tpe_params: vec![],
        },
    };
    /// AvlTree.digest
    pub static ref DIGEST_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, DIGEST_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref INSERT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: INSERT_METHOD_ID,
        name: "insert",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree,
                         SType::SColl(
                           Box::new(
                               SType::STuple(
                                   STuple::pair(
                                       SType::SColl(Box::new(SType::SByte)),
                                       SType::SColl(Box::new(SType::SByte))
                                   )
                               )
                           )
                         ),
                         SType::SColl(Box::new(SType::SByte)),
                       ],
            t_range: SType::SOption(Box::new(SType::SAvlTree)).into(),
            tpe_params: vec![],
        },
    };
    /// AvlTree.insert
    pub static ref INSERT_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, INSERT_METHOD_DESC.clone(),);
}
