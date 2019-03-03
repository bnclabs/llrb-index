/// Can be used while indexing keys without values, like ``Llrb<K, Empty>``.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Empty {}
