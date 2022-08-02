//! Collections that can be manipulated from outside Rust

use std::convert::{TryFrom, TryInto};

use bounded_vec::{BoundedVec, BoundedVecOutOfBounds, OptBoundedVecToVec};

use crate::error::*;
use crate::util::{const_ptr_as_ref, mut_ptr_as_mut};

pub struct Collection<T>(pub Vec<T>);
pub type CollectionPtr<T> = *mut Collection<T>;
pub type ConstCollectionPtr<T> = *const Collection<T>;

pub unsafe fn collection_new<T>(collection_out: *mut CollectionPtr<T>) -> Result<(), Error> {
    let collection_out = mut_ptr_as_mut(collection_out, "collection_out")?;
    *collection_out = Box::into_raw(Box::new(Collection(vec![])));
    Ok(())
}

pub unsafe fn collection_delete<T>(collection: CollectionPtr<T>) {
    if !collection.is_null() {
        let boxed = Box::from_raw(collection);
        std::mem::drop(boxed);
    }
}

pub unsafe fn collection_len<T>(collection: ConstCollectionPtr<T>) -> Result<usize, Error> {
    let collection = const_ptr_as_ref(collection, "collection")?;
    Ok(collection.0.len())
}

pub unsafe fn collection_get<T: Clone>(
    collection: ConstCollectionPtr<T>,
    index: usize,
    elem_out: *mut *mut T,
) -> Result<bool, Error> {
    let collection = const_ptr_as_ref(collection, "collection")?;
    let elem_out = mut_ptr_as_mut(elem_out, "elem_out")?;
    if let Some(elem) = collection.0.get(index) {
        *elem_out = Box::into_raw(Box::new(elem.clone()));
        return Ok(true);
    }
    Ok(false)
}

pub unsafe fn collection_add<T: Clone>(
    collection_out: CollectionPtr<T>,
    elem: *const T,
) -> Result<(), Error> {
    let collection_out = mut_ptr_as_mut(collection_out, "collection_out")?;
    let elem = const_ptr_as_ref(elem, "elem")?;
    collection_out.0.push(elem.clone());
    Ok(())
}

impl<T, S: Into<T>> From<Vec<S>> for Collection<T> {
    fn from(vec: Vec<S>) -> Self {
        Collection(vec.into_iter().map(Into::into).collect())
    }
}

impl<T, S: Into<T>> From<Collection<S>> for Vec<T> {
    fn from(vec: Collection<S>) -> Self {
        vec.0.into_iter().map(Into::into).collect()
    }
}

impl<T, S: Into<T> + Clone> From<&Collection<S>> for Vec<T> {
    fn from(vec: &Collection<S>) -> Self {
        vec.0.clone().into_iter().map(Into::into).collect()
    }
}

impl<T, S: Into<T>, const L: usize, const U: usize> From<BoundedVec<S, L, U>> for Collection<T> {
    fn from(bvec: BoundedVec<S, L, U>) -> Self {
        bvec.to_vec().into()
    }
}

impl<T, S: Into<T>, const L: usize, const U: usize> From<Option<BoundedVec<S, L, U>>>
    for Collection<T>
{
    fn from(maybe_bvec: Option<BoundedVec<S, L, U>>) -> Self {
        maybe_bvec.to_vec().into()
    }
}

impl<T, S: Into<T> + Clone, const L: usize, const U: usize> TryFrom<Collection<S>>
    for Option<BoundedVec<T, L, U>>
{
    type Error = BoundedVecOutOfBounds;

    fn try_from(tokens: Collection<S>) -> Result<Self, Self::Error> {
        (&tokens).try_into()
    }
}

impl<T, S: Into<T> + Clone, const L: usize, const U: usize> TryFrom<&Collection<S>>
    for Option<BoundedVec<T, L, U>>
{
    type Error = BoundedVecOutOfBounds;

    fn try_from(tokens: &Collection<S>) -> Result<Self, Self::Error> {
        if tokens.0.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                tokens
                    .0
                    .clone()
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<T>>()
                    .try_into()?,
            ))
        }
    }
}
