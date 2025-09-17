use bytes::Bytes;
use std::marker::PhantomData;
use tokio::sync::Mutex;

use crate::infrastructure::from_slice::FromSlice;

pub struct Dsv<T: FromSlice> {
    pub has_headers: bool,
    pub buffer: Bytes,
    _structure: PhantomData<T>,
    pub cache: Mutex<Option<Vec<<T as FromSlice>::Item>>>,
}

impl<T: FromSlice> Dsv<T> {
    pub fn new(has_headers: bool, buffer: Bytes) -> Self {
        Self {
            has_headers,
            buffer,
            _structure: PhantomData,
            cache: Mutex::new(None),
        }
    }
}
