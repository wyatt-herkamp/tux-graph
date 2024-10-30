macro_rules! id_type {
    (
        $ty:ident
    ) => {
        impl std::cmp::PartialEq for $ty {
            fn eq(&self, other: &$ty) -> bool {
                self.0 == other.0
            }
        }
        impl std::cmp::PartialEq<$ty> for usize {
            fn eq(&self, other: &$ty) -> bool {
                *self == other.0
            }
        }
        impl std::cmp::PartialEq<usize> for $ty {
            fn eq(&self, other: &usize) -> bool {
                self.0 == *other
            }
        }
        impl std::cmp::Eq for $ty {}
        #[allow(clippy::non_canonical_partial_ord_impl)]
        impl core::cmp::PartialOrd for $ty {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }
        impl core::cmp::Ord for $ty {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }

        impl core::cmp::PartialOrd<usize> for $ty {
            fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }
        impl core::cmp::PartialOrd<$ty> for usize {
            fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }
        impl std::hash::Hash for $ty {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }
        impl crate::utils::IdType for $ty {
            fn from_usize(id: usize) -> Self {
                Self(id)
            }
        }
    };
}
pub(crate) use id_type;
