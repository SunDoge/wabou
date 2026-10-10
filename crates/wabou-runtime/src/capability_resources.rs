//! Host-owned native resources accessed through generational handles.
use crate::resource::{ResourceKey, ResourceRegistry};
use std::sync::{Arc, Mutex};

/// One resource family belonging to a resident host provider. Construct resources
/// in resident code: storing a replaceable library's objects or callbacks here
/// would retain its destructors/vtables and prevent safe unloading.
pub struct CapabilityResources<Family, Value> {
    entries: Arc<Mutex<ResourceRegistry<Family, Value>>>,
}
impl<Family, Value> Clone for CapabilityResources<Family, Value> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}
impl<Family, Value> Default for CapabilityResources<Family, Value> {
    fn default() -> Self {
        Self {
            entries: Arc::new(Mutex::new(ResourceRegistry::new())),
        }
    }
}
impl<Family, Value> CapabilityResources<Family, Value> {
    /// Insert a resident resource. Removing it invalidates this handle generation.
    pub fn insert(&self, value: Value) -> Result<ResourceKey<Family>, String> {
        Ok(self
            .entries
            .lock()
            .map_err(|e| e.to_string())?
            .insert(value))
    }
    /// Access a resource under the registry lock. Keep callbacks bounded and do
    /// not reenter this registry. Async work should clone an Arc resource here.
    pub fn with<T>(
        &self,
        key: ResourceKey<Family>,
        access: impl FnOnce(&Value) -> Result<T, String>,
    ) -> Result<T, String> {
        let entries = self.entries.lock().map_err(|e| e.to_string())?;
        access(
            entries
                .get(key)
                .ok_or("resource handle is stale or unknown")?,
        )
    }
    /// Remove a resource. Drop/close occurs after releasing the registry lock.
    pub fn remove(&self, key: ResourceKey<Family>) -> Result<Value, String> {
        self.entries
            .lock()
            .map_err(|e| e.to_string())?
            .remove(key)
            .ok_or_else(|| "resource handle is stale or unknown".into())
    }
}
