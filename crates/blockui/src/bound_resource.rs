use crate::{Cx, File, Key, ReloadPolicy, Resource, Result};

/// A runtime resource descriptor borrowed for one actor turn, never its native handle.
///
/// ```compile_fail
/// use blockui::{BindState, BoundResource, Cx, File, Resource};
/// fn escape(cx: &Cx<'_>) -> BoundResource<'static, 'static, File> {
///     Resource::<File>::new("document").bind(cx).unwrap()
/// }
/// ```
/// ```compile_fail
/// use blockui::{BindState, Cx, File, Resource};
/// fn send(cx: &Cx<'_>) {
///     let file = Resource::<File>::new("document").bind(cx).unwrap();
///     std::thread::scope(|scope| { scope.spawn(move || file.close()); });
/// }
/// ```
pub struct BoundResource<'cx, 'host, T> {
    pub(crate) resource: Resource<T>,
    pub(crate) cx: &'cx Cx<'host>,
}
impl<'cx, 'host, T> BoundResource<'cx, 'host, T> {
    pub(crate) fn new(resource: Resource<T>, cx: &'cx Cx<'host>) -> Self {
        Self { resource, cx }
    }
    /// Data-only identity; never captures a native handle or turn context.
    pub fn handle(&self) -> Resource<T> {
        self.resource
    }
    pub fn key(&self) -> u64 {
        self.resource.key()
    }
    pub fn scoped(&self, scope: Key) -> Self {
        Self::new(self.resource.scoped(scope), self.cx)
    }
    pub fn on_reload(&self, policy: ReloadPolicy) -> Self {
        Self::new(self.resource.on_reload(policy), self.cx)
    }
}
impl BoundResource<'_, '_, File> {
    /// See [`Resource::open`]. Failed turns restore the old resource.
    pub fn open(&self, path: &str) -> Result<()> {
        self.resource.open(self.cx, path)
    }
    /// See [`Resource::open_writable`]. Does not create or truncate files.
    pub fn open_writable(&self, path: &str) -> Result<()> {
        self.resource.open_writable(self.cx, path)
    }
    /// See [`Resource::close`]. Host work already admitted owns its resources.
    pub fn close(&self) -> Result<()> {
        self.resource.close(self.cx)
    }
    /// See [`Resource::read`]. External bytes are not automatically reactive.
    pub fn read(&self, offset: u64, limit: usize) -> Result<Vec<u8>> {
        self.resource.read(self.cx, offset, limit)
    }
}
/// Extract only the durable identity/policy, never the borrowed turn context.
impl<T> From<&BoundResource<'_, '_, T>> for Resource<T> {
    fn from(value: &BoundResource<'_, '_, T>) -> Self {
        value.resource
    }
}
