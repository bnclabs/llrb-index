use std::ops::Bound;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::prelude::random;
use rand::{rngs::SmallRng, SeedableRng};

use crate::error::LlrbError;
use crate::llrb::Llrb;

#[test]
fn test_id() {
    let llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    assert_eq!(llrb.id(), "test-llrb".to_string());
}

#[test]
fn test_len() {
    let llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    assert_eq!(llrb.len(), 0);
}

#[test]
fn test_create() {
    let mut llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    let mut refns = RefNodes::new(10);

    assert!(llrb.create(2, 10).is_none());
    refns.create(2, 10);
    assert!(llrb.create(1, 10).is_none());
    refns.create(1, 10);
    assert!(llrb.create(3, 10).is_none());
    refns.create(3, 10);
    assert!(llrb.create(6, 10).is_none());
    refns.create(6, 10);
    assert!(llrb.create(5, 10).is_none());
    refns.create(5, 10);
    assert!(llrb.create(4, 10).is_none());
    refns.create(4, 10);
    assert!(llrb.create(8, 10).is_none());
    refns.create(8, 10);
    assert!(llrb.create(0, 10).is_none());
    refns.create(0, 10);
    assert!(llrb.create(9, 10).is_none());
    refns.create(9, 10);
    assert!(llrb.create(7, 10).is_none());
    refns.create(7, 10);

    assert_eq!(llrb.len(), 10);
    assert!(llrb.validate().is_ok());

    // error case
    assert_eq!(llrb.create(7, 20), Some(LlrbError::OverwriteKey));

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
fn test_random() {
    let mut llrb: Llrb<i64, i64> = Llrb::new("test-llrb");
    let mut rng = SmallRng::from_seed(make_seed().to_le_bytes());

    assert_eq!(llrb.random(&mut rng), None);

    assert!(llrb.create(0, 0).is_none());
    assert_eq!(llrb.random(&mut rng), Some((0, 0)));
    assert_eq!(llrb.random(&mut rng), Some((0, 0)));

    for key in 1..1_000_000 {
        assert!(llrb.set(key, key * 10).is_none());
    }
    for _i in 0..2_000_000 {
        let (key, value) = llrb.random(&mut rng).unwrap();
        assert!(key >= 0 && key < 1_000_000);
        assert_eq!(value, key * 10);
    }
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

    assert_eq!(llrb.len(), 10);
    assert!(llrb.validate().is_ok());

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

    assert_eq!(llrb.len(), 10);
    assert!(llrb.validate().is_ok());

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
    assert_eq!(llrb.len(), 0);
    assert!(llrb.validate().is_ok());
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
        let op: i64 = (random::<i64>() % 4).abs();
        //println!("key {} value {} op {}", key, value, op);
        match op {
            0 => {
                let ok1 = llrb.get(&key).is_none();
                let ok2 = llrb.create(key, value).is_none();
                refns.create(key, value);
                assert_eq!(ok1, ok2);
                false
            }
            1 => {
                let val = llrb.set(key, value);
                let refval = refns.set(key, value);
                assert_eq!(val, refval);
                false
            }
            2 => {
                let val = llrb.delete(&key);
                let refval = refns.delete(key);
                assert_eq!(val, refval);
                true
            }
            3 => {
                let val = llrb.get(&key);
                let refval = refns.get(key);
                assert_eq!(val, refval);
                true
            }
            op => panic!("unreachable {}", op),
        };

        assert!(llrb.validate().is_ok());
    }

    println!("index-length {}", llrb.len());

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
    for _ in 0..1_0000 {
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

fn make_seed() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

include!("./ref_test.rs");
