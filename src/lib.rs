//! LLRB, [left-leaning-red-black][llrb], is memory optimized data structure
//! for indexing sortable data. This package provides a basic implementation
//! with following properties:
//! - CRUD operations, via create(), set(), get(), delete() api.
//! - No Durability gaurantee.
//! - Not thread safe.
//! - Full table scan, to iterate over all entries.
//! - Range scan, to iterate between a ``low`` and ``high``.
//! - Reverse iteration.
//!
//! LLRB instance and its API uses Rust's ownership model and borrow
//! semantics to ensure thread safe operation.
//!
//! [llrb]: https://en.wikipedia.org/wiki/Left-leaning_red-black_tree
mod error;
mod llrb;

pub use crate::error::LlrbError;
pub use crate::llrb::Llrb;

#[cfg(test)]
mod llrb_test;
