use bytes::Bytes;
use std::marker::PhantomData;

use crate::infrastructure::from_slice::FromSlice;

pub struct Dsv<T: FromSlice> {
    pub has_headers: bool,
    pub buffer: Bytes,
    _structure: PhantomData<T>,
}

impl<T: FromSlice> Dsv<T> {
    pub fn new(has_headers: bool, buffer: Bytes) -> Self {
        Self {
            has_headers,
            buffer,
            _structure: PhantomData,
        }
    }
}
