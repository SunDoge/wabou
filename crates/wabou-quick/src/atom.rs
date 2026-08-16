//! Runtime-scoped atoms shared by Rust and QuickJS through stable `u32` IDs.

use lasso::{Key, Rodeo, Spur};

/// Stable for the lifetime of one [`AtomPool`]. Zero is reserved as `NONE`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Atom(u32);

impl Atom {
    #[cfg(test)]
    pub(crate) const NONE: Self = Self(0);

    pub(crate) const fn get(self) -> u32 {
        self.0
    }

    pub(crate) const fn from_raw(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Default)]
pub(crate) struct AtomPool {
    inner: Rodeo,
}

impl AtomPool {
    pub(crate) fn intern(&mut self, value: &str) -> Atom {
        let key = self.inner.get_or_intern(value);
        Atom(u32::try_from(key.into_usize()).expect("atom pool exceeded u32") + 1)
    }

    pub(crate) fn resolve(&self, atom: Atom) -> Option<&str> {
        let index = atom.0.checked_sub(1)? as usize;
        let key = Spur::try_from_usize(index)?;
        self.inner.try_resolve(&key)
    }

    pub(crate) fn get(&self, value: &str) -> Option<Atom> {
        let key = self.inner.get(value)?;
        Some(Atom(u32::try_from(key.into_usize()).ok()? + 1))
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_values_share_an_atom_and_zero_is_reserved() {
        let mut pool = AtomPool::default();
        let first = pool.intern("background-color");
        let second = pool.intern("background-color");

        assert_ne!(first, Atom::NONE);
        assert_eq!(first, second);
        assert_eq!(pool.resolve(first), Some("background-color"));
        assert_eq!(pool.resolve(Atom::NONE), None);
        assert_eq!(pool.len(), 1);
    }
}
