//! LLRB, [left-leaning-red-black][llrb], tree is memory optimized data
//! structure for indexing sortable data. This package provides a basic
//! implementation with following properties:
//!
//! - Each entry in LLRB instance correspond to a {Key, Value} pair.
//! - Parametrised over Key type and Value type.
//! - CRUD operations, via create(), set(), get(), delete() api.
//! - No Durability guarantee.
//! - Not thread safe.
//! - Full table scan, to iterate over all entries.
//! - Range scan, to iterate between a ``low`` and ``high``.
//! - Reverse iteration.
//!
//! [Llrb] instance and its API uses Rust's ownership model and borrow
//! semantics to ensure thread safe operation.
//!
//! Constructing a new [Llrb] instance:
//! ```
//! use llrb_index::Llrb;
//! let llrb: Llrb<i32,i32> = Llrb::new("myinstance");
//! let id = llrb.id();
//! assert_eq!(id, "myinstance");
//! ```
//!
//! Constructing [Llrb] instance from an iterator:
//! ```
//! use llrb_index::Llrb;
//! let iter = vec![(1,10), (2,20)].into_iter();
//! let llrb: Llrb<i32,i32> = Llrb::load_from("myinstance", iter).unwrap();
//! ```
//!
//! CRUD operations on [Llrb] instance:
//! ```
//! use llrb_index::Llrb;
//! let mut llrb: Llrb<String,String> = Llrb::new("myinstance");
//!
//! llrb.create("key1".to_string(), "value1".to_string());
//! llrb.create("key2".to_string(), "value2".to_string());
//! llrb.set("key2".to_string(), "value3".to_string());
//!
//! let n = llrb.count();
//! assert_eq!(n, 2);
//!
//! let value = llrb.get("key1").unwrap();
//! assert_eq!(value, "value1".to_string());
//! let value = llrb.get("key2").unwrap();
//! assert_eq!(value, "value3".to_string());
//!
//! let old_value = llrb.delete("key1").unwrap();
//! assert_eq!(old_value, "value1".to_string());
//! ```
//!
//! Full table scan:
//! ```
//! use llrb_index::Llrb;
//! let mut llrb: Llrb<String,String> = Llrb::new("myinstance");
//! llrb.set("key1".to_string(), "value1".to_string());
//! llrb.set("key2".to_string(), "value2".to_string());
//!
//! for (i, (key, value)) in llrb.iter().enumerate() {
//!     let refkey = format!("key{}", i+1);
//!     let refval = format!("value{}", i+1);
//!     assert_eq!(refkey, key);
//!     assert_eq!(refval, value);
//! }
//! ```
//!
//! Range scan:
//! ```
//! use std::ops::Bound;
//! use llrb_index::Llrb;
//! let mut llrb: Llrb<String,String> = Llrb::new("myinstance");
//!
//! llrb.set("key1".to_string(), "value1".to_string());
//! llrb.set("key2".to_string(), "value2".to_string());
//! llrb.set("key3".to_string(), "value3".to_string());
//!
//! let low = Bound::Excluded("key1".to_string());
//! let high = Bound::Excluded("key2".to_string());
//! let item = llrb.range(low, high).next();
//! assert_eq!(item, None);
//!
//! let low = Bound::Excluded("key1".to_string());
//! let high = Bound::Excluded("key3".to_string());
//! let item = llrb.range(low, high).next();
//! assert_eq!(item, Some(("key2".to_string(), "value2".to_string())));
//!
//! let low = Bound::Included("key1".to_string());
//! let high = Bound::Included("key3".to_string());
//! let mut ranger = llrb.range(low, high);
//! let item = ranger.next();
//! assert_eq!(item, Some(("key1".to_string(), "value1".to_string())));
//! let item = ranger.last();
//! assert_eq!(item, Some(("key3".to_string(), "value3".to_string())));
//! ```
//!
//! Reverse scan:
//! ```
//! use std::ops::Bound;
//! use llrb_index::Llrb;
//! let mut llrb: Llrb<String,String> = Llrb::new("myinstance");
//!
//! llrb.set("key1".to_string(), "value1".to_string());
//! llrb.set("key2".to_string(), "value2".to_string());
//! llrb.set("key3".to_string(), "value3".to_string());
//!
//! let low = Bound::Included("key1".to_string());
//! let high = Bound::Included("key3".to_string());
//! let mut iter = llrb.range(low, high).rev();
//! let item = iter.next();
//! assert_eq!(item, Some(("key3".to_string(), "value3".to_string())));
//! let item = iter.last();
//! assert_eq!(item, Some(("key1".to_string(), "value1".to_string())));
//! ```
//!
//! [llrb]: https://en.wikipedia.org/wiki/Left-leaning_red-black_tree
mod error;
mod llrb;

pub use crate::error::LlrbError;
pub use crate::llrb::Llrb;

#[cfg(test)]
mod llrb_test;
