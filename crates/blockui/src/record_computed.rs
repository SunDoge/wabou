//! Typed records in the existing host-owned text memo cache.
use crate::{Computed, ComputedSpec, Cx, InstancePath, Record, Result, record_codec};
use std::marker::PhantomData;

/// A derived record cached as canonical encoded data by the host. Encoding and
/// decoding stay in the current implementation; no Rust value crosses the ABI.
pub struct RecordComputed<T> {
    encoded: Computed<String>,
    marker: PhantomData<fn() -> T>,
}
impl<T> Copy for RecordComputed<T> {}
impl<T> Clone for RecordComputed<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Record> RecordComputed<T> {
    #[doc(hidden)]
    pub const fn __encoded(name: &str, compute: for<'a> fn(&Cx<'a>) -> Result<String>) -> Self {
        Self {
            encoded: Computed::<String>::new(name, compute),
            marker: PhantomData,
        }
    }
    pub const fn key(self) -> u64 {
        self.encoded.key()
    }
    pub const fn spec(self) -> ComputedSpec {
        self.encoded.spec()
    }
    #[doc(hidden)]
    pub const fn __instance(mut self, instance: InstancePath) -> Self {
        self.encoded = self.encoded.__instance(instance);
        self
    }
    /// Read and decode a host-cached record, subscribing the current observer.
    pub fn get(self, cx: &Cx<'_>) -> Result<T> {
        record_codec::decode_record(&self.encoded.get(cx)?)
    }
}
