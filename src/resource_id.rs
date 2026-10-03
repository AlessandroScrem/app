use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_RESOURCE_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ResourceId(usize);

impl ResourceId {
    pub fn new() -> Self {
        Self(NEXT_RESOURCE_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn raw(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let a = ResourceId::new();
        let b = ResourceId::new();

        assert_ne!(a, b);
        assert!(b.raw() > a.raw());
    }
}
