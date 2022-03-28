//! ErgoTree

use std::convert::TryFrom;

use ergo_lib::{
    ergo_chain_types::Base16DecodedBytes, ergotree_ir::serialization::SigmaSerializable,
};

use crate::{
    constant::{ConstConstantPtr, Constant, ConstantPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoTree(pub(crate) ergo_lib::ergotree_ir::ergo_tree::ErgoTree);
pub type ErgoTreePtr = *mut ErgoTree;
pub type ConstErgoTreePtr = *const ErgoTree;

/// Decode from base16 encoded serialized ErgoTree
pub unsafe fn ergo_tree_from_base16_bytes(
    bytes_str: &str,
    ergo_tree_out: *mut ErgoTreePtr,
) -> Result<(), Error> {
    let ergo_tree_out = mut_ptr_as_mut(ergo_tree_out, "ergo_tree_out")?;
    let bytes = Base16DecodedBytes::try_from(bytes_str.to_string())?.0;
    let ergo_tree =
        ergo_lib::ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(&bytes).map(ErgoTree)?;
    *ergo_tree_out = Box::into_raw(Box::new(ergo_tree));
    Ok(())
}

/// Decode from encoded serialized ErgoTree
pub unsafe fn ergo_tree_from_bytes(
    bytes_ptr: *const u8,
    len: usize,
    ergo_tree_out: *mut ErgoTreePtr,
) -> Result<(), Error> {
    if bytes_ptr.is_null() {
        return Err(Error::Misc("bytes_ptr is null".into()));
    }
    let bytes = std::slice::from_raw_parts(bytes_ptr, len);
    let ergo_tree_out = mut_ptr_as_mut(ergo_tree_out, "ergo_tree_out")?;
    let ergo_tree =
        ergo_lib::ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(bytes).map(ErgoTree)?;
    *ergo_tree_out = Box::into_raw(Box::new(ergo_tree));
    Ok(())
}

/// Return length of the `&[u8]` serialized representation of `ErgoTree`.
pub unsafe fn ergo_tree_bytes_len(ergo_tree_ptr: ConstErgoTreePtr) -> Result<usize, Error> {
    let ergo_tree = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?;
    let len = ergo_tree.0.sigma_serialize_bytes().map(|v| v.len())?;
    Ok(len)
}

/// Convert to serialized bytes. Key assumption: enough memory has been allocated at the address
/// pointed-to by `output`. Use `ergo_tree_bytes_len` to determine the length of the byte array.
pub unsafe fn ergo_tree_to_bytes(
    ergo_tree_ptr: ConstErgoTreePtr,
    output: *mut u8,
) -> Result<(), Error> {
    let ergo_tree = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?;
    let src = ergo_tree.0.sigma_serialize_bytes()?;
    std::ptr::copy_nonoverlapping(src.as_ptr(), output, src.len());
    Ok(())
}

/// Convert to base16-encoded serialized bytes
pub unsafe fn ergo_tree_to_base16_bytes(ergo_tree_ptr: ConstErgoTreePtr) -> Result<String, Error> {
    let ergo_tree = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?;
    let s = ergo_tree.0.to_base16_bytes()?;
    Ok(s)
}

/// Returns constants number as stored in serialized ErgoTree or error if the parsing of constants
/// failed
pub unsafe fn ergo_tree_constants_len(ergo_tree_ptr: ConstErgoTreePtr) -> Result<usize, Error> {
    let ergo_tree = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?;
    let len = ergo_tree.0.constants_len()?;
    Ok(len)
}

/// If constant with given index (as stored in serialized ErgoTree) exists, allocate it and store in
/// `constant_out` and return `Ok(true)`. If constant doesn't exist at the given index return
/// Ok(false).  If parsing failed then return error.
pub unsafe fn ergo_tree_get_constant(
    ergo_tree_ptr: ConstErgoTreePtr,
    index: usize,
    constant_out: *mut ConstantPtr,
) -> Result<bool, Error> {
    let ergo_tree = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?;
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    let constant = ergo_tree.0.get_constant(index).map(|c| c.map(Constant))?;
    if let Some(constant) = constant {
        *constant_out = Box::into_raw(Box::new(constant));
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Returns new ErgoTree instance with a new constant value for a given index in constants list (as
/// stored in serialized ErgoTree), or an error. Note that the original ErgoTree instance
/// pointed-at by `ergo_tree_ptr` is untouched.
pub unsafe fn ergo_tree_with_constant(
    ergo_tree_ptr: ConstErgoTreePtr,
    index: usize,
    constant_ptr: ConstConstantPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) -> Result<(), Error> {
    let ergo_tree_cloned = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?.clone();
    let constant = const_ptr_as_ref(constant_ptr, "constant_ptr")?;
    let ergo_tree_out = mut_ptr_as_mut(ergo_tree_out, "ergo_tree_out")?;
    let new_inner = ergo_tree_cloned
        .0
        .with_constant(index, constant.0.clone())?;
    *ergo_tree_out = Box::into_raw(Box::new(ErgoTree(new_inner)));
    Ok(())
}

/// Return length of the `&[u8]` serialized representation of `ErgoTree` template.
pub unsafe fn ergo_tree_template_bytes_len(
    ergo_tree_ptr: ConstErgoTreePtr,
) -> Result<usize, Error> {
    let ergo_tree = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?;
    let len = ergo_tree.0.template_bytes().map(|v| v.len())?;
    Ok(len)
}

/// Serialized proposition expression of SigmaProp type with ConstantPlaceholder nodes instead of
/// Constant nodes. Key assumption: enough memory has been allocated at the address pointed-to by
/// `output`. Use `ergo_tree_template_bytes_len` to determine the length of the byte array.
pub unsafe fn ergo_tree_template_bytes(
    ergo_tree_ptr: ConstErgoTreePtr,
    output: *mut u8,
) -> Result<(), Error> {
    let ergo_tree = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?;
    let src = ergo_tree.0.template_bytes()?;
    std::ptr::copy_nonoverlapping(src.as_ptr(), output, src.len());
    Ok(())
}
