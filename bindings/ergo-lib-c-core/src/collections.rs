//! Collections that can be manipulated from outside Rust

use crate::error::*;
use crate::util::{const_ptr_as_ref, mut_ptr_as_mut};

pub struct Collection<T>(Vec<T>);
pub type CollectionPtr<T> = *mut Collection<T>;
pub type ConstCollectionPtr<T> = *const Collection<T>;

pub unsafe fn collection_new<T>(collection_out: *mut CollectionPtr<T>) -> Result<(), Error> {
    let collection_out = mut_ptr_as_mut(collection_out, "collection_out")?;
    *collection_out = Box::into_raw(Box::new(Collection(vec![])));
    Ok(())
}

pub fn collection_delete<T>(collection: CollectionPtr<T>) {
    if !collection.is_null() {
        let boxed = unsafe { Box::from_raw(collection) };
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
) -> Result<Option<T>, Error> {
    let collection = const_ptr_as_ref(collection, "collection")?;
    Ok(collection.0.get(index).cloned())
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
