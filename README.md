Left Leaning Red Black Tree
===========================

[![Rustdoc](https://img.shields.io/badge/rustdoc-hosted-blue.svg)](https://docs.rs/llbr-index)
[![GitPitch](https://gitpitch.com/assets/badge.svg)](https://gitpitch.com/bnclabs/llbr-index/master?grs=github)
[![Build Status](https://travis-ci.org/bnclabs/llbr-index.svg?branch=master)](https://travis-ci.org/bnclabs/llbr-index)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

LLRB, Left Leaning Red Black, tree a popular data structured with
interesting features like:
* [x] Optimized for in-memory index.
* [x] Self-balancing data structure.
* [x] Read optimized.
* [x] A random successful search examines log2 N − 0.5 nodes.
* [x] The average tree height is about 2 * log2 N.
* [x] The average size of left subtree exhibits log-oscillating behavior.

Refer to this [wikipedia link] for more information on LLRB algorithm.

**Compatibility policy**

``llrb-index`` shall officially support the latest version of rust stable
compiler and nightly builds.

[wikipedia link]: https://en.wikipedia.org/wiki/Left-leaning_red%E2%80%93black_tree
