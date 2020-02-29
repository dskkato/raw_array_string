use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;
use std::str;
use std::str::Utf8Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::array::Array;
use crate::errors::CapacityError;
use crate::maybe_uninit::MaybeUninit as MaybeUninitCopy;

#[derive(Copy)]
pub struct RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    xs: MaybeUninitCopy<A>,
}

impl<A> Default for RawArrayString<A>
    where A: Array<Item=u8> + Copy
{
    /// Return an empty `RawArrayString`
    fn default() -> RawArrayString<A> {
        RawArrayString::new()
    }
}


impl<A> RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    /// Create a new empty `RawArrayString`.
    ///
    /// Capacity is inferred from the type parameter.
    ///
    /// ```
    /// use raw_array_string::RawArrayString;
    ///
    /// let mut string = RawArrayString::<[_; 16]>::new();
    /// string.push_str("foo");
    /// assert_eq!(&string[..], "foo");
    /// assert_eq!(string.capacity(), 16);
    /// ```
    pub fn new() -> RawArrayString<A> {
        unsafe {
            let mut xs = MaybeUninitCopy::uninitialized();
            *xs.ptr_mut() = 0u8;
            RawArrayString { xs }
        }
    }

    /// Return the length of the string.
    #[inline]
    pub fn len(&self) -> usize {
        let array = self.xs.ptr() as *const A;
        let s = unsafe { *array };
        let n = s.as_slice().iter().position(|&x| x == 0u8);
        match n {
            Some(n) => n,
            _ => A::CAPACITY,
        }
    }

    /// Returns whether the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { *self.xs.ptr() == 0u8 }
    }

    /// Create a new `RawArrayString` from a `str`.
    ///
    /// Capacity is inferred from the type parameter.
    ///
    /// **Errors** if the backing array is not large enough to fit the string.
    ///
    /// ```
    /// use raw_array_string::RawArrayString;
    ///
    /// let mut string = RawArrayString::<[_; 3]>::from("foo").unwrap();
    /// assert_eq!(&string[..], "foo");
    /// assert_eq!(string.len(), 3);
    /// assert_eq!(string.capacity(), 3);
    /// ```
    pub fn from(s: &str) -> Result<Self, CapacityError<&str>> {
        let mut arraystr = Self::new();
        arraystr.try_push_str(s)?;
        Ok(arraystr)
    }

    /// Create a new `RawArrayString` from a byte string literal.
    ///
    /// **Errors** if the byte string literal is not valid UTF-8.
    ///
    /// ```
    /// use raw_array_string::RawArrayString;
    ///
    /// let string = RawArrayString::from_byte_string(b"hello world").unwrap();
    /// ```
    pub fn from_byte_string(b: &A) -> Result<Self, Utf8Error> {
        let len = str::from_utf8(b.as_slice())?.len();
        debug_assert_eq!(len, A::CAPACITY);
        Ok(RawArrayString {
            xs: MaybeUninitCopy::from(*b),
        })
    }

    /// Make the string empty.
    pub fn clear(&mut self) {
        unsafe {
            let dst = self.xs.ptr_mut();
            *dst = 0u8;
        }
    }

    /// Return the capacity of the `RawArrayString`.
    ///
    /// ```
    /// use raw_array_string::RawArrayString;
    ///
    /// let string = RawArrayString::<[_; 3]>::new();
    /// assert_eq!(string.capacity(), 3);
    /// ```
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        A::CAPACITY
    }
    /// Return if the `RawArrayString` is completely filled.
    ///
    /// ```
    /// use raw_array_string::RawArrayString;
    ///
    /// let mut string = RawArrayString::<[_; 1]>::new();
    /// assert!(!string.is_full());
    /// string.push_str("A");
    /// assert!(string.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Adds the given string slice to the end of the string.
    ///
    /// Returns `Ok` if the push succeeds.
    ///
    /// **Errors** if the backing array is not large enough to fit the string.
    ///
    /// ```
    /// use raw_array_string::RawArrayString;
    ///
    /// let mut string = RawArrayString::<[_; 2]>::new();
    ///
    /// string.try_push_str("a").unwrap();
    /// let overflow1 = string.try_push_str("bc");
    /// string.try_push_str("d").unwrap();
    /// let overflow2 = string.try_push_str("ef");
    ///
    /// assert_eq!(&string[..], "ad");
    /// assert_eq!(overflow1.unwrap_err().element(), "bc");
    /// assert_eq!(overflow2.unwrap_err().element(), "ef");
    /// ```
    pub fn try_push_str<'a>(&mut self, s: &'a str) -> Result<(), CapacityError<&'a str>> {
        if s.len() > self.capacity() - self.len() {
            return Err(CapacityError::new(s));
        } else if s.len() == self.capacity() - self.len() {
            unsafe {
                let dst = self.xs.ptr_mut().offset(self.len() as isize);
                let src = s.as_ptr();
                ptr::copy_nonoverlapping(src, dst, s.len());
            }
        } else {
            unsafe {
                let dst = self.xs.ptr_mut().offset(self.len() as isize);
                let src = s.as_ptr();
                ptr::copy_nonoverlapping(src, dst, s.len());
                *((dst as usize + s.len()) as *mut u8) = 0u8;
            }
        }

        Ok(())
    }

    /// Adds the given string slice to the end of the string.
    ///
    /// ***Panics*** if the backing array is not large enough to fit the string.
    ///
    /// ```
    /// use raw_array_string::RawArrayString;
    ///
    /// let mut string = RawArrayString::<[_; 2]>::new();
    ///
    /// string.push_str("a");
    /// string.push_str("d");
    ///
    /// assert_eq!(&string[..], "ad");
    /// ```
    pub fn push_str(&mut self, s: &str) {
        self.try_push_str(s).unwrap()
    }
}

impl<A> Clone for RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    fn clone(&self) -> RawArrayString<A> {
        *self
    }
    fn clone_from(&mut self, rhs: &Self) {
        // guaranteed to fit due to types matching.
        self.clear();
        self.try_push_str(rhs).ok();
    }
}

impl<A> Deref for RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        unsafe {
            let sl = slice::from_raw_parts(self.xs.ptr(), self.len());
            str::from_utf8_unchecked(sl)
        }
    }
}

impl<A> DerefMut for RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        unsafe {
            let sl = slice::from_raw_parts_mut(self.xs.ptr_mut(), self.len());
            str::from_utf8_unchecked_mut(sl)
        }
    }
}

impl<A> PartialEq for RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    fn eq(&self, rhs: &Self) -> bool {
        **self == **rhs
    }
}

impl<A> PartialEq<str> for RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    fn eq(&self, rhs: &str) -> bool {
        &**self == rhs
    }
}

impl<A> PartialEq<RawArrayString<A>> for str
where
    A: Array<Item = u8> + Copy,
{
    fn eq(&self, rhs: &RawArrayString<A>) -> bool {
        self == &**rhs
    }
}

impl<A> fmt::Debug for RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<A> fmt::Display for RawArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[cfg(feature = "serde")]
/// Requires crate feature `"serde"`
impl<A> Serialize for ArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&*self)
    }
}

#[cfg(feature = "serde")]
/// Requires crate feature `"serde"`
impl<'de, A> Deserialize<'de> for ArrayString<A>
where
    A: Array<Item = u8> + Copy,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::marker::PhantomData;

        struct ArrayStringVisitor<A: Array>(PhantomData<A>);

        impl<'de, A: Copy + Array<Item = u8>> Visitor<'de> for ArrayStringVisitor<A> {
            type Value = RawArrayString<A>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a string no more than {} bytes long",
                    A::CAPACITY
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                RawArrayString::from(v).map_err(|_| E::invalid_length(v.len(), &self))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let s = str::from_utf8(v)
                    .map_err(|_| E::invalid_value(de::Unexpected::Bytes(v), &self))?;

                RawArrayString::from(s).map_err(|_| E::invalid_length(s.len(), &self))
            }
        }

        deserializer.deserialize_str(ArrayStringVisitor::<A>(PhantomData))
    }
}
