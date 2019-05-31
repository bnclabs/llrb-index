use std::{
    borrow::Borrow,
    cmp::{Ord, Ordering},
    mem,
    ops::{Bound, Deref, DerefMut, RangeBounds},
};

use rand::Rng;

use crate::depth::Depth;
use crate::error::Error;

// TODO: Should we make this configurable ???
const ITER_LIMIT: usize = 100;

/// Llrb manage a single instance of in-memory index using
/// [left-leaning-red-black][llrb] tree.
///
/// [llrb]: https://en.wikipedia.org/wiki/Left-leaning_red-black_tree
#[derive(Clone)]
pub struct Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    name: String,
    root: Option<Box<Node<K, V>>>,
    n_count: usize, // number of entries in the tree.
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
            root: Default::default(),
            n_count: Default::default(),
        }
    }

    /// Create a new instance of Llrb tree and load it with entries
    /// from `iter`. Note that iterator should return (key, value) tuples,
    /// where key must be ``unique``.
    pub fn load_from<S, I>(name: S, iter: I) -> Result<Llrb<K, V>, Error<K>>
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
    #[inline]
    pub fn id(&self) -> String {
        self.name.clone()
    }

    /// Return number of entries in this instance.
    #[inline]
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
        Stats::new(self.n_count, mem::size_of::<Node<K, V>>())
    }
}

type Insert<K, V> = (Box<Node<K, V>>, Option<Error<K>>);

type Upsert<K, V> = (Box<Node<K, V>>, Option<V>);

type Delete<K, V> = (Option<Box<Node<K, V>>>, Option<V>);

type Delmin<K, V> = (Option<Box<Node<K, V>>>, Option<Node<K, V>>);

/// Write operations on Llrb instance.
impl<K, V> Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    /// Create a new {key, value} entry in the index. If key is already
    /// present return error.
    pub fn create(&mut self, key: K, value: V) -> Result<(), Error<K>> {
        let (mut root, error) = Llrb::insert(self.root.take(), key, value);
        root.set_black();
        self.root = Some(root);
        match error {
            Some(err) => Err(err),
            None => {
                self.n_count += 1;
                Ok(())
            }
        }
    }

    /// Set value for key. If there is an existing entry for key,
    /// overwrite the old value with new value and return the old value.
    pub fn set(&mut self, key: K, value: V) -> Option<V> {
        let (mut root, old_value) = Llrb::upsert(self.root.take(), key, value);
        root.set_black();
        self.root = Some(root);
        match old_value {
            old_value @ Some(_) => old_value,
            None => {
                self.n_count += 1;
                None
            }
        }
    }

    /// Delete key from this instance and return its value. If key is
    /// not present, then delete is effectively a no-op.
    pub fn delete<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (root, old_value) = match Llrb::do_delete(self.root.take(), key) {
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
    ///
    /// * From root to any leaf, no consecutive reds allowed in its path.
    /// * Number of blacks should be same under left child and right child.
    /// * Make sure keys are in sorted order.
    ///
    /// Additionally return full statistics on the tree. Refer to [`Stats`]
    /// for more information.
    pub fn validate(&self) -> Result<Stats, Error<K>> {
        let root = self.root.as_ref().map(Deref::deref);
        let (red, nb, d) = (is_red(root), 0, 0);
        let mut stats = Stats::new(self.n_count, mem::size_of::<Node<K, V>>());
        stats.set_depths(Depth::new());
        let blacks = Llrb::validate_tree(root, red, nb, d, &mut stats)?;
        stats.set_blacks(blacks);
        Ok(stats)
    }
}

/// Read operations on Llrb instance.
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
        let mut node = self.root.as_ref().map(Deref::deref);
        while let Some(nref) = node {
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
        let mut nref = self.root.as_ref().map(Deref::deref)?;

        let mut at_depth = rng.gen::<u8>() % 40;
        loop {
            let next = match rng.gen::<u8>() % 2 {
                0 => nref.left_deref(),
                1 => nref.right_deref(),
                _ => unreachable!(),
            };
            if at_depth == 0 || next.is_none() {
                break Some((nref.key.clone(), nref.value.clone()));
            }
            at_depth -= 1;
            nref = next.unwrap();
        }
    }

    /// Return an iterator over all entries in this instance.
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            root: self.root.as_ref().map(Deref::deref),
            node_iter: vec![].into_iter(),
            after_key: Some(Bound::Unbounded),
            limit: ITER_LIMIT,
        }
    }

    /// Range over all entries from low to high.
    pub fn range<Q, R>(&self, range: R) -> Range<K, V>
    where
        K: Borrow<Q>,
        R: RangeBounds<Q>,
        Q: Ord + ToOwned<Owned = K> + ?Sized,
    {
        let low: Bound<K> = match range.start_bound() {
            Bound::Included(key) => Bound::Included(key.to_owned()),
            Bound::Excluded(key) => Bound::Excluded(key.to_owned()),
            Bound::Unbounded => Bound::Unbounded,
        };
        let high: Bound<K> = match range.end_bound() {
            Bound::Included(key) => Bound::Included(key.to_owned()),
            Bound::Excluded(key) => Bound::Excluded(key.to_owned()),
            Bound::Unbounded => Bound::Unbounded,
        };

        Range {
            root: self.root.as_ref().map(Deref::deref),
            node_iter: vec![].into_iter(),
            low: Some(low),
            high,
            limit: ITER_LIMIT,
        }
    }
}

impl<K, V> Llrb<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn insert(node: Option<Box<Node<K, V>>>, key: K, value: V) -> Insert<K, V> {
        if node.is_none() {
            return (Node::new(key, value, false /*black*/), None);
        }

        let mut node = Llrb::walkdown_rot23(node.unwrap());

        match node.key.cmp(&key) {
            Ordering::Greater => {
                let (left, e) = Llrb::insert(node.left.take(), key, value);
                node.left = Some(left);
                (Llrb::walkuprot_23(node), e)
            }
            Ordering::Less => {
                let (right, e) = Llrb::insert(node.right.take(), key, value);
                node.right = Some(right);
                (Llrb::walkuprot_23(node), e)
            }
            Ordering::Equal => {
                let err = Some(Error::OverwriteKey);
                (Llrb::walkuprot_23(node), err)
            }
        }
    }

    fn upsert(node: Option<Box<Node<K, V>>>, key: K, value: V) -> Upsert<K, V> {
        if node.is_none() {
            return (Node::new(key, value, false /*black*/), None);
        }

        let mut node = Llrb::walkdown_rot23(node.unwrap());

        match node.key.cmp(&key) {
            Ordering::Greater => {
                let (left, o) = Llrb::upsert(node.left.take(), key, value);
                node.left = Some(left);
                (Llrb::walkuprot_23(node), o)
            }
            Ordering::Less => {
                let (right, o) = Llrb::upsert(node.right.take(), key, value);
                node.right = Some(right);
                (Llrb::walkuprot_23(node), o)
            }
            Ordering::Equal => {
                let old_value = node.value.clone();
                node.set_value(value);
                (Llrb::walkuprot_23(node), Some(old_value))
            }
        }
    }

    fn do_delete<Q>(node: Option<Box<Node<K, V>>>, key: &Q) -> Delete<K, V>
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

    fn validate_tree(
        node: Option<&Node<K, V>>,
        fromred: bool,
        mut nb: usize,
        depth: usize,
        stats: &mut Stats,
    ) -> Result<usize, Error<K>> {
        if node.is_none() {
            stats.depths.as_mut().unwrap().sample(depth);
            return Ok(nb);
        }

        let red = is_red(node.as_ref().map(Deref::deref));
        if fromred && red {
            return Err(Error::ConsecutiveReds);
        }
        if !red {
            nb += 1;
        }
        let node = &node.as_ref().unwrap();
        let (left, right) = (node.left_deref(), node.right_deref());
        let lblacks = Llrb::validate_tree(left, red, nb, depth + 1, stats)?;
        let rblacks = Llrb::validate_tree(right, red, nb, depth + 1, stats)?;
        if lblacks != rblacks {
            let err = format!("left: {} right: {}", lblacks, rblacks);
            return Err(Error::UnbalancedBlacks(err));
        }
        if node.left.is_some() {
            let left = node.left.as_ref().unwrap();
            if left.key.ge(&node.key) {
                let (lkey, parent) = (left.key.clone(), node.key.clone());
                return Err(Error::SortError(lkey, parent));
            }
        }
        if node.right.is_some() {
            let right = node.right.as_ref().unwrap();
            if right.key.le(&node.key) {
                let (rkey, parent) = (right.key.clone(), node.key.clone());
                return Err(Error::SortError(rkey, parent));
            }
        }
        Ok(lblacks)
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
    node.map_or(false, |node| !node.is_black())
}

fn is_black<K, V>(node: Option<&Node<K, V>>) -> bool
where
    K: Clone + Ord,
    V: Clone,
{
    node.map_or(true, |node| node.is_black())
}

pub struct Iter<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    root: Option<&'a Node<K, V>>,
    node_iter: std::vec::IntoIter<(K, V)>,
    after_key: Option<Bound<K>>,
    limit: usize,
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

        let (left, right) = (node.left_deref(), node.right_deref());
        match &self.after_key {
            None => return false,
            Some(Bound::Included(akey)) | Some(Bound::Excluded(akey)) => {
                if node.key.borrow().le(akey) {
                    return self.scan_iter(right, acc);
                }
            }
            Some(Bound::Unbounded) => (),
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
        match self.node_iter.next() {
            None => {
                let mut a: Vec<(K, V)> = Vec::with_capacity(self.limit);
                self.scan_iter(self.root, &mut a);
                self.after_key = a.last().map(|x| Bound::Excluded(x.0.clone()));
                self.node_iter = a.into_iter();
                self.node_iter.next()
            }
            item @ Some(_) => item,
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
    low: Option<Bound<K>>,
    high: Bound<K>,
    limit: usize,
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
            low: self.low.unwrap(),
            high: Some(self.high),
            limit: self.limit,
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

        let (left, right) = (node.left_deref(), node.right_deref());
        match &self.low {
            Some(Bound::Included(qow)) if node.key.lt(qow) => {
                return self.range_iter(right, acc);
            }
            Some(Bound::Excluded(qow)) if node.key.le(qow) => {
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
        let item = match self.node_iter.next() {
            None if self.low.is_some() => {
                let mut acc: Vec<(K, V)> = Vec::with_capacity(self.limit);
                self.range_iter(self.root, &mut acc);
                self.low = acc.last().map(|x| Bound::Excluded(x.0.clone()));
                self.node_iter = acc.into_iter();
                self.node_iter.next()
            }
            None => None,
            item @ Some(_) => item,
        };
        // check for lower bound
        match item {
            None => None,
            Some(item) => match &self.high {
                Bound::Unbounded => Some(item),
                Bound::Included(qigh) if item.0.le(qigh) => Some(item),
                Bound::Excluded(qigh) if item.0.lt(qigh) => Some(item),
                _ => {
                    self.low = None;
                    None
                }
            },
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
    high: Option<Bound<K>>,
    low: Bound<K>,
    limit: usize,
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

        let (left, right) = (node.left_deref(), node.right_deref());
        match &self.high {
            Some(Bound::Included(qigh)) if node.key.gt(qigh) => {
                return self.reverse_iter(left, acc);
            }
            Some(Bound::Excluded(qigh)) if node.key.ge(qigh) => {
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
        let item = match self.node_iter.next() {
            None if self.high.is_some() => {
                let mut acc: Vec<(K, V)> = Vec::with_capacity(self.limit);
                self.reverse_iter(self.root, &mut acc);
                self.high = acc.last().map(|x| Bound::Excluded(x.0.clone()));
                self.node_iter = acc.into_iter();
                self.node_iter.next()
            }
            None => None,
            item @ Some(_) => item,
        };
        // check for lower bound
        match item {
            None => None,
            Some(item) => match &self.low {
                Bound::Unbounded => Some(item),
                Bound::Included(qow) if item.0.ge(qow) => Some(item),
                Bound::Excluded(qow) if item.0.gt(qow) => Some(item),
                _ => {
                    self.high = None;
                    None
                }
            },
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

    #[inline]
    fn left_deref(&self) -> Option<&Node<K, V>> {
        self.left.as_ref().map(Deref::deref)
    }

    #[inline]
    fn right_deref(&self) -> Option<&Node<K, V>> {
        self.right.as_ref().map(Deref::deref)
    }

    // prepend operation, equivalent to SET / INSERT / UPDATE
    #[inline]
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

/// Statistics on [`Llrb`] tree. Serves two purpose:
///
/// * To get partial but quick statistics via [`Llrb::stats`] method.
/// * To get full statisics via [`Llrb::validate`] method.
#[derive(Default, Debug)]
pub struct Stats {
    entries: usize, // number of entries in the tree.
    node_size: usize,
    blacks: Option<usize>,
    depths: Option<Depth>,
}

impl Stats {
    fn new(entries: usize, node_size: usize) -> Stats {
        Stats {
            entries,
            node_size,
            blacks: Default::default(),
            depths: Default::default(),
        }
    }

    #[inline]
    fn set_blacks(&mut self, blacks: usize) {
        self.blacks = Some(blacks)
    }

    #[inline]
    fn set_depths(&mut self, depths: Depth) {
        self.depths = Some(depths)
    }

    /// Return number entries in [`Llrb`] instance.
    #[inline]
    pub fn entries(&self) -> usize {
        self.entries
    }

    /// Return node-size, including over-head for `Llrb<k,V>`. Although
    /// the node overhead is constant, the node size varies based on
    /// key and value types. EG:
    ///
    /// ```
    /// use llrb_index::Llrb;
    /// let mut llrb: Llrb<u64,i128> = Llrb::new("myinstance");
    ///
    /// // size of key: 8 bytes
    /// // size of value: 16 bytes
    /// // overhead is 24 bytes
    /// assert_eq!(llrb.stats().node_size(), 48);
    /// ```
    #[inline]
    pub fn node_size(&self) -> usize {
        self.node_size
    }

    /// Return number of black nodes from root to leaf, on both left
    /// and right child.
    #[inline]
    pub fn blacks(&self) -> Option<usize> {
        self.blacks
    }

    /// Return [`Depth`] statistics.
    pub fn depths(&self) -> Option<Depth> {
        if self.depths.as_ref().unwrap().samples() == 0 {
            None
        } else {
            self.depths.clone()
        }
    }
}
