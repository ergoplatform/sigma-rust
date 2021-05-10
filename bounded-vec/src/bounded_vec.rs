use std::convert::TryFrom;
use std::slice::Iter;

/// Non-empty Vec bounded with minimal (L - lower bound) and maximal (U - upper bound) items quantity
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoundedVec<T, const L: usize, const U: usize>
// enable when feature(const_evaluatable_checked) is stable
// where
//     Assert<{ L > 0 }>: IsTrue,
{
    inner: Vec<T>,
}

// enum Assert<const COND: bool> {}

// trait IsTrue {}

// impl IsTrue for Assert<true> {}

/// BoundedVec errors
#[derive(Debug)]
pub enum BoundedVecOutOfBounds {
    /// Items quantity is less than L (lower bound)
    LowerBoundError {
        /// L (lower bound)
        lower_bound: usize,
        /// provided value
        got: usize,
    },
    /// Items quantity is more than U (upper bound)
    UpperBoundError {
        /// U (upper bound)
        upper_bound: usize,
        /// provided value
        got: usize,
    },
}

impl<T, const L: usize, const U: usize> BoundedVec<T, L, U> {
    /// Creates new BoundedVec or returns error if items count is out of bounds
    ///
    /// # Example
    /// ```
    /// use bounded_vec::BoundedVec;
    /// let data: BoundedVec<_, 2, 8> = BoundedVec::from_vec(vec![1u8, 2]).unwrap();
    /// ```
    pub fn from_vec(items: Vec<T>) -> Result<Self, BoundedVecOutOfBounds> {
        // remove when feature(const_evaluatable_checked) is stable
        // and this requirement is encoded in type sig
        assert!(L > 0);
        let len = items.len();
        if len < L {
            Err(BoundedVecOutOfBounds::LowerBoundError {
                lower_bound: L,
                got: len,
            })
        } else if len > U {
            Err(BoundedVecOutOfBounds::UpperBoundError {
                upper_bound: U,
                got: len,
            })
        } else {
            Ok(BoundedVec { inner: items })
        }
    }

    /// Returns a reference to underlying `Vec``
    ///
    /// # Example
    /// ```
    /// use bounded_vec::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.as_vec(), &vec![1u8,2]);
    /// ```
    pub fn as_vec(&self) -> &Vec<T> {
        &self.inner
    }

    /// Returns the number of elements in the vector
    ///
    /// # Example
    /// ```
    /// use bounded_vec::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<u8, 2, 4> = vec![1u8,2].try_into().unwrap();
    /// assert_eq!(data.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Always returns `false` (cannot be empty)
    ///
    /// # Example
    /// ```
    /// use bounded_vec::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// # Example
    /// ```
    /// use bounded_vec::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.as_slice(), &[1u8,2]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    /// Returns the first element of non-empty Vec
    ///
    /// # Example
    /// ```
    /// use bounded_vec::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(*data.first(), 1);
    /// ```
    pub fn first(&self) -> &T {
        #![allow(clippy::unwrap_used)]
        self.inner.first().unwrap()
    }

    /// Returns the last element of non-empty Vec
    ///
    /// # Example
    /// ```
    /// use bounded_vec::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(*data.last(), 2);
    /// ```
    pub fn last(&self) -> &T {
        #![allow(clippy::unwrap_used)]
        self.inner.last().unwrap()
    }

    /// Create a new `BoundedVec` by consuming `self` and mapping each element.
    ///
    /// This is useful as it keeps the knowledge that the length is >= U, <= L,
    /// even through the old `BoundedVec` is consumed and turned into an iterator.
    ///
    /// # Example
    ///
    /// ```
    /// use bounded_vec::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let data = data.mapped(|x|x*2);
    /// assert_eq!(data, [2u8,4].into());
    /// ```
    pub fn mapped<F, N>(self, map_fn: F) -> BoundedVec<N, L, U>
    where
        F: FnMut(T) -> N,
    {
        BoundedVec {
            inner: self.inner.into_iter().map(map_fn).collect::<Vec<_>>(),
        }
    }

    /// Create a new `BoundedVec` by mapping references to the elements of self
    ///
    /// This is useful as it keeps the knowledge that the length is >= U, <= L,
    /// will still hold for new `BoundedVec`
    ///
    /// # Example
    ///
    /// ```
    /// use bounded_vec::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let data = data.mapped_ref(|x|x*2);
    /// assert_eq!(data, [2u8,4].into());
    /// ```
    pub fn mapped_ref<F, N>(&self, map_fn: F) -> BoundedVec<N, L, U>
    where
        F: FnMut(&T) -> N,
    {
        BoundedVec {
            inner: self.inner.iter().map(map_fn).collect::<Vec<_>>(),
        }
    }

    /// Returns a reference for an element at index or `None` if out of bounds
    ///
    /// # Example
    ///
    /// ```
    /// use bounded_vec::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let elem = *data.get(1).unwrap();
    /// assert_eq!(elem, 2);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    /// Get an iterator
    pub fn iter(&self) -> Iter<T> {
        self.inner.iter()
    }
}

impl<T, const L: usize, const U: usize> TryFrom<Vec<T>> for BoundedVec<T, L, U> {
    type Error = BoundedVecOutOfBounds;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        BoundedVec::from_vec(value)
    }
}

// when feature(const_evaluatable_checked) is stable cover all array sizes (L..=U)
impl<T, const L: usize, const U: usize> From<[T; L]> for BoundedVec<T, L, U> {
    fn from(arr: [T; L]) -> Self {
        BoundedVec { inner: arr.into() }
    }
}

impl<T, const L: usize, const U: usize> From<BoundedVec<T, L, U>> for Vec<T> {
    fn from(v: BoundedVec<T, L, U>) -> Self {
        v.inner
    }
}
