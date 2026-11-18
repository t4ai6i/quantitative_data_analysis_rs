use bytes::Bytes;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::sync::{Mutex, OnceLock};

use crate::infrastructure::from_slice::FromSlice;

/// Delimited data parsed lazily and cached per instance after a successful parse.
///
/// Do not modify `buffer` or `has_headers` after construction. Changes to these
/// public fields do not invalidate the cached items.
pub struct Dsv<T: FromSlice> {
    pub has_headers: bool,
    pub buffer: Bytes,
    _structure: PhantomData<T>,
    parsed_items: OnceLock<Vec<T::Item>>,
    initialization: Mutex<()>,
}

impl<T: FromSlice> Dsv<T> {
    pub fn new(has_headers: bool, buffer: Bytes) -> Self {
        Self {
            has_headers,
            buffer,
            _structure: PhantomData,
            parsed_items: OnceLock::new(),
            initialization: Mutex::new(()),
        }
    }

    pub(crate) fn items(&self) -> anyhow::Result<&[T::Item]>
    where
        <T::Item as TryFrom<T::Deserialize>>::Error: Debug + Display,
    {
        if let Some(items) = self.parsed_items.get() {
            return Ok(items);
        }
        let _guard = self
            .initialization
            .lock()
            .map_err(|error| anyhow::anyhow!("DSV cache initialization lock poisoned: {error}"))?;
        if let Some(items) = self.parsed_items.get() {
            return Ok(items);
        }
        let items = T::process_tabular_data(self.buffer.as_ref(), self.has_headers)?;
        Ok(self.parsed_items.get_or_init(|| items))
    }
}
