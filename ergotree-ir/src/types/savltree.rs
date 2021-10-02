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
/// AvlTree.enabledOperations property
pub const ENABLED_OPERATIONS_METHOD_ID: MethodId = MethodId(2);
/// AvlTree.keyLength property
pub const KEY_LENGTH_METHOD_ID: MethodId = MethodId(3);
/// AvlTree.valueLengthOpt property
pub const VALUE_LENGTH_OPT_METHOD_ID: MethodId = MethodId(4);
/// AvlTree.isInsertAllowed property
pub const IS_INSERT_ALLOWED_METHOD_ID: MethodId = MethodId(5);
/// AvlTree.isUpdateAllowed property
pub const IS_UPDATE_ALLOWED_METHOD_ID: MethodId = MethodId(6);
/// AvlTree.isRemoveAllowed property
pub const IS_REMOVE_ALLOWED_METHOD_ID: MethodId = MethodId(7);
/// AvlTree.insert property
pub const INSERT_METHOD_ID: MethodId = MethodId(12);

lazy_static! {
    /// AvlTree method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> =
        vec![
            &DIGEST_METHOD_DESC,
            &INSERT_METHOD_DESC,
            &ENABLED_OPERATIONS_METHOD_DESC,
            &KEY_LENGTH_METHOD_DESC,
            &VALUE_LENGTH_OPT_METHOD_DESC,
            &IS_INSERT_ALLOWED_METHOD_DESC,
            &IS_UPDATE_ALLOWED_METHOD_DESC,
            &IS_REMOVE_ALLOWED_METHOD_DESC,
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
    static ref ENABLED_OPERATIONS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: ENABLED_OPERATIONS_METHOD_ID,
        name: "enabledOperations",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree],
            t_range: SType::SByte.into(),
            tpe_params: vec![],
        },
    };
    /// AvlTree.enabledOperations
    pub static ref ENABLED_OPERATIONS_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, ENABLED_OPERATIONS_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref KEY_LENGTH_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: KEY_LENGTH_METHOD_ID,
        name: "keyLength",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree],
            t_range: SType::SInt.into(),
            tpe_params: vec![],
        },
    };
    /// AvlTree.keyLength
    pub static ref KEY_LENGTH_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, KEY_LENGTH_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref VALUE_LENGTH_OPT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: VALUE_LENGTH_OPT_METHOD_ID,
        name: "valueLengthOpt",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree],
            t_range: SType::SOption(Box::new(SType::SInt)).into(),
            tpe_params: vec![],
        },
    };
    /// AvlTree.valueLengthOpt
    pub static ref VALUE_LENGTH_OPT_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, VALUE_LENGTH_OPT_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref IS_INSERT_ALLOWED_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: IS_INSERT_ALLOWED_METHOD_ID,
        name: "isInsertAllowed",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree],
            t_range: SType::SBoolean.into(),
            tpe_params: vec![],
        },
    };
    /// AvlTree.isInsertAllowed
    pub static ref IS_INSERT_ALLOWED_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, IS_INSERT_ALLOWED_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref IS_UPDATE_ALLOWED_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: IS_UPDATE_ALLOWED_METHOD_ID,
        name: "isUpdateAllowed",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree],
            t_range: SType::SBoolean.into(),
            tpe_params: vec![],
        },
    };
    /// AvlTree.isUpdateAllowed
    pub static ref IS_UPDATE_ALLOWED_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, IS_UPDATE_ALLOWED_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref IS_REMOVE_ALLOWED_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: IS_REMOVE_ALLOWED_METHOD_ID,
        name: "isRemoveAllowed",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree],
            t_range: SType::SBoolean.into(),
            tpe_params: vec![],
        },
    };
    /// AvlTree.isRemoveAllowed
    pub static ref IS_REMOVE_ALLOWED_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, IS_REMOVE_ALLOWED_METHOD_DESC.clone(),);
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
