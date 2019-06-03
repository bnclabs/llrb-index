use std::convert::TryInto;

#[derive(Clone)]
struct RefNode {
    key: i64,
    value: i64,
}

struct RefNodes {
    entries: Vec<RefNode>,
}

impl RefNodes {
    fn new(capacity: usize) -> RefNodes {
        let mut entries: Vec<RefNode> = Vec::with_capacity(capacity);
        (0..capacity).for_each(|_| entries.push(RefNode { key: -1, value: 0 }));
        RefNodes { entries }
    }

    fn get(&self, key: i64) -> Option<i64> {
        let off: usize = key.try_into().unwrap();
        let entry = self.entries[off].clone();
        if entry.key < 0 {
            None
        } else {
            Some(entry.value)
        }
    }

    fn iter(&self) -> std::vec::IntoIter<(i64, i64)> {
        self.entries
            .iter()
            .filter_map(|item| {
                if item.key < 0 {
                    None
                } else {
                    Some((item.key, item.value))
                }
            })
            .collect::<Vec<(i64, i64)>>()
            .into_iter()
    }

    fn range(&self, low: Bound<i64>, high: Bound<i64>) -> std::vec::IntoIter<(i64, i64)> {
        let low = match low {
            Bound::Included(low) => low.try_into().unwrap(),
            Bound::Excluded(low) => (low + 1).try_into().unwrap(),
            Bound::Unbounded => 0,
        };
        let high = match high {
            Bound::Included(high) => (high + 1).try_into().unwrap(),
            Bound::Excluded(high) => high.try_into().unwrap(),
            Bound::Unbounded => self.entries.len(),
        };
        let ok = low < self.entries.len();
        let ok = ok && (high >= low && high <= self.entries.len());
        let entries = if ok {
            &self.entries[low..high]
        } else {
            &self.entries[..0]
        };

        entries
            .iter()
            .filter_map(|item| {
                if item.key < 0 {
                    None
                } else {
                    Some((item.key, item.value))
                }
            })
            .collect::<Vec<(i64, i64)>>()
            .into_iter()
    }

    fn reverse(&self, low: Bound<i64>, high: Bound<i64>) -> std::vec::IntoIter<(i64, i64)> {
        let low = match low {
            Bound::Included(low) => low.try_into().unwrap(),
            Bound::Excluded(low) => (low + 1).try_into().unwrap(),
            Bound::Unbounded => 0,
        };
        let high = match high {
            Bound::Included(high) => (high + 1).try_into().unwrap(),
            Bound::Excluded(high) => high.try_into().unwrap(),
            Bound::Unbounded => self.entries.len(),
        };
        //println!("reverse ref compute low high {} {}", low, high);
        let ok = low < self.entries.len();
        let ok = ok && (high >= low && high <= self.entries.len());
        let entries = if ok {
            &self.entries[low..high]
        } else {
            &self.entries[..0]
        };

        entries
            .iter()
            .rev()
            .filter_map(|item| {
                if item.key < 0 {
                    None
                } else {
                    Some((item.key, item.value))
                }
            })
            .collect::<Vec<(i64, i64)>>()
            .into_iter()
    }

    fn create(&mut self, key: i64, value: i64) {
        let off: usize = key.try_into().unwrap();
        let entry = &mut self.entries[off];
        if entry.key < 0 {
            entry.key = key;
            entry.value = value;
        };
    }

    fn set(&mut self, key: i64, value: i64) -> Option<i64> {
        let off: usize = key.try_into().unwrap();
        let entry = &mut self.entries[off];
        let old_value = if entry.key < 0 {
            None
        } else {
            Some(entry.value)
        };
        entry.key = key;
        entry.value = value;
        old_value
    }

    fn delete(&mut self, key: i64) -> Option<i64> {
        let off: usize = key.try_into().unwrap();
        let entry = &mut self.entries[off];
        if entry.key < 0 {
            None
        } else {
            entry.key = -1;
            Some(entry.value)
        }
    }
}

fn random_low_high(size: usize) -> (Bound<i64>, Bound<i64>) {
    let size = size as u64;
    let low = (random::<u64>() % size) as i64;
    let high = (random::<u64>() % size) as i64;
    let low = match random::<u8>() % 3 {
        0 => Bound::Included(low),
        1 => Bound::Excluded(low),
        2 => Bound::Unbounded,
        _ => unreachable!(),
    };
    let high = match random::<u8>() % 3 {
        0 => Bound::Included(high),
        1 => Bound::Excluded(high),
        2 => Bound::Unbounded,
        _ => unreachable!(),
    };
    //println!("low_high {:?} {:?}", low, high);
    (low, high)
}
