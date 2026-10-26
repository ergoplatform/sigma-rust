use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use crate::ergo_tree::ErgoTreeVersion;
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
/// AvlTree.updateOperations property
pub const UPDATE_OPERATIONS_METHOD_ID: MethodId = MethodId(8);
/// AvlTree.contains property
pub const CONTAINS_METHOD_ID: MethodId = MethodId(9);
/// AvlTree.get property
pub const GET_METHOD_ID: MethodId = MethodId(10);
/// AvlTree.getMany property
pub const GET_MANY_METHOD_ID: MethodId = MethodId(11);
/// AvlTree.insert property
pub const INSERT_METHOD_ID: MethodId = MethodId(12);
/// AvlTree.remove property
pub const REMOVE_METHOD_ID: MethodId = MethodId(14);
/// AvlTree.update property
pub const UPDATE_METHOD_ID: MethodId = MethodId(13);
/// AvlTree.updateDigest property
pub const UPDATE_DIGEST_METHOD_ID: MethodId = MethodId(15);
/// AvlTree.insertOrUpdate property
pub const INSERT_OR_UPDATE_METHOD_ID: MethodId = MethodId(16);

lazy_static! {
    /// AvlTree method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> =
        vec![
            DIGEST_METHOD_DESC.clone(),
            ENABLED_OPERATIONS_METHOD_DESC.clone(),
            KEY_LENGTH_METHOD_DESC.clone(),
            VALUE_LENGTH_OPT_METHOD_DESC.clone(),
            IS_INSERT_ALLOWED_METHOD_DESC.clone(),
            IS_UPDATE_ALLOWED_METHOD_DESC.clone(),
            IS_REMOVE_ALLOWED_METHOD_DESC.clone(),
            UPDATE_OPERATIONS_METHOD_DESC.clone(),
            CONTAINS_METHOD_DESC.clone(),
            GET_METHOD_DESC.clone(),
            GET_MANY_METHOD_DESC.clone(),
            INSERT_METHOD_DESC.clone(),
            REMOVE_METHOD_DESC.clone(),
            UPDATE_METHOD_DESC.clone(),
            UPDATE_DIGEST_METHOD_DESC.clone(),
            INSERT_OR_UPDATE_METHOD_DESC.clone(),
        ]
    ;
}

lazy_static! {
    static ref DIGEST_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: DIGEST_METHOD_ID,
        name: "digest",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree],
            t_range: SType::SColl(Arc::new(SType::SByte)).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
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
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
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
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
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
            t_range: SType::SOption(Arc::new(SType::SInt)).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
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
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
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
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
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
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
    /// AvlTree.isRemoveAllowed
    pub static ref IS_REMOVE_ALLOWED_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, IS_REMOVE_ALLOWED_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref UPDATE_OPERATIONS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: UPDATE_OPERATIONS_METHOD_ID,
        name: "updateOperations",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree, SType::SByte],
            t_range: SType::SAvlTree.into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
    /// AvlTree.updateOperations
    pub static ref UPDATE_OPERATIONS_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, UPDATE_OPERATIONS_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref GET_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GET_METHOD_ID,
        name: "get",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree,
                         SType::SColl(SType::SByte.into()),
                         SType::SColl(SType::SByte.into()),
                       ],
            t_range: SType::SOption(SType::SColl(SType::SByte.into()).into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };

    /// AvlTree.get
    pub static ref GET_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, GET_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref GET_MANY_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GET_MANY_METHOD_ID,
        name: "getMany",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree,
                         SType::SColl(SType::SColl(SType::SByte.into()).into()),
                         SType::SColl(SType::SByte.into()),
                       ],
            t_range: SType::SColl(SType::SOption(SType::SColl(SType::SByte.into()).into()).into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };

    /// AvlTree.getMany
    pub static ref GET_MANY_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, GET_MANY_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref INSERT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: INSERT_METHOD_ID,
        name: "insert",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree,
                         SType::SColl(
                           Arc::new(
                               SType::STuple(
                                   STuple::pair(
                                       SType::SColl(Arc::new(SType::SByte)),
                                       SType::SColl(Arc::new(SType::SByte))
                                   )
                               )
                           )
                         ),
                         SType::SColl(Arc::new(SType::SByte)),
                       ],
            t_range: SType::SOption(Arc::new(SType::SAvlTree)).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
    /// AvlTree.insert
    pub static ref INSERT_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, INSERT_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref REMOVE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: REMOVE_METHOD_ID,
        name: "remove",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree,
                         SType::SColl(
                            Arc::new(
                                SType::SColl(Arc::new(SType::SByte))
                            )
                         ),
                         SType::SColl(Arc::new(SType::SByte)),
                       ],
            t_range: SType::SOption(Arc::new(SType::SAvlTree)).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
    /// AvlTree.remove
    pub static ref REMOVE_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, REMOVE_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref CONTAINS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: CONTAINS_METHOD_ID,
        name: "contains",
        tpe: SFunc {
            t_dom: vec![
                SType::SAvlTree,
                SType::SColl(Arc::new(SType::SByte)),
                SType::SColl(Arc::new(SType::SByte)),
            ],
            t_range: SType::SBoolean.into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
    /// AvlTree.contains
    pub static ref CONTAINS_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, CONTAINS_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref UPDATE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: UPDATE_METHOD_ID,
        name: "update",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree,
                         SType::SColl(
                            Arc::new(
                                SType::STuple(
                                    STuple::pair(
                                        SType::SColl(Arc::new(SType::SByte)),
                                        SType::SColl(Arc::new(SType::SByte))
                                    )
                                )
                            )
                         ),
                         SType::SColl(Arc::new(SType::SByte)),
                       ],
            t_range: SType::SOption(Arc::new(SType::SAvlTree)).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
    /// AvlTree.update
    pub static ref UPDATE_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, UPDATE_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref UPDATE_DIGEST_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: UPDATE_DIGEST_METHOD_ID,
        name: "updateDigest",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree, SType::SColl(Arc::new(SType::SByte))],
            t_range: SType::SAvlTree.into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
    /// AvlTree.updateDigest
    pub static ref UPDATE_DIGEST_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, UPDATE_DIGEST_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref INSERT_OR_UPDATE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: INSERT_OR_UPDATE_METHOD_ID,
        name: "insertOrUpdate",
        tpe: SFunc {
            t_dom: vec![ SType::SAvlTree,
                         SType::SColl(
                           Arc::new(
                               SType::STuple(
                                   STuple::pair(
                                       SType::SColl(Arc::new(SType::SByte)),
                                       SType::SColl(Arc::new(SType::SByte))
                                   )
                               )
                           )
                         ),
                         SType::SColl(Arc::new(SType::SByte)),
                       ],
            t_range: SType::SOption(Arc::new(SType::SAvlTree)).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    /// AvlTree.insertOrUpdate
    pub static ref INSERT_OR_UPDATE_METHOD: SMethod =
        SMethod::new(STypeCompanion::AvlTree, INSERT_OR_UPDATE_METHOD_DESC.clone(),);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_CODE, DIGEST_METHOD_ID).map(|e| e.name()) == Ok("digest"));
        assert!(
            SMethod::from_ids(TYPE_CODE, ENABLED_OPERATIONS_METHOD_ID).map(|e| e.name())
                == Ok("enabledOperations")
        );
        assert!(
            SMethod::from_ids(TYPE_CODE, KEY_LENGTH_METHOD_ID).map(|e| e.name()) == Ok("keyLength")
        );
        assert!(
            SMethod::from_ids(TYPE_CODE, VALUE_LENGTH_OPT_METHOD_ID).map(|e| e.name())
                == Ok("valueLengthOpt")
        );
        assert!(
            SMethod::from_ids(TYPE_CODE, IS_INSERT_ALLOWED_METHOD_ID).map(|e| e.name())
                == Ok("isInsertAllowed")
        );
        assert!(
            SMethod::from_ids(TYPE_CODE, IS_UPDATE_ALLOWED_METHOD_ID).map(|e| e.name())
                == Ok("isUpdateAllowed")
        );
        assert!(
            SMethod::from_ids(TYPE_CODE, IS_REMOVE_ALLOWED_METHOD_ID).map(|e| e.name())
                == Ok("isRemoveAllowed")
        );
        assert!(
            SMethod::from_ids(TYPE_CODE, UPDATE_OPERATIONS_METHOD_ID).map(|e| e.name())
                == Ok("updateOperations")
        );
        assert!(SMethod::from_ids(TYPE_CODE, GET_METHOD_ID).map(|e| e.name()) == Ok("get"));
        assert!(
            SMethod::from_ids(TYPE_CODE, GET_MANY_METHOD_ID).map(|e| e.name()) == Ok("getMany")
        );
        assert!(SMethod::from_ids(TYPE_CODE, INSERT_METHOD_ID).map(|e| e.name()) == Ok("insert"));
        assert!(SMethod::from_ids(TYPE_CODE, REMOVE_METHOD_ID).map(|e| e.name()) == Ok("remove"));
        assert!(SMethod::from_ids(TYPE_CODE, UPDATE_METHOD_ID).map(|e| e.name()) == Ok("update"));
        assert!(
            SMethod::from_ids(TYPE_CODE, UPDATE_DIGEST_METHOD_ID).map(|e| e.name())
                == Ok("updateDigest")
        );
    }
}
