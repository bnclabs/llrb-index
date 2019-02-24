/// LlrbError enumerates over all possible errors that this package
/// shall return.
#[derive(Debug, PartialEq)]
pub enum LlrbError<K>
where
    K: Clone + Ord,
{
    /// Fatal case, breaking one of the two LLRB rules.
    ConsecutiveReds,
    /// Fatal case, breaking one of the two LLRB rules. The String
    /// component of this variant can be used for debugging.
    UnbalancedBlacks(String),
    /// Fatal case, index entries are not in sort-order.
    SortError(K, K),
    /// Returned by create() API when key is already present.
    OverwriteKey,
}
