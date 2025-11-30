use core::hash::{Hash, Hasher};
use error_rail::types::accumulator::Accumulator;
use error_rail::types::ErrorVec;
use std::collections::hash_map::DefaultHasher;

#[test]
fn test_accumulator_new() {
    let acc: Accumulator<i32> = Accumulator::new();
    assert!(acc.is_empty());
    assert_eq!(acc.len(), 0);
}

#[test]
fn test_accumulator_push() {
    let mut acc = Accumulator::new();
    acc.push(1);
    acc.push(2);
    acc.push(3);

    assert_eq!(acc.len(), 3);
    assert!(!acc.is_empty());
}

#[test]
fn test_accumulator_pop() {
    let mut acc = Accumulator::new();
    acc.push(1);
    acc.push(2);

    assert_eq!(acc.pop(), Some(2));
    assert_eq!(acc.pop(), Some(1));
    assert_eq!(acc.pop(), None);

    assert!(acc.is_empty());
    assert_eq!(acc.len(), 0);
}

#[test]
fn test_accumulator_pop_empty() {
    let mut acc: Accumulator<i32> = Accumulator::new();
    assert_eq!(acc.pop(), None);
}

#[test]
fn test_accumulator_extend() {
    let mut acc = Accumulator::new();
    acc.push(1);

    acc.extend(vec![2, 3, 4]);

    assert_eq!(acc.len(), 4);

    let items: Vec<i32> = acc.iter().copied().collect();
    assert_eq!(items, vec![1, 2, 3, 4]);
}

#[test]
fn test_accumulator_extend_empty() {
    let mut acc = Accumulator::new();
    acc.extend(Vec::<i32>::new());

    assert!(acc.is_empty());
    assert_eq!(acc.len(), 0);
}

#[test]
fn test_accumulator_extend_from_empty() {
    let mut acc: Accumulator<i32> = Accumulator::new();
    acc.extend(vec![1, 2, 3]);

    assert_eq!(acc.len(), 3);

    let items: Vec<i32> = acc.iter().copied().collect();
    assert_eq!(items, vec![1, 2, 3]);
}

#[test]
fn test_accumulator_iter() {
    let mut acc = Accumulator::new();
    acc.push(10);
    acc.push(20);
    acc.push(30);

    let items: Vec<i32> = acc.iter().copied().collect();
    assert_eq!(items, vec![10, 20, 30]);
}

#[test]
fn test_accumulator_iter_empty() {
    let acc: Accumulator<i32> = Accumulator::new();
    let items: Vec<i32> = acc.iter().copied().collect();
    assert_eq!(items, Vec::<i32>::new());
}

#[test]
fn test_accumulator_iter_mut() {
    let mut acc = Accumulator::new();
    acc.push(1);
    acc.push(2);
    acc.push(3);

    for item in acc.iter_mut() {
        *item *= 2;
    }

    let items: Vec<i32> = acc.iter().copied().collect();
    assert_eq!(items, vec![2, 4, 6]);
}

#[test]
fn test_accumulator_iter_mut_empty() {
    let mut acc: Accumulator<i32> = Accumulator::new();
    for item in acc.iter_mut() {
        *item = 1; // Should never execute
    }
    assert!(acc.is_empty());
}

#[test]
fn test_accumulator_into_inner() {
    let mut acc = Accumulator::new();
    acc.push(100);
    acc.push(200);

    let inner: ErrorVec<i32> = acc.into_inner();
    let items: Vec<i32> = inner.into_iter().collect();
    assert_eq!(items, vec![100, 200]);
}

#[test]
fn test_accumulator_into_inner_empty() {
    let acc: Accumulator<i32> = Accumulator::new();
    let inner: ErrorVec<i32> = acc.into_inner();
    assert!(inner.is_empty());
}

#[test]
fn test_accumulator_debug() {
    let mut acc = Accumulator::new();
    acc.push(1);
    acc.push(2);

    let debug_str = format!("{:?}", acc);
    assert!(debug_str.contains("Accumulator"));
    assert!(debug_str.contains("1"));
    assert!(debug_str.contains("2"));
}

#[test]
fn test_accumulator_clone() {
    let mut acc1 = Accumulator::new();
    acc1.push("hello");
    acc1.push("world");

    let acc2 = acc1.clone();

    assert_eq!(acc1.len(), acc2.len());
    assert_eq!(
        acc1.iter().collect::<Vec<_>>(),
        acc2.iter().collect::<Vec<_>>()
    );

    // Verify they are independent
    acc1.push("extra");
    assert_ne!(acc1.len(), acc2.len());
}

#[test]
fn test_accumulator_partial_ord() {
    let acc1: Accumulator<i32> = vec![1, 2, 3].into_iter().collect();
    let acc2: Accumulator<i32> = vec![1, 2, 4].into_iter().collect();
    let acc3: Accumulator<i32> = vec![1, 2, 3].into_iter().collect();

    assert!(acc1 < acc2);
    assert!(acc2 > acc1);
    assert_eq!(acc1.partial_cmp(&acc3), Some(core::cmp::Ordering::Equal));
}

#[test]
fn test_accumulator_ord() {
    let acc1: Accumulator<i32> = vec![1, 2, 3].into_iter().collect();
    let acc2: Accumulator<i32> = vec![1, 2, 4].into_iter().collect();
    let acc3: Accumulator<i32> = vec![1, 2, 3].into_iter().collect();

    assert!(acc1 < acc2);
    assert!(acc2 > acc1);
    assert_eq!(acc1.cmp(&acc3), core::cmp::Ordering::Equal);
}

#[test]
fn test_accumulator_eq() {
    let acc1: Accumulator<i32> = vec![1, 2, 3].into_iter().collect();
    let acc2: Accumulator<i32> = vec![1, 2, 3].into_iter().collect();
    let acc3: Accumulator<i32> = vec![1, 2, 4].into_iter().collect();

    assert_eq!(acc1, acc2);
    assert_ne!(acc1, acc3);
}

#[test]
fn test_accumulator_hash() {
    let acc1: Accumulator<i32> = vec![1, 2, 3].into_iter().collect();
    let acc2: Accumulator<i32> = vec![1, 2, 3].into_iter().collect();
    let acc3: Accumulator<i32> = vec![1, 2, 4].into_iter().collect();

    let mut hasher1 = DefaultHasher::new();
    acc1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    acc2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    acc3.hash(&mut hasher3);
    let hash3 = hasher3.finish();

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_accumulator_from_error_vec() {
    let error_vec: ErrorVec<i32> = vec![10, 20, 30].into_iter().collect();
    let acc = Accumulator::from(error_vec);

    assert_eq!(acc.len(), 3);
    let items: Vec<i32> = acc.iter().copied().collect();
    assert_eq!(items, vec![10, 20, 30]);
}

#[test]
fn test_accumulator_from_iterator() {
    let vec = vec![1, 2, 3, 4, 5];
    let acc: Accumulator<i32> = vec.into_iter().collect();

    assert_eq!(acc.len(), 5);
    let items: Vec<i32> = acc.iter().copied().collect();
    assert_eq!(items, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_accumulator_from_iterator_empty() {
    let vec: Vec<i32> = vec![];
    let acc: Accumulator<i32> = vec.into_iter().collect();

    assert!(acc.is_empty());
    assert_eq!(acc.len(), 0);
}

#[test]
fn test_accumulator_into_iterator() {
    let mut acc = Accumulator::new();
    acc.push(100);
    acc.push(200);
    acc.push(300);

    let items: Vec<i32> = acc.into_iter().collect();
    assert_eq!(items, vec![100, 200, 300]);
}

#[test]
fn test_accumulator_into_iterator_empty() {
    let acc: Accumulator<i32> = Accumulator::new();
    let items: Vec<i32> = acc.into_iter().collect();
    assert_eq!(items, Vec::<i32>::new());
}

#[test]
fn test_accumulator_default() {
    let acc: Accumulator<String> = Accumulator::default();
    assert!(acc.is_empty());
    assert_eq!(acc.len(), 0);
}

#[test]
fn test_accumulator_with_strings() {
    let mut acc = Accumulator::new();
    acc.push("hello".to_string());
    acc.push("world".to_string());
    acc.push("test".to_string());

    assert_eq!(acc.len(), 3);

    let items: Vec<String> = acc.iter().cloned().collect();
    assert_eq!(
        items,
        vec!["hello".to_string(), "world".to_string(), "test".to_string()]
    );
}

#[test]
fn test_accumulator_with_complex_types() {
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct TestStruct {
        id: u32,
        name: String,
    }

    let mut acc = Accumulator::new();
    acc.push(TestStruct {
        id: 1,
        name: "first".to_string(),
    });
    acc.push(TestStruct {
        id: 2,
        name: "second".to_string(),
    });

    assert_eq!(acc.len(), 2);

    let items: Vec<TestStruct> = acc.iter().cloned().collect();
    assert_eq!(items[0].id, 1);
    assert_eq!(items[1].id, 2);
}

#[test]
fn test_accumulator_large_capacity() {
    let mut acc = Accumulator::new();

    // Add many items to test beyond inline storage
    for i in 0..100 {
        acc.push(i);
    }

    assert_eq!(acc.len(), 100);

    let items: Vec<i32> = acc.iter().copied().collect();
    for (i, &item) in items.iter().enumerate() {
        assert_eq!(item, i as i32);
    }
}

#[test]
fn test_accumulator_extend_with_iterator() {
    let mut acc = Accumulator::new();
    acc.push(0);

    let iter = 1..=10;
    acc.extend(iter);

    assert_eq!(acc.len(), 11);

    let items: Vec<i32> = acc.iter().copied().collect();
    for (i, &item) in items.iter().enumerate() {
        assert_eq!(item, i as i32);
    }
}

#[test]
fn test_accumulator_mixed_operations() {
    let mut acc = Accumulator::new();

    // Push some items
    acc.push(1);
    acc.push(2);
    acc.push(3);

    // Pop one
    assert_eq!(acc.pop(), Some(3));

    // Extend with more
    acc.extend(vec![4, 5]);

    // Final state should be [1, 2, 4, 5]
    assert_eq!(acc.len(), 4);
    let items: Vec<i32> = acc.iter().copied().collect();
    assert_eq!(items, vec![1, 2, 4, 5]);
}

#[test]
fn test_accumulator_iteration_order() {
    let mut acc = Accumulator::new();
    acc.push("first");
    acc.push("second");
    acc.push("third");

    let items: Vec<&str> = acc.iter().copied().collect();
    assert_eq!(items, vec!["first", "second", "third"]);
}

#[test]
fn test_accumulator_into_iter_consumes() {
    let mut acc = Accumulator::new();
    acc.push(1);
    acc.push(2);

    let items: Vec<i32> = acc.into_iter().collect();
    assert_eq!(items, vec![1, 2]);

    // acc is now moved and can't be used
}

#[test]
fn test_accumulator_with_different_numeric_types() {
    let mut acc_i32: Accumulator<i32> = Accumulator::new();
    let mut acc_f64: Accumulator<f64> = Accumulator::new();
    let mut acc_bool: Accumulator<bool> = Accumulator::new();

    acc_i32.push(42);
    acc_f64.push(3.14);
    acc_bool.push(true);

    assert_eq!(acc_i32.len(), 1);
    assert_eq!(acc_f64.len(), 1);
    assert_eq!(acc_bool.len(), 1);

    assert_eq!(acc_i32.iter().next(), Some(&42));
    assert_eq!(acc_f64.iter().next(), Some(&3.14));
    assert_eq!(acc_bool.iter().next(), Some(&true));
}
