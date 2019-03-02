use std::borrow::Borrow;
use std::cmp::{Ord, Ordering};
use std::ops::{Bound, DerefMut};

use rand::Rng;

use crate::depth::Depth;
use crate::error::LlrbError;

// TODO: Sizing.
// TODO: optimize comparison
// TODO: llrb_depth_histogram, as feature, to measure the depth of LLRB tree.

const ITER_LIMIT: usize = 100;

type Delmin<K, V> = (Option<Box<Node<K, V>>>, Option<Node<K, V>>);

type Insert<K, V> = (Option<Box<Node<K, V>>>, Option<LlrbError<K>>);

/// Llrb manage a single instance of in-memory index using
/// [left-leaning-red-black][llrb] tree.
///
/// [llrb]: https://en.wikipedia.org/wiki/Left-leaning_red-black_tree
pub struct Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    name: String,
    root: Option<Box<Node<K, V>>>,
    n_count: usize, // number of entries in the tree.
}

impl<K, V> Clone for Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn clone(&self) -> Llrb<K, V> {
        Llrb {
            name: self.name.clone(),
            root: self.root.clone(),
            n_count: self.n_count,
        }
    }
}

/// Different ways to construct a new Llrb instance.
impl<K, V> Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    /// Create an empty instance of Llrb, identified by `name`.
    /// Applications can choose unique names.
    pub fn new<S>(name: S) -> Llrb<K, V>
    where
        S: AsRef<str>,
    {
        Llrb {
            name: name.as_ref().to_string(),
            root: None,
            n_count: 0,
        }
    }

    /// Create a new instance of Llrb tree and load it with entries
    /// from `iter`. Note that iterator should return (key, value) tuples,
    /// where key must be ``unique``.
    pub fn load_from<S, I>(name: S, iter: I) -> Result<Llrb<K, V>, LlrbError<K>>
    where
        S: AsRef<str>,
        I: Iterator<Item = (K, V)>,
    {
        let mut llrb = Llrb::new(name);
        for (key, value) in iter {
            llrb.set(key, value);
            llrb.n_count += 1;
        }
        Ok(llrb)
    }
}

/// Maintenance API.
impl<K, V> Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    /// Identify this instance. Applications can choose unique names while
    /// creating Llrb instances.
    pub fn id(&self) -> String {
        self.name.clone()
    }

    /// Return number of entries in this instance.
    pub fn len(&self) -> usize {
        self.n_count
    }

    /// Check whether this index is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n_count == 0
    }

    /// Return quickly with basic statisics, only entries() method is valid
    /// with this statisics.
    pub fn stats(&self) -> Stats {
        let node_size = std::mem::size_of::<Node<K, V>>();
        Stats::new(self.n_count, node_size)
    }
}

/// CRUD operations on Llrb instance.
impl<K, V> Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    /// Get the value for key.
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let root = self.root.as_ref().map(std::ops::Deref::deref);
        self.do_get(root, key)
    }

    fn do_get<Q>(&self, mut node: Option<&Node<K, V>>, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        while node.is_some() {
            let nref = node.unwrap();
            node = match nref.key.borrow().cmp(key) {
                Ordering::Less => nref.right_deref(),
                Ordering::Greater => nref.left_deref(),
                Ordering::Equal => return Some(nref.value.clone()),
            };
        }
        None
    }

    /// Return a random entry from this index.
    pub fn random<R: Rng>(&self, rng: &mut R) -> Option<(K, V)> {
        let mut node = self.root.as_ref().map(std::ops::Deref::deref);

        node?;

        let mut depth = 0;
        let at_depth = rng.gen::<u8>() % 40;
        loop {
            let r: u8 = rng.gen();
            let nref = node.unwrap();

            if at_depth == depth {
                break Some((nref.key.clone(), nref.value.clone()));
            }
            depth += 1;

            let next = if r % 2 == 0 {
                nref.left_deref()
            } else {
                nref.right_deref()
            };

            if next.is_none() {
                break Some((nref.key.clone(), nref.value.clone()));
            } else {
                node = next;
            }
        }
    }

    /// Return an iterator over all entries in this instance.
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            root: self.root.as_ref().map(std::ops::Deref::deref),
            node_iter: vec![].into_iter(),
            after_key: Bound::Unbounded,
            limit: ITER_LIMIT,
            fin: false,
        }
    }

    /// Range over all entries from low to high.
    pub fn range(&self, low: Bound<K>, high: Bound<K>) -> Range<K, V> {
        Range {
            root: self.root.as_ref().map(std::ops::Deref::deref),
            node_iter: vec![].into_iter(),
            low,
            high,
            limit: ITER_LIMIT,
            fin: false,
        }
    }

    /// Create a new {key, value} entry in the index. If key is already
    /// present returns error.
    pub fn create(&mut self, key: K, value: V) -> Option<LlrbError<K>> {
        let root = self.root.take();

        let error = match Llrb::insert(root, key, value) {
            (Some(mut root), error) => {
                root.set_black();
                self.root = Some(root);
                error
            }
            _ => unreachable!(),
        };
        if error.is_none() {
            self.n_count += 1;
        }
        error
    }

    /// Set value for key. If there is an existing entry for key,
    /// overwrite the old value with new value and return the old value.
    pub fn set(&mut self, key: K, value: V) -> Option<V> {
        let root = self.root.take();

        let old_value = match Llrb::upsert(root, key, value) {
            (Some(mut root), old_value) => {
                root.set_black();
                self.root = Some(root);
                old_value
            }
            _ => unreachable!(),
        };
        if old_value.is_none() {
            self.n_count += 1;
        }
        old_value
    }

    /// Delete key from this instance and return its value. If key is
    /// not present, then delete is effectively a no-op.
    pub fn delete<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let root = self.root.take();
        let (root, old_value) = match Llrb::do_delete(root, key) {
            (None, old_value) => (None, old_value),
            (Some(mut root), old_value) => {
                root.set_black();
                (Some(root), old_value)
            }
        };
        self.root = root;
        if old_value.is_some() {
            self.n_count -= 1;
        }
        old_value
    }

    /// Validate LLRB tree with following rules:
    /// * From root to any leaf, no consecutive reds allowed in its path.
    /// * Number of blacks should be same on under left child and right child.
    /// * Make sure that keys are in sorted order.
    ///
    /// Additionally return full statistics on the tree. Refer to [`Stats`]
    /// for more information.
    pub fn validate(&self) -> Result<Stats, LlrbError<K>> {
        let root = self.root.as_ref().map(std::ops::Deref::deref);
        let (red, nb, d) = (is_red(root), 0, 0);
        let node_size = std::mem::size_of::<Node<K, V>>();
        let mut stats = Stats::new(self.n_count, node_size);

        stats.blacks = Llrb::validate_tree(root, red, nb, d, &mut stats)?;
        Ok(stats)
    }

    fn validate_tree(
        node: Option<&Node<K, V>>,
        fromred: bool,
        mut nb: usize,
        depth: usize,
        stats: &mut Stats,
    ) -> Result<usize, LlrbError<K>> {
        if node.is_none() {
            stats.depths.sample(depth);
            return Ok(nb);
        }

        let red = is_red(node.as_ref().map(std::ops::Deref::deref));
        if fromred && red {
            return Err(LlrbError::ConsecutiveReds);
        }
        if !red {
            nb += 1;
        }
        let node = &node.as_ref().unwrap();
        let left = node.left_deref();
        let right = node.right_deref();
        let lblacks = Llrb::validate_tree(left, red, nb, depth + 1, stats)?;
        let rblacks = Llrb::validate_tree(right, red, nb, depth + 1, stats)?;
        if lblacks != rblacks {
            let err = format!(
                "llrb_store: unbalanced blacks left: {} and right: {}",
                lblacks, rblacks,
            );
            return Err(LlrbError::UnbalancedBlacks(err));
        }
        if node.left.is_some() {
            let left = node.left.as_ref().unwrap();
            if left.key.ge(&node.key) {
                let (lkey, parent) = (left.key.clone(), node.key.clone());
                return Err(LlrbError::SortError(lkey, parent));
            }
        }
        if node.right.is_some() {
            let right = node.right.as_ref().unwrap();
            if right.key.le(&node.key) {
                let (rkey, parent) = (right.key.clone(), node.key.clone());
                return Err(LlrbError::SortError(rkey, parent));
            }
        }
        Ok(lblacks)
    }
}

impl<K, V> Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn insert(node: Option<Box<Node<K, V>>>, key: K, value: V) -> Insert<K, V> {
        if node.is_none() {
            return (Some(Node::new(key, value, false /*black*/)), None);
        }

        let mut node = Llrb::walkdown_rot23(node.unwrap());

        match node.key.cmp(&key) {
            Ordering::Greater => {
                let left = node.left.take();
                let (left, e) = Llrb::insert(left, key, value);
                node.left = left;
                (Some(Llrb::walkuprot_23(node)), e)
            }
            Ordering::Less => {
                let right = node.right.take();
                let (right, e) = Llrb::insert(right, key, value);
                node.right = right;
                (Some(Llrb::walkuprot_23(node)), e)
            }
            Ordering::Equal => (
                Some(Llrb::walkuprot_23(node)),
                Some(LlrbError::OverwriteKey),
            ),
        }
    }

    fn upsert(
        node: Option<Box<Node<K, V>>>,
        key: K,
        value: V,
    ) -> (Option<Box<Node<K, V>>>, Option<V>) {
        if node.is_none() {
            return (Some(Node::new(key, value, false /*black*/)), None);
        }

        let mut node = node.unwrap();
        node = Llrb::walkdown_rot23(node);

        match node.key.cmp(&key) {
            Ordering::Greater => {
                let left = node.left.take();
                let (left, o) = Llrb::upsert(left, key, value);
                node.left = left;
                (Some(Llrb::walkuprot_23(node)), o)
            }
            Ordering::Less => {
                let right = node.right.take();
                let (right, o) = Llrb::upsert(right, key, value);
                node.right = right;
                (Some(Llrb::walkuprot_23(node)), o)
            }
            Ordering::Equal => {
                let old_value = node.value.clone();
                node.set_value(value);
                (Some(Llrb::walkuprot_23(node)), Some(old_value))
            }
        }
    }

    fn do_delete<Q>(node: Option<Box<Node<K, V>>>, key: &Q) -> (Option<Box<Node<K, V>>>, Option<V>)
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut node = match node {
            None => return (None, None),
            Some(node) => node,
        };

        if node.key.borrow().gt(key) {
            if node.left.is_none() {
                (Some(node), None)
            } else {
                let ok = !is_red(node.left_deref());
                if ok && !is_red(node.left.as_ref().unwrap().left_deref()) {
                    node = Llrb::move_red_left(node);
                }
                let (left, old_value) = Llrb::do_delete(node.left.take(), key);
                node.left = left;
                (Some(Llrb::fixup(node)), old_value)
            }
        } else {
            if is_red(node.left_deref()) {
                node = Llrb::rotate_right(node);
            }

            if !node.key.borrow().lt(key) && node.right.is_none() {
                return (None, Some(node.value.clone()));
            }

            let ok = node.right.is_some() && !is_red(node.right_deref());
            if ok && !is_red(node.right.as_ref().unwrap().left_deref()) {
                node = Llrb::move_red_right(node);
            }

            if !node.key.borrow().lt(key) {
                // node == key
                let (right, mut res_node) = Llrb::delete_min(node.right.take());
                node.right = right;
                if res_node.is_none() {
                    panic!("do_delete(): fatal logic, call the programmer");
                }
                let subdel = res_node.take().unwrap();
                let mut newnode = Box::new(subdel.clone_detach());
                newnode.left = node.left.take();
                newnode.right = node.right.take();
                newnode.black = node.black;
                (Some(Llrb::fixup(newnode)), Some(node.value.clone()))
            } else {
                let (right, old_value) = Llrb::do_delete(node.right.take(), key);
                node.right = right;
                (Some(Llrb::fixup(node)), old_value)
            }
        }
    }

    fn delete_min(node: Option<Box<Node<K, V>>>) -> Delmin<K, V> {
        if node.is_none() {
            return (None, None);
        }
        let mut node = node.unwrap();
        if node.left.is_none() {
            return (None, Some(*node));
        }
        let left = node.left_deref();
        if !is_red(left) && !is_red(left.unwrap().left_deref()) {
            node = Llrb::move_red_left(node);
        }
        let (left, old_node) = Llrb::delete_min(node.left.take());
        node.left = left;
        (Some(Llrb::fixup(node)), old_node)
    }

    //--------- rotation routines for 2-3 algorithm ----------------

    fn walkdown_rot23(node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        node
    }

    fn walkuprot_23(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        if is_red(node.right_deref()) && !is_red(node.left_deref()) {
            node = Llrb::rotate_left(node);
        }
        let left = node.left_deref();
        if is_red(left) && is_red(left.unwrap().left_deref()) {
            node = Llrb::rotate_right(node);
        }
        if is_red(node.left_deref()) && is_red(node.right_deref()) {
            Llrb::flip(node.deref_mut())
        }
        node
    }

    //              (i)                       (i)
    //               |                         |
    //              node                       x
    //              /  \                      / \
    //             /    (r)                 (r)  \
    //            /       \                 /     \
    //          left       x             node      xr
    //                    / \            /  \
    //                  xl   xr       left   xl
    //
    fn rotate_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        if is_black(node.right_deref()) {
            panic!("rotateleft(): rotating a black link ? Call the programmer");
        }
        let mut x = node.right.take().unwrap();
        node.right = x.left.take();
        x.black = node.black;
        node.set_red();
        x.left = Some(node);
        x
    }

    //              (i)                       (i)
    //               |                         |
    //              node                       x
    //              /  \                      / \
    //            (r)   \                   (r)  \
    //           /       \                 /      \
    //          x       right             xl      node
    //         / \                                / \
    //       xl   xr                             xr  right
    //
    fn rotate_right(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        if is_black(node.left_deref()) {
            panic!("rotateright(): rotating a black link ? Call the programmer")
        }
        let mut x = node.left.take().unwrap();
        node.left = x.right.take();
        x.black = node.black;
        node.set_red();
        x.right = Some(node);
        x
    }

    //        (x)                   (!x)
    //         |                     |
    //        node                  node
    //        / \                   / \
    //      (y) (z)              (!y) (!z)
    //     /      \              /      \
    //   left    right         left    right
    //
    fn flip(node: &mut Node<K, V>) {
        node.left.as_mut().unwrap().toggle_link();
        node.right.as_mut().unwrap().toggle_link();
        node.toggle_link();
    }

    fn fixup(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        node = if is_red(node.right_deref()) {
            Llrb::rotate_left(node)
        } else {
            node
        };
        node = {
            let left = node.left_deref();
            if is_red(left) && is_red(left.unwrap().left_deref()) {
                Llrb::rotate_right(node)
            } else {
                node
            }
        };
        if is_red(node.left_deref()) && is_red(node.right_deref()) {
            Llrb::flip(node.deref_mut());
        }
        node
    }

    fn move_red_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        Llrb::flip(node.deref_mut());
        if is_red(node.right.as_ref().unwrap().left_deref()) {
            node.right = Some(Llrb::rotate_right(node.right.take().unwrap()));
            node = Llrb::rotate_left(node);
            Llrb::flip(node.deref_mut());
        }
        node
    }

    fn move_red_right(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        Llrb::flip(node.deref_mut());
        if is_red(node.left.as_ref().unwrap().left_deref()) {
            node = Llrb::rotate_right(node);
            Llrb::flip(node.deref_mut());
        }
        node
    }
}

fn is_red<K, V>(node: Option<&Node<K, V>>) -> bool
where
    K: Clone + Ord,
    V: Clone,
{
    match node {
        None => false,
        node @ Some(_) => !is_black(node),
    }
}

fn is_black<K, V>(node: Option<&Node<K, V>>) -> bool
where
    K: Clone + Ord,
    V: Clone,
{
    match node {
        None => true,
        Some(node) => node.is_black(),
    }
}

pub struct Iter<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    root: Option<&'a Node<K, V>>,
    node_iter: std::vec::IntoIter<(K, V)>,
    after_key: Bound<K>,
    limit: usize,
    fin: bool,
}

impl<'a, K, V> Iter<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn scan_iter(
        &self,
        node: Option<&Node<K, V>>,
        acc: &mut Vec<(K, V)>, // accumulator for batch of nodes
    ) -> bool {
        if node.is_none() {
            return true;
        }
        let node = node.unwrap();

        let left = node.left_deref();
        let right = node.right_deref();
        match &self.after_key {
            Bound::Included(akey) | Bound::Excluded(akey) => {
                if node.key.borrow().le(akey) {
                    return self.scan_iter(right, acc);
                }
            }
            Bound::Unbounded => (),
        }

        if !self.scan_iter(left, acc) {
            return false;
        }

        acc.push((node.key.clone(), node.value.clone()));
        if acc.len() >= self.limit {
            return false;
        }

        self.scan_iter(right, acc)
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.fin {
            return None;
        }

        let item = self.node_iter.next();
        if item.is_some() {
            return item;
        }

        let mut acc: Vec<(K, V)> = Vec::with_capacity(self.limit);
        self.scan_iter(self.root, &mut acc);

        if acc.is_empty() {
            self.fin = true;
            None
        } else {
            self.after_key = Bound::Excluded(acc.last().unwrap().0.clone());
            self.node_iter = acc.into_iter();
            self.node_iter.next()
        }
    }
}

pub struct Range<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    root: Option<&'a Node<K, V>>,
    node_iter: std::vec::IntoIter<(K, V)>,
    low: Bound<K>,
    high: Bound<K>,
    limit: usize,
    fin: bool,
}

impl<'a, K, V> Range<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    pub fn rev(self) -> Reverse<'a, K, V> {
        Reverse {
            root: self.root,
            node_iter: vec![].into_iter(),
            low: self.low,
            high: self.high,
            limit: self.limit,
            fin: false,
        }
    }

    fn range_iter(
        &self,
        node: Option<&Node<K, V>>,
        acc: &mut Vec<(K, V)>, // accumulator for batch of nodes
    ) -> bool {
        if node.is_none() {
            return true;
        }
        let node = node.unwrap();

        let left = node.left_deref();
        let right = node.right_deref();
        match &self.low {
            Bound::Included(qow) if node.key.lt(qow) => {
                return self.range_iter(right, acc);
            }
            Bound::Excluded(qow) if node.key.le(qow) => {
                return self.range_iter(right, acc);
            }
            _ => (),
        }

        if !self.range_iter(left, acc) {
            return false;
        }

        acc.push((node.key.clone(), node.value.clone()));
        if acc.len() >= self.limit {
            return false;
        }

        self.range_iter(right, acc)
    }
}

impl<'a, K, V> Iterator for Range<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.fin {
            return None;
        }

        let mut item = self.node_iter.next();
        if item.is_none() {
            let mut acc: Vec<(K, V)> = Vec::with_capacity(self.limit);
            self.range_iter(self.root, &mut acc);
            item = if !acc.is_empty() {
                self.low = Bound::Excluded(acc.last().unwrap().0.clone());
                self.node_iter = acc.into_iter();
                self.node_iter.next()
            } else {
                None
            };
        }

        if item.is_none() {
            self.fin = true;
            return None;
        }

        // handle upper limit
        let item = item.unwrap();
        match &self.high {
            Bound::Unbounded => Some(item),
            Bound::Included(qigh) if item.0.le(qigh) => Some(item),
            Bound::Excluded(qigh) if item.0.lt(qigh) => Some(item),
            _ => {
                self.fin = true;
                None
            }
        }
    }
}

pub struct Reverse<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    root: Option<&'a Node<K, V>>,
    node_iter: std::vec::IntoIter<(K, V)>,
    high: Bound<K>,
    low: Bound<K>,
    limit: usize,
    fin: bool,
}

impl<'a, K, V> Reverse<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn reverse_iter(
        &self,
        node: Option<&Node<K, V>>,
        acc: &mut Vec<(K, V)>, // accumulator for batch of nodes
    ) -> bool {
        if node.is_none() {
            return true;
        }
        let node = node.unwrap();

        let left = node.left_deref();
        let right = node.right_deref();
        match &self.high {
            Bound::Included(qigh) if node.key.gt(qigh) => {
                return self.reverse_iter(left, acc);
            }
            Bound::Excluded(qigh) if node.key.ge(qigh) => {
                return self.reverse_iter(left, acc);
            }
            _ => (),
        }

        if !self.reverse_iter(right, acc) {
            return false;
        }

        acc.push((node.key.clone(), node.value.clone()));
        if acc.len() >= self.limit {
            return false;
        }

        self.reverse_iter(left, acc)
    }
}

impl<'a, K, V> Iterator for Reverse<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        //println!("yyy");
        if self.fin {
            return None;
        }

        let mut item = self.node_iter.next();
        if item.is_none() {
            let mut acc: Vec<(K, V)> = Vec::with_capacity(self.limit);
            self.reverse_iter(self.root, &mut acc);
            item = if !acc.is_empty() {
                self.high = Bound::Excluded(acc.last().unwrap().0.clone());
                self.node_iter = acc.into_iter();
                self.node_iter.next()
            } else {
                None
            }
        }

        if item.is_none() {
            self.fin = true;
            return None;
        }

        // handle lower limit
        let item = item.unwrap();
        match &self.low {
            Bound::Unbounded => Some(item),
            Bound::Included(qow) if item.0.ge(qow) => Some(item),
            Bound::Excluded(qow) if item.0.gt(qow) => Some(item),
            _ => {
                //println!("llrb reverse over {:?}", &self.low);
                self.fin = true;
                None
            }
        }
    }
}

/// Node corresponds to a single entry in Llrb instance.
#[derive(Clone)]
pub struct Node<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    key: K,
    value: V,
    black: bool,                    // store: black or red
    left: Option<Box<Node<K, V>>>,  // store: left child
    right: Option<Box<Node<K, V>>>, // store: right child
}

// Primary operations on a single node.
impl<K, V> Node<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    // CREATE operation
    fn new(key: K, value: V, black: bool) -> Box<Node<K, V>> {
        Box::new(Node {
            key,
            value,
            black,
            left: None,
            right: None,
        })
    }

    // clone and detach this node from the tree.
    fn clone_detach(&self) -> Node<K, V> {
        Node {
            key: self.key.clone(),
            value: self.value.clone(),
            black: self.black,
            left: None,
            right: None,
        }
    }

    fn left_deref(&self) -> Option<&Node<K, V>> {
        self.left.as_ref().map(std::ops::Deref::deref)
    }

    fn right_deref(&self) -> Option<&Node<K, V>> {
        self.right.as_ref().map(std::ops::Deref::deref)
    }

    #[allow(dead_code)]
    fn left_deref_mut(&mut self) -> Option<&mut Node<K, V>> {
        self.left.as_mut().map(std::ops::DerefMut::deref_mut)
    }

    #[allow(dead_code)]
    fn right_deref_mut(&mut self) -> Option<&mut Node<K, V>> {
        self.right.as_mut().map(std::ops::DerefMut::deref_mut)
    }

    // prepend operation, equivalent to SET / INSERT / UPDATE
    fn set_value(&mut self, value: V) {
        self.value = value
    }

    #[inline]
    fn set_red(&mut self) {
        self.black = false
    }

    #[inline]
    fn set_black(&mut self) {
        self.black = true
    }

    #[inline]
    fn toggle_link(&mut self) {
        self.black = !self.black
    }

    #[inline]
    fn is_black(&self) -> bool {
        self.black
    }
}

/// Statistics on LLRB tree.
#[derive(Default, Debug)]
pub struct Stats {
    entries: usize, // number of entries in the tree.
    blacks: usize,
    depths: Depth,
    node_size: usize,
}

impl Stats {
    fn new(entries: usize, node_size: usize) -> Stats {
        Stats {
            entries,
            blacks: 0,
            depths: Depth::new(),
            node_size,
        }
    }

    pub fn entries(&self) -> usize {
        self.entries
    }

    pub fn blacks(&self) -> usize {
        self.blacks
    }

    pub fn node_size(&self) -> usize {
        self.node_size
    }

    pub fn depths(&self) -> Option<Depth> {
        if self.depths.samples() == 0 {
            None
        } else {
            Some(self.depths.clone())
        }
    }
}
