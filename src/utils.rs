pub mod macros;
/// Returns true if the slice contains the item.
///
/// Uses binary search to determine if the slice contains the item.
#[inline]
pub fn binary_search_contains<T: Ord>(slice: &[T], item: &T) -> bool {
    slice.binary_search(item).is_ok()
}
pub trait ExtendedVec<T> {
    fn binary_search_contains(&self, item: &T) -> bool
    where
        T: Ord;
    /// Push where the return value is the index of the pushed item.
    fn push_with_id(&mut self, item: T) -> usize;
    /// Pushes the item and returns the wrapped id.
    fn push_with_wrapped_id<I>(&mut self, item: T) -> I
    where
        I: IdType,
    {
        I::from_usize(self.push_with_id(item))
    }
}

impl<T> ExtendedVec<T> for Vec<T> {
    fn binary_search_contains(&self, item: &T) -> bool
    where
        T: Ord,
    {
        binary_search_contains(self, item)
    }
    fn push_with_id(&mut self, item: T) -> usize {
        let index = self.len();
        self.push(item);
        index
    }
}
pub trait IdType {
    fn from_usize(id: usize) -> Self;
}
