HEAD
====

- Implement Empty type that can be used as values.

0.2.0
=====

* Implement create() method on Llrb.
* Implement random() method on Llrb.
* Rename count() API to len() method on Llrb.
* Stats type and new stats() method on Llrb.
* validate() method should return Stats type.
* Tree depth statistics, in min, mean, max, percentiles.

0.1.0
=====

* Self-balancing data structure.
* Optimized for in-memory index.
* Each entry in LLRB instance correspond to a {Key, Value} pair.
* Parametrised over Key type and Value type.
* CRUD operations, via set(), get(), delete() API.
* Read optimized.
* Full table scan, to iterate over all entries.
* Range scan, to iterate between a ``low`` and ``high``.
* Reverse iteration.

Release Checklist
=================

* Bump up the version:
  * __major__: backward incompatible API changes.
  * __minor__: backward compatible API Changes.
  * __patch__: bug fixes.
* Cargo checklist
  * cargo +stable build; cargo +nightly build
  * cargo +stable doc
  * cargo +nightly clippy --all-targets --all-features
  * cargo +nightly audit
  * cargo +nightly test
  * cargo +nightly bench
  * cargo +nightly benchcmp <old> <new>
  * cargo fix --edition --all-targets
* Travis-CI integration.
* Spell check.
* Create a git-tag for the new version.
* Cargo publish the new version.
* Badges
  * Build passing, Travis continuous integration.
  * Code coverage, codecov and coveralls.
  * Crates badge
  * Downloads badge
  * License badge
  * Rust version badge.
  * Maintenance-related badges based on isitmaintained.com
  * Documentation
  * Gitpitch
