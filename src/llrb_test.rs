use std::ops::Bound;

use rand::prelude::random;

use crate::llrb::Llrb;

#[test]
fn test_id() {
    let llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    assert_eq!(llrb.id(), "test-llrb".to_string());
}

#[test]
fn test_count() {
    let llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    assert_eq!(llrb.count(), 0);
}

#[test]
fn test_set() {
    let mut llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    let mut refns = RefNodes::new(10);

    assert!(llrb.set(2, 10).is_none());
    refns.set(2, 10);
    assert!(llrb.set(1, 10).is_none());
    refns.set(1, 10);
    assert!(llrb.set(3, 10).is_none());
    refns.set(3, 10);
    assert!(llrb.set(6, 10).is_none());
    refns.set(6, 10);
    assert!(llrb.set(5, 10).is_none());
    refns.set(5, 10);
    assert!(llrb.set(4, 10).is_none());
    refns.set(4, 10);
    assert!(llrb.set(8, 10).is_none());
    refns.set(8, 10);
    assert!(llrb.set(0, 10).is_none());
    refns.set(0, 10);
    assert!(llrb.set(9, 10).is_none());
    refns.set(9, 10);
    assert!(llrb.set(7, 10).is_none());
    refns.set(7, 10);

    assert_eq!(llrb.count(), 10);
    assert_eq!(llrb.validate(), Ok(()));

    // test get
    for i in 0..10 {
        let val = llrb.get(&i);
        let refval = refns.get(i);
        assert_eq!(val, refval);
    }
    // test iter
    let (mut iter, mut iter_ref) = (llrb.iter(), refns.iter());
    loop {
        match (iter.next(), iter_ref.next()) {
            (Some(item), Some(ref_item)) => {
                assert_eq!(item.0, ref_item.0);
                assert_eq!(item.1, ref_item.1);
            }
            (None, None) => break,
            (_, _) => panic!("invalid"),
        }
    }
}

#[test]
fn test_delete() {
    let mut llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    let mut refns = RefNodes::new(11);

    assert!(llrb.set(2, 100).is_none());
    refns.set(2, 100);
    assert!(llrb.set(1, 100).is_none());
    refns.set(1, 100);
    assert!(llrb.set(3, 100).is_none());
    refns.set(3, 100);
    assert!(llrb.set(6, 100).is_none());
    refns.set(6, 100);
    assert!(llrb.set(5, 100).is_none());
    refns.set(5, 100);
    assert!(llrb.set(4, 100).is_none());
    refns.set(4, 100);
    assert!(llrb.set(8, 100).is_none());
    refns.set(8, 100);
    assert!(llrb.set(0, 100).is_none());
    refns.set(0, 100);
    assert!(llrb.set(9, 100).is_none());
    refns.set(9, 100);
    assert!(llrb.set(7, 100).is_none());
    refns.set(7, 100);

    // delete a missing node.
    assert!(llrb.delete(&10).is_none());
    assert!(refns.delete(10).is_none());

    assert_eq!(llrb.count(), 10);
    assert_eq!(llrb.validate(), Ok(()));

    // test iter
    {
        let (mut iter, mut iter_ref) = (llrb.iter(), refns.iter());
        loop {
            match (iter.next(), iter_ref.next()) {
                (Some(item), Some(ref_item)) => {
                    assert_eq!(item.0, ref_item.0);
                    assert_eq!(item.1, ref_item.1);
                }
                (None, None) => break,
                (_, _) => panic!("invalid"),
            }
        }
    }

    // delete all entry. And set new entries
    for i in 0..10 {
        let val = llrb.delete(&i);
        let refval = refns.delete(i);
        assert_eq!(val, refval);
    }
    assert_eq!(llrb.count(), 0);
    assert_eq!(llrb.validate(), Ok(()));
    // test iter
    assert!(llrb.iter().next().is_none());
}

#[test]
fn test_crud() {
    let size = 1000;
    let mut llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    let mut refns = RefNodes::new(size);

    for _ in 0..100_000 {
        let key: i64 = (random::<i64>() % (size as i64)).abs();
        let value: i64 = random();
        let op: i64 = (random::<i64>() % 2).abs();
        //println!("key {} value {} op {}", key, value, op);
        match op {
            0 => {
                let val = llrb.set(key, value);
                let refval = refns.set(key, value);
                assert_eq!(val, refval);
                false
            }
            1 => {
                let val = llrb.delete(&key);
                let refval = refns.delete(key);
                assert_eq!(val, refval);
                true
            }
            op => panic!("unreachable {}", op),
        };

        assert_eq!(llrb.validate(), Ok(()));
    }

    println!("count {}", llrb.count());

    // test iter
    let (mut iter, mut iter_ref) = (llrb.iter(), refns.iter());
    loop {
        match (iter.next(), iter_ref.next()) {
            (Some(item), Some(ref_item)) => {
                assert_eq!(item.0, ref_item.0);
                assert_eq!(item.1, ref_item.1);
            }
            (None, None) => break,
            (_, _) => panic!("invalid"),
        }
    }

    // ranges and reverses
    for _ in 0..10000 {
        let (low, high) = random_low_high(size);
        //println!("test loop {:?} {:?}", low, high);

        let mut iter = llrb.range(low, high);
        let mut iter_ref = refns.range(low, high);
        loop {
            match (iter.next(), iter_ref.next()) {
                (Some(item), Some(ref_item)) => {
                    //println!("{:?} {:?}", ref_item, item);
                    assert_eq!(item.0, ref_item.0);
                    assert_eq!(item.1, ref_item.1);
                }
                (None, None) => break,
                (Some(item), None) => panic!("invalid item: {:?}", item),
                (None, Some(ref_item)) => panic!("invalid none: {:?}", ref_item),
            }
        }

        let mut iter = llrb.range(low, high).rev();
        let mut iter_ref = refns.reverse(low, high);
        loop {
            match (iter.next(), iter_ref.next()) {
                (Some(item), Some(ref_item)) => {
                    assert_eq!(item.0, ref_item.0);
                    assert_eq!(item.1, ref_item.1);
                }
                (None, None) => break,
                (_, _) => panic!("invalid"),
            }
        }
    }
}

include!("./ref_test.rs");
